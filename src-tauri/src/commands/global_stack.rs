#![allow(deprecated)]
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;

use crate::core::error::AppError;
use crate::core::global_stack::{ServiceConfig, ServiceId, ServiceStatus};
use crate::core::state::AppState;
use crate::adapters::docker::networks::ensure_global_network;

// ---------------------------------------------------------------------------
// Event payload
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ServiceStatusEvent {
    pub service: ServiceId,
    pub status: ServiceStatus,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Emit "global://service-status" and update the status map in one call.
async fn emit_status(
    app: &AppHandle,
    state: &AppState,
    service: ServiceId,
    status: ServiceStatus,
) {
    {
        let mut map = state.global_stack.statuses.write().await;
        map.insert(service, status.clone());
    }
    app.emit(
        "global://service-status",
        ServiceStatusEvent { service, status },
    )
    .ok();
}

/// Read current status for a service (takes read lock).
async fn read_status(state: &AppState, service: ServiceId) -> ServiceStatus {
    let map = state.global_stack.statuses.read().await;
    map.get(&service)
        .cloned()
        .unwrap_or(ServiceStatus::Stopped)
}

/// Read current config for a service (takes read lock).
async fn read_config(state: &AppState, service: ServiceId) -> ServiceConfig {
    let map = state.global_stack.configs.read().await;
    map.get(&service)
        .cloned()
        .unwrap_or_else(|| ServiceConfig::default_for(service))
}

/// Write a config into the in-memory map.
async fn write_config(state: &AppState, service: ServiceId, config: ServiceConfig) {
    let mut map = state.global_stack.configs.write().await;
    map.insert(service, config);
}

// ---------------------------------------------------------------------------
// Internal start/stop logic (shared between toggle_service, global_on, global_off)
// ---------------------------------------------------------------------------

async fn start_service(app: AppHandle, state: &AppState, service: ServiceId) -> Result<(), AppError> {
    // Already starting/running — skip
    let current = read_status(state, service).await;
    if matches!(current, ServiceStatus::Running | ServiceStatus::Starting) {
        return Ok(());
    }

    emit_status(&app, state, service, ServiceStatus::Starting).await;

    let docker_guard = state.docker.read().await;
    let docker = match docker_guard.as_ref() {
        Some(d) => d.client.clone(),
        None => {
            emit_status(&app, state, service, ServiceStatus::Error("Docker not connected".to_string())).await;
            return Err(AppError::DockerUnavailable("Docker not connected".to_string()));
        }
    };
    drop(docker_guard);

    let config = read_config(state, service).await;
    let statuses_arc = state.global_stack.statuses.clone();
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        // Ensure shared network exists
        if let Err(e) = ensure_global_network(&docker).await {
            let mut map = statuses_arc.write().await;
            let err_status = ServiceStatus::Error(format!("Network error: {}", e));
            map.insert(service, err_status.clone());
            app_clone
                .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                .ok();
            return;
        }

        let container_name = service.container_name();

        // Inspect existing container
        let inspect_result = docker.inspect_container(container_name, None::<bollard::container::InspectContainerOptions>).await;

        match inspect_result {
            Ok(info) => {
                // Container exists — check its state
                let running = info
                    .state
                    .as_ref()
                    .and_then(|s| s.running)
                    .unwrap_or(false);

                if running {
                    // Already running — update status
                    let mut map = statuses_arc.write().await;
                    map.insert(service, ServiceStatus::Running);
                    app_clone
                        .emit("global://service-status", ServiceStatusEvent { service, status: ServiceStatus::Running })
                        .ok();
                    return;
                }

                // Stopped container — start it
                if let Err(e) = docker.start_container(container_name, None::<bollard::container::StartContainerOptions<String>>).await {
                    let mut map = statuses_arc.write().await;
                    let err_status = ServiceStatus::Error(e.to_string());
                    map.insert(service, err_status.clone());
                    app_clone
                        .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                        .ok();
                    return;
                }
            }
            Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => {
                // Container doesn't exist — create + start
                use bollard::container::{Config, CreateContainerOptions};
                use bollard::models::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum};

                let image = config.image_tag.clone();

                // Pull image before creating container (no-op if already local)
                {
                    use bollard::image::CreateImageOptions;
                    use futures_util::StreamExt;
                    let mut pull = docker.create_image(
                        Some(CreateImageOptions {
                            from_image: image.clone(),
                            ..Default::default()
                        }),
                        None,
                        None,
                    );
                    while let Some(msg) = pull.next().await {
                        if let Err(e) = msg {
                            let err_status = ServiceStatus::Error(
                                format!("Pull '{}' failed: {}", image, e),
                            );
                            let mut map = statuses_arc.write().await;
                            map.insert(service, err_status.clone());
                            app_clone
                                .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                                .ok();
                            return;
                        }
                    }
                }

                let internal_port = service.internal_port();
                let host_port = config.port.to_string();

                let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
                port_bindings.insert(
                    internal_port.to_string(),
                    Some(vec![PortBinding {
                        host_ip: Some("127.0.0.1".to_string()),
                        host_port: Some(host_port),
                    }]),
                );

                let host_config = HostConfig {
                    network_mode: Some("orcker-global".to_string()),
                    port_bindings: Some(port_bindings),
                    restart_policy: Some(RestartPolicy {
                        name: Some(RestartPolicyNameEnum::NO),
                        maximum_retry_count: None,
                    }),
                    ..Default::default()
                };

                let mut exposed_ports: HashMap<&str, HashMap<(), ()>> = HashMap::new();
                exposed_ports.insert(internal_port, HashMap::new());

                let container_config = Config {
                    image: Some(image.as_str()),
                    host_config: Some(host_config),
                    exposed_ports: Some(exposed_ports),
                    ..Default::default()
                };

                let create_result = docker
                    .create_container(
                        Some(CreateContainerOptions {
                            name: container_name,
                            platform: None,
                        }),
                        container_config,
                    )
                    .await;

                if let Err(e) = create_result {
                    let msg = e.to_string();
                    let err_status = if msg.contains("port is already allocated") {
                        ServiceStatus::Error(format!(
                            "Port {} is already in use — change the port in settings",
                            config.port
                        ))
                    } else {
                        ServiceStatus::Error(msg)
                    };
                    let mut map = statuses_arc.write().await;
                    map.insert(service, err_status.clone());
                    app_clone
                        .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                        .ok();
                    return;
                }

                if let Err(e) = docker
                    .start_container(container_name, None::<bollard::container::StartContainerOptions<String>>)
                    .await
                {
                    let msg = e.to_string();
                    let err_status = if msg.contains("port is already allocated") {
                        ServiceStatus::Error(format!(
                            "Port {} is already in use — change the port in settings",
                            config.port
                        ))
                    } else {
                        ServiceStatus::Error(msg)
                    };
                    let mut map = statuses_arc.write().await;
                    map.insert(service, err_status.clone());
                    app_clone
                        .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                        .ok();
                    return;
                }
            }
            Err(e) => {
                let mut map = statuses_arc.write().await;
                let err_status = ServiceStatus::Error(e.to_string());
                map.insert(service, err_status.clone());
                app_clone
                    .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                    .ok();
                return;
            }
        }

        // Poll until running or 30s timeout
        let poll = async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                match docker.inspect_container(container_name, None::<bollard::container::InspectContainerOptions>).await {
                    Ok(info) => {
                        let running = info
                            .state
                            .as_ref()
                            .and_then(|s| s.running)
                            .unwrap_or(false);
                        if running {
                            break Ok::<(), String>(());
                        }
                    }
                    Err(e) => break Err(e.to_string()),
                }
            }
        };

        let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(30));

        tokio::select! {
            result = poll => {
                match result {
                    Ok(_) => {
                        let mut map = statuses_arc.write().await;
                        map.insert(service, ServiceStatus::Running);
                        app_clone
                            .emit("global://service-status", ServiceStatusEvent { service, status: ServiceStatus::Running })
                            .ok();
                    }
                    Err(e) => {
                        let mut map = statuses_arc.write().await;
                        let err_status = ServiceStatus::Error(e);
                        map.insert(service, err_status.clone());
                        app_clone
                            .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                            .ok();
                    }
                }
            }
            _ = timeout => {
                let mut map = statuses_arc.write().await;
                let err_status = ServiceStatus::Error(
                    "Timeout: service did not start within 30s".to_string(),
                );
                map.insert(service, err_status.clone());
                app_clone
                    .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                    .ok();
            }
        }
    });

    Ok(())
}

