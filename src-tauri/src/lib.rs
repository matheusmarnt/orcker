pub mod adapters;
pub mod commands;
pub mod core;

use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use crate::core::state::AppState;
use crate::adapters::docker::client::DockerAdapter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // STEP 1: Build tauri-specta collection BEFORE tauri::Builder
    // This must happen at startup — not in build.rs
    let specta_builder = SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::docker::get_docker_version,
            commands::docker::list_containers,
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

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("orcker=debug".parse().unwrap())
                .add_directive("bollard=warn".parse().unwrap())
                .add_directive("tauri=info".parse().unwrap()),
        )
        .init();

    // STEP 3: Build Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: None },
                ))
                .build(),
        )
        .setup(|app| {
            // Register disconnected state IMMEDIATELY — window opens before Docker probe
            app.manage(AppState::disconnected());

            // Spawn async Docker connection probe — NEVER block setup()
            // setup() runs on the main thread; .await here would deadlock
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
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
                    ).await;
                }
                drop(guard);
                // If subscribe_events returns, Docker disconnected — retry
                tracing::warn!("Docker event stream ended — retrying in 4s");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Docker probe failed");
                handle.emit("docker://error", serde_json::to_string(&e).unwrap_or_default()).ok();
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
    }
}
