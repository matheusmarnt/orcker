#![allow(deprecated)]
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;

use crate::adapters::docker::networks::ensure_global_network;
use crate::core::error::AppError;
use crate::core::global_stack::{ServiceConfig, ServiceId, ServiceStatus};
use crate::core::state::AppState;

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
async fn emit_status(app: &AppHandle, state: &AppState, service: ServiceId, status: ServiceStatus) {
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
    map.get(&service).cloned().unwrap_or(ServiceStatus::Stopped)
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
// Health monitor — runs in background after a service reaches Running
// ---------------------------------------------------------------------------

async fn health_monitor(
    docker: bollard::Docker,
    service: ServiceId,
    statuses_arc: std::sync::Arc<tokio::sync::RwLock<HashMap<ServiceId, ServiceStatus>>>,
    app_handle: AppHandle,
) {
    use bollard::models::HealthStatusEnum;

    let container_name = service.container_name();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // Exit if service is no longer running or unhealthy (intentionally stopped)
        {
            let map = statuses_arc.read().await;
            let status = map.get(&service).cloned().unwrap_or(ServiceStatus::Stopped);
            if !matches!(status, ServiceStatus::Running | ServiceStatus::Unhealthy) {
                return;
            }
        }

        match docker
            .inspect_container(
                container_name,
                None::<bollard::container::InspectContainerOptions>,
            )
            .await
        {
            Ok(info) => {
                let running = info.state.as_ref().and_then(|s| s.running).unwrap_or(false);
                if !running {
                    let mut map = statuses_arc.write().await;
                    map.insert(service, ServiceStatus::Stopped);
                    drop(map);
                    app_handle
                        .emit(
                            "global://service-status",
                            ServiceStatusEvent {
                                service,
                                status: ServiceStatus::Stopped,
                            },
                        )
                        .ok();
                    return;
                }

                let health = info
                    .state
                    .as_ref()
                    .and_then(|s| s.health.as_ref())
                    .and_then(|h| h.status.as_ref())
                    .cloned();

                let new_status = match health {
                    Some(HealthStatusEnum::UNHEALTHY) => ServiceStatus::Unhealthy,
                    Some(HealthStatusEnum::HEALTHY) => ServiceStatus::Running,
                    _ => continue, // STARTING, NONE, EMPTY — no change yet
                };

                let mut map = statuses_arc.write().await;
                let current = map.get(&service).cloned().unwrap_or(ServiceStatus::Stopped);
                if std::mem::discriminant(&current) != std::mem::discriminant(&new_status) {
                    map.insert(service, new_status.clone());
                    drop(map);
                    app_handle
                        .emit(
                            "global://service-status",
                            ServiceStatusEvent {
                                service,
                                status: new_status,
                            },
                        )
                        .ok();
                }
            }
            Err(_) => {
                // Container gone unexpectedly
                let mut map = statuses_arc.write().await;
                map.insert(service, ServiceStatus::Stopped);
                drop(map);
                app_handle
                    .emit(
                        "global://service-status",
                        ServiceStatusEvent {
                            service,
                            status: ServiceStatus::Stopped,
                        },
                    )
                    .ok();
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal start/stop logic (shared between toggle_service, global_on, global_off)
// ---------------------------------------------------------------------------

async fn start_service(
    app: AppHandle,
    state: &AppState,
    service: ServiceId,
) -> Result<(), AppError> {
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
            emit_status(
                &app,
                state,
                service,
                ServiceStatus::Error("Docker not connected".to_string()),
            )
            .await;
            return Err(AppError::DockerUnavailable(
                "Docker not connected".to_string(),
            ));
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
                .emit(
                    "global://service-status",
                    ServiceStatusEvent {
                        service,
                        status: err_status,
                    },
                )
                .ok();
            return;
        }

        let container_name = service.container_name();

        // If container exists and is already running → emit Running and return early.
        // If it exists but is not running → remove it so we always recreate with current
        // config (applies env vars, port changes, image updates).
        match docker
            .inspect_container(
                container_name,
                None::<bollard::container::InspectContainerOptions>,
            )
            .await
        {
            Ok(info) => {
                let running = info.state.as_ref().and_then(|s| s.running).unwrap_or(false);

                if running {
                    let mut map = statuses_arc.write().await;
                    map.insert(service, ServiceStatus::Running);
                    app_clone
                        .emit(
                            "global://service-status",
                            ServiceStatusEvent {
                                service,
                                status: ServiceStatus::Running,
                            },
                        )
                        .ok();
                    return;
                }

                // Not running — remove to recreate fresh with current config
                use bollard::container::RemoveContainerOptions;
                docker
                    .remove_container(
                        container_name,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await
                    .ok();
            }
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                // Container doesn't exist — will create below
            }
            Err(e) => {
                let mut map = statuses_arc.write().await;
                let err_status = ServiceStatus::Error(e.to_string());
                map.insert(service, err_status.clone());
                app_clone
                    .emit(
                        "global://service-status",
                        ServiceStatusEvent {
                            service,
                            status: err_status,
                        },
                    )
                    .ok();
                return;
            }
        }

        // Clone docker for health monitor (before poll borrows it)
        let docker_for_health = docker.clone();

        // Pull image (no-op if already local)
        {
            use bollard::image::CreateImageOptions;
            use futures_util::StreamExt;
            let mut pull = docker.create_image(
                Some(CreateImageOptions {
                    from_image: config.image_tag.clone(),
                    ..Default::default()
                }),
                None,
                None,
            );
            while let Some(msg) = pull.next().await {
                if let Err(e) = msg {
                    let err_status =
                        ServiceStatus::Error(format!("Pull '{}' failed: {}", config.image_tag, e));
                    let mut map = statuses_arc.write().await;
                    map.insert(service, err_status.clone());
                    app_clone
                        .emit(
                            "global://service-status",
                            ServiceStatusEvent {
                                service,
                                status: err_status,
                            },
                        )
                        .ok();
                    return;
                }
            }
        }

        // Create container with full config (env vars, ports, network)
        {
            use bollard::container::{Config, CreateContainerOptions};
            use bollard::models::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum};

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

            // Service-specific env vars (e.g. POSTGRES_PASSWORD for postgres)
            let env_vars = service.default_env_vars();
            let env_refs: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();

            let container_config = Config {
                image: Some(config.image_tag.as_str()),
                host_config: Some(host_config),
                exposed_ports: Some(exposed_ports),
                env: if env_refs.is_empty() {
                    None
                } else {
                    Some(env_refs)
                },
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
                    .emit(
                        "global://service-status",
                        ServiceStatusEvent {
                            service,
                            status: err_status,
                        },
                    )
                    .ok();
                return;
            }
        }

        // Start container
        if let Err(e) = docker
            .start_container(
                container_name,
                None::<bollard::container::StartContainerOptions<String>>,
            )
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
                .emit(
                    "global://service-status",
                    ServiceStatusEvent {
                        service,
                        status: err_status,
                    },
                )
                .ok();
            return;
        }

        // Poll until running, crashed, or 30s timeout.
        // Fail fast: exit_code is Some when container exited — no need to wait full 30s.
        let poll = async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                match docker
                    .inspect_container(
                        container_name,
                        None::<bollard::container::InspectContainerOptions>,
                    )
                    .await
                {
                    Ok(info) => {
                        let state_ref = info.state.as_ref();
                        let running = state_ref.and_then(|s| s.running).unwrap_or(false);
                        if running {
                            break Ok::<(), String>(());
                        }
                        // exit_code is Some(n) once container has exited — fail fast
                        if let Some(code) = state_ref.and_then(|s| s.exit_code) {
                            break Err(format!("Container exited (code {})", code));
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
                        drop(map);
                        app_clone
                            .emit("global://service-status", ServiceStatusEvent { service, status: ServiceStatus::Running })
                            .ok();
                        tauri::async_runtime::spawn(health_monitor(
                            docker_for_health,
                            service,
                            statuses_arc.clone(),
                            app_clone.clone(),
                        ));
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

async fn stop_service(
    app: AppHandle,
    state: &AppState,
    service: ServiceId,
) -> Result<(), AppError> {
    let current = read_status(state, service).await;
    if matches!(current, ServiceStatus::Stopped | ServiceStatus::Stopping) {
        return Ok(());
    }

    emit_status(&app, state, service, ServiceStatus::Stopping).await;

    let docker_guard = state.docker.read().await;
    let docker = match docker_guard.as_ref() {
        Some(d) => d.client.clone(),
        None => {
            emit_status(
                &app,
                state,
                service,
                ServiceStatus::Error("Docker not connected".to_string()),
            )
            .await;
            return Err(AppError::DockerUnavailable(
                "Docker not connected".to_string(),
            ));
        }
    };
    drop(docker_guard);

    let statuses_arc = state.global_stack.statuses.clone();
    let app_clone = app.clone();
    let container_name = service.container_name();

    tauri::async_runtime::spawn(async move {
        // Stop container (ignore 404)
        let stop_result = docker
            .stop_container(
                container_name,
                None::<bollard::container::StopContainerOptions>,
            )
            .await;

        if let Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) = &stop_result
        {
            // Already gone — mark stopped
            let mut map = statuses_arc.write().await;
            map.insert(service, ServiceStatus::Stopped);
            app_clone
                .emit(
                    "global://service-status",
                    ServiceStatusEvent {
                        service,
                        status: ServiceStatus::Stopped,
                    },
                )
                .ok();
            return;
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
            Ok(_)
            | Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                let mut map = statuses_arc.write().await;
                map.insert(service, ServiceStatus::Stopped);
                app_clone
                    .emit(
                        "global://service-status",
                        ServiceStatusEvent {
                            service,
                            status: ServiceStatus::Stopped,
                        },
                    )
                    .ok();
            }
            Err(e) => {
                let mut map = statuses_arc.write().await;
                let err_status = ServiceStatus::Error(e.to_string());
                map.insert(service, err_status.clone());
                app_clone
                    .emit(
                        "global://service-status",
                        ServiceStatusEvent {
                            service,
                            status: err_status,
                        },
                    )
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
pub async fn get_service_configs(
    state: State<'_, AppState>,
) -> Result<HashMap<ServiceId, ServiceConfig>, AppError> {
    let map = state.global_stack.configs.read().await;
    Ok(map.clone())
}

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
        ServiceStatus::Running | ServiceStatus::Starting | ServiceStatus::Unhealthy => {
            stop_service(app, &state, service).await
        }
        ServiceStatus::Stopped | ServiceStatus::Error(_) => {
            start_service(app, &state, service).await
        }
        ServiceStatus::Stopping => Ok(()),
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
    store
        .save()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update in-memory config
    write_config(&state, service, config).await;

    // Return true if restart is required (service currently running)
    let current = read_status(&state, service).await;
    Ok(matches!(current, ServiceStatus::Running))
}

#[tauri::command]
#[specta::specta]
pub async fn global_on(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
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
pub async fn global_off(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    for service in ServiceId::all() {
        let current = read_status(&state, service).await;
        if matches!(
            current,
            ServiceStatus::Running | ServiceStatus::Starting | ServiceStatus::Unhealthy
        ) {
            stop_service(app.clone(), &state, service).await?;
        }
    }
    Ok(())
}