async fn stop_service(app: AppHandle, state: &AppState, service: ServiceId) -> Result<(), AppError> {
    let current = read_status(state, service).await;
    if matches!(current, ServiceStatus::Stopped | ServiceStatus::Stopping) {
        return Ok(());
    }

    emit_status(&app, state, service, ServiceStatus::Stopping).await;

    let docker_guard = state.docker.read().await;
    let docker = match docker_guard.as_ref() {
        Some(d) => d.client.clone(),
        None => {
            emit_status(&app, state, service, ServiceStatus::Error("Docker not connected".to_string())).await;
            return Err(AppError::DockerUnavailable("Docker not connected".to_string()));
        }
    };
    drop(docker_guard);

    let statuses_arc = state.global_stack.statuses.clone();
    let app_clone = app.clone();
    let container_name = service.container_name();

    tauri::async_runtime::spawn(async move {
        // Stop container (ignore 404)
        let stop_result = docker
            .stop_container(container_name, None::<bollard::container::StopContainerOptions>)
            .await;

        if let Err(e) = &stop_result {
            if let bollard::errors::Error::DockerResponseServerError { status_code: 404, .. } = e {
                // Already gone — mark stopped
                let mut map = statuses_arc.write().await;
                map.insert(service, ServiceStatus::Stopped);
                app_clone
                    .emit("global://service-status", ServiceStatusEvent { service, status: ServiceStatus::Stopped })
                    .ok();
                return;
            }
        }

        // Remove container (force=true, ignore 404)
        use bollard::container::RemoveContainerOptions;
        let remove_result = docker
            .remove_container(
                container_name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;

        match remove_result {
            Ok(_) | Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => {
                let mut map = statuses_arc.write().await;
                map.insert(service, ServiceStatus::Stopped);
                app_clone
                    .emit("global://service-status", ServiceStatusEvent { service, status: ServiceStatus::Stopped })
                    .ok();
            }
            Err(e) => {
                let mut map = statuses_arc.write().await;
                let err_status = ServiceStatus::Error(e.to_string());
                map.insert(service, err_status.clone());
                app_clone
                    .emit("global://service-status", ServiceStatusEvent { service, status: err_status })
                    .ok();
            }
        }
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
pub async fn get_services_status(
    state: State<'_, AppState>,
) -> Result<HashMap<ServiceId, ServiceStatus>, AppError> {
    let map = state.global_stack.statuses.read().await;
    Ok(map.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_service(
    app: AppHandle,
    state: State<'_, AppState>,
    service: ServiceId,
) -> Result<(), AppError> {
    let current = read_status(&state, service).await;

    match current {
        ServiceStatus::Running | ServiceStatus::Starting => {
            stop_service(app, &state, service).await
        }
        ServiceStatus::Stopped | ServiceStatus::Error(_) => {
            start_service(app, &state, service).await
        }
        ServiceStatus::Stopping => {
            // Already stopping — no-op
            Ok(())
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn set_service_config(
    app: AppHandle,
    state: State<'_, AppState>,
    service: ServiceId,
    config: ServiceConfig,
) -> Result<bool, AppError> {
    // Persist to store
    let store = app
        .store("orcker-services.json")
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let service_key = format!("{:?}", service);
    store.set(
        service_key,
        serde_json::to_value(&config).map_err(|e| AppError::Internal(e.to_string()))?,
    );
    store.save().map_err(|e| AppError::Internal(e.to_string()))?;

    // Update in-memory config
    write_config(&state, service, config).await;

    // Return true if restart is required (service currently running)
    let current = read_status(&state, service).await;
    Ok(matches!(current, ServiceStatus::Running))
}

#[tauri::command]
#[specta::specta]
pub async fn global_on(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    for service in ServiceId::all() {
        let current = read_status(&state, service).await;
        if matches!(current, ServiceStatus::Stopped | ServiceStatus::Error(_)) {
            start_service(app.clone(), &state, service).await?;
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn global_off(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    for service in ServiceId::all() {
        let current = read_status(&state, service).await;
        if matches!(current, ServiceStatus::Running | ServiceStatus::Starting) {
            stop_service(app.clone(), &state, service).await?;
        }
    }
    Ok(())
}
