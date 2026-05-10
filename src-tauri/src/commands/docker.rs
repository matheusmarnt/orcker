use crate::core::{error::AppError, state::AppState};

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
pub async fn list_containers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, AppError> {
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
                .iter()
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect())
        }
    }
}
