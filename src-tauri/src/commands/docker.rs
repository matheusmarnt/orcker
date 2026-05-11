use crate::core::{error::AppError, state::AppState};

/// Typed summary of a container — specta-exportable (no serde_json::Value)
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub status: String,
    pub state: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_docker_version(
    state: tauri::State<'_, AppState>,
) -> Result<String, AppError> {
    let guard = state.docker.read().await;
    match guard.as_ref() {
        None => Err(AppError::DockerUnavailable("Not connected".to_string())),
        Some(adapter) => {
            let version = adapter.client.version().await.map_err(AppError::from)?;
            Ok(version.version.unwrap_or_default())
        }
    }
}

#[tauri::command]
#[specta::specta]
#[allow(deprecated)]
pub async fn list_containers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ContainerSummary>, AppError> {
    let guard = state.docker.read().await;
    match guard.as_ref() {
        None => Err(AppError::DockerUnavailable("Not connected".to_string())),
        Some(adapter) => {
            let containers = adapter
                .client
                .list_containers(Some(bollard::container::ListContainersOptions::<String> {
                    all: true,
                    ..Default::default()
                }))
                .await
                .map_err(AppError::from)?;
            Ok(containers
                .into_iter()
                .map(|c| ContainerSummary {
                    id: c.id.unwrap_or_default(),
                    names: c.names.unwrap_or_default(),
                    image: c.image.unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                    state: c.state.map(|s| s.to_string()).unwrap_or_default(),
                })
                .collect())
        }
    }
}
