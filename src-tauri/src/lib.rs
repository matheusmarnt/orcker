pub mod adapters;
pub mod commands;
pub mod core;

use crate::adapters::docker::client::DockerAdapter;
use crate::core::compose::detect_compose_driver;
use crate::core::projects::ProjectsState;
use crate::core::settings::{AppSettings, AppSettingsData};
use crate::core::state::AppState;
use std::sync::atomic::Ordering;
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_store::StoreExt;
use tauri_specta::{collect_commands, collect_events, Builder as SpectaBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // STEP 1: Build tauri-specta collection BEFORE tauri::Builder
    // This must happen at startup — not in build.rs
    let specta_builder = SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
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
            commands::projects::read_env_file,
            commands::projects::save_env_file,
            commands::projects::toggle_vite_auto,
            commands::projects::generate_xdebug_config,
            commands::projects::read_php_ini,
            commands::projects::save_php_ini,
            commands::projects::list_supervisor_workers,
            commands::projects::restart_supervisor_worker,
            commands::projects::open_project_folder,
            commands::projects::start_project,
            commands::projects::stop_project,
            commands::projects::get_project_status,
            commands::compose::read_compose_file,
            commands::compose::save_compose_file,
            commands::compose::get_compose_diff,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::database::create_testing_db,
            commands::database::dump_db,
            commands::database::restore_db,
            commands::database::open_db_cli,
            commands::infra::list_volumes,
            commands::infra::prune_volumes,
            commands::infra::list_images,
            commands::infra::pull_image,
            commands::infra::remove_image,
            commands::infra::prune_images,
        ])
        .events(collect_events![crate::core::projects::ProjectStatusEvent]);

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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--silent"]),
        ))
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

            // Load settings from store (or use Default if missing)
            let settings_data = if let Ok(store) = app.store("settings.json") {
                store
                    .get("settings")
                    .and_then(|v| serde_json::from_value::<AppSettingsData>(v).ok())
                    .unwrap_or_default()
            } else {
                AppSettingsData::default()
            };
            app.manage(AppSettings::new(settings_data));

            // Build system tray (guard prevents duplicates on hot-reload)
            if app.tray_by_id("main").is_none() {
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let start_i = MenuItem::with_id(app, "start_all", "Start All", true, None::<&str>)?;
                let stop_i = MenuItem::with_id(app, "stop_all", "Stop All", true, None::<&str>)?;

                // Recent projects submenu — read up to 5 projects from ProjectsState
                // At setup time the list is empty (projects are loaded later from store),
                // so we always show the "(no projects)" placeholder here. The tray menu
                // is rebuilt whenever the user opens the tray, so this is acceptable.
                let none_i =
                    MenuItem::with_id(app, "no_projects", "(no projects)", false, None::<&str>)?;
                let recent_sub = Submenu::with_items(app, "Recent Projects", true, &[&none_i])?;

                let menu = Menu::with_items(app, &[&start_i, &stop_i, &recent_sub, &quit_i])?;

                TrayIconBuilder::with_id("main")
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "start_all" => {
                            // TODO: invoke global_stack_on via app_handle (future plan)
                        }
                        "stop_all" => {
                            // TODO: invoke global_stack_off via app_handle (future plan)
                        }
                        id if id.starts_with("project:") => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    })
                    .build(app)?;
            }

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
        .on_window_event(|window, event| {
            // Close-to-tray: when tray is enabled, hide window instead of closing
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let settings = app.state::<AppSettings>();
                if settings.tray_enabled.load(Ordering::Relaxed) {
                    window.hide().ok();
                    api.prevent_close();
                }
            }
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
