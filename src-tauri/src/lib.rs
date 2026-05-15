pub mod adapters;
pub mod commands;
pub mod core;

use crate::adapters::docker::client::DockerAdapter;
use crate::core::compose::detect_compose_driver;
use crate::core::projects::ProjectsState;
use crate::core::state::AppState;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, Builder as SpectaBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // STEP 1: Build tauri-specta collection BEFORE tauri::Builder
    // This must happen at startup — not in build.rs
    let specta_builder = SpectaBuilder::<tauri::Wry>::new().commands(collect_commands![
        commands::docker::get_docker_version,
        commands::docker::list_containers,
        commands::global_stack::toggle_service,
        commands::global_stack::get_services_status,
        commands::global_stack::get_service_configs,
        commands::global_stack::set_service_config,
        commands::global_stack::global_on,
        commands::global_stack::global_off,
        commands::projects::pick_project_folder,
        commands::projects::register_project,
        commands::projects::import_project,
        commands::projects::list_projects,
        commands::projects::get_compose_driver,
        commands::artisan::list_artisan_commands,
        commands::artisan::run_artisan_command,
        commands::artisan::cancel_artisan_command,
        commands::projects::scaffold_project,
        commands::logs::start_log_stream,
        commands::logs::stop_log_stream,
    ]);

    // Export TypeScript bindings in debug builds only
    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/ipc/bindings.ts",
        )
        .expect("Failed to export tauri-specta bindings — check src/ipc/ directory exists");

    // STEP 2: Wire tracing + log bridge
    // tracing-log bridge: routes log::* macros (from tauri internals) into tracing
    // MUST be initialized before tracing_subscriber::fmt().init()
    tracing_log::LogTracer::init().ok();

    // try_init: tauri_plugin_log may already set a log subscriber — ignore conflict
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("orcker=debug".parse().unwrap())
                .add_directive("bollard=warn".parse().unwrap())
                .add_directive("tauri=info".parse().unwrap()),
        )
        .try_init();

    // STEP 3: Build Tauri app
    // Note: tauri_plugin_log conflicts with tracing_log::LogTracer (both own log global)
    // tracing_subscriber handles stdout — tauri_plugin_log not needed
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["CmdOrCtrl+Shift+G"])
                .expect("failed to build shortcut")
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        app.emit("global://shortcut-toggle", ()).ok();
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Detect compose driver synchronously before window opens (R-M2.5)
            // detect_compose_driver() shells out to `docker compose version` / `docker-compose version`
            let compose_driver = detect_compose_driver();
            app.manage(ProjectsState::new(compose_driver));

            // Register disconnected state IMMEDIATELY — window opens before Docker probe
            app.manage(AppState::disconnected());

            // Spawn async init: hydrate persisted configs → then probe Docker
            // setup() runs on the main thread; .await here would deadlock
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Load saved service configs from store before first Docker probe
                if let Ok(store) = handle.store("orcker-services.json") {
                    let state = handle.state::<AppState>();
                    let mut configs = state.global_stack.configs.write().await;
                    for id in crate::core::global_stack::ServiceId::all() {
                        let key = format!("{:?}", id);
                        if let Some(val) = store.get(&key) {
                            if let Ok(config) = serde_json::from_value::<
                                crate::core::global_stack::ServiceConfig,
                            >(val)
                            {
                                configs.insert(id, config);
                            }
                        }
                    }
                }

                probe_docker_and_update(handle).await;
            });

            Ok(())
        })
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Probe Docker socket in background, update AppState on success or emit error event.
/// Auto-retries every 4 seconds if Docker is not initially reachable.
async fn probe_docker_and_update(handle: tauri::AppHandle) {
    // Wait for Vue to mount and register event listeners before first emit
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    loop {
        tracing::debug!("Probing Docker socket...");
        match DockerAdapter::connect().await {
            Ok((adapter, socket_path)) => {
                tracing::info!(socket = %socket_path, "Docker connected");
                let state = handle.state::<AppState>();
                {
                    let mut docker_guard = state.docker.write().await;
                    *docker_guard = Some(adapter);
                }
                {
                    let mut socket_guard = state.docker_socket.write().await;
                    *socket_guard = Some(socket_path.clone());
                }
                // Notify frontend that Docker is connected
                handle.emit("docker://connected", socket_path).ok();

                // Subscribe to container events (runs until stream breaks)
                let app_state = handle.state::<AppState>();
                let guard = app_state.docker.read().await;
                if let Some(adapter) = guard.as_ref() {
                    adapters::docker::containers::subscribe_events(
                        adapter.client.clone(),
                        handle.clone(),
                    )
                    .await;
                }
                drop(guard);
                // If subscribe_events returns, Docker disconnected — retry
                tracing::warn!("Docker event stream ended — retrying in 4s");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Docker probe failed");
                handle
                    .emit(
                        "docker://error",
                        serde_json::to_string(&e).unwrap_or_default(),
                    )
                    .ok();
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
    }
}
