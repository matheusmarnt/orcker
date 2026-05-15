use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::adapters::docker::log_stream::{docker_log_stream, file_tail_stream, LogLine, LogSource};
use crate::core::error::AppError;
use crate::core::projects::ProjectsState;
use crate::core::state::AppState;

/// Start streaming logs for a project: docker container stdout/stderr + laravel.log file tail.
///
/// Both streams share a single CancellationToken keyed "logs:{project_id}" in ProjectsState.
/// Call stop_log_stream to cancel all streams for this project.
#[tauri::command]
#[specta::specta]
pub async fn start_log_stream(
    project_id: String,
    project_path: String,
    app_container: String,
    on_line: Channel<LogLine>,
    app_state: State<'_, AppState>,
    projects_state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    // Acquire Docker client
    let docker_guard = app_state.docker.read().await;
    let adapter = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    let docker = adapter.client.clone();
    drop(docker_guard);

    // One shared token per project — cancels ALL log streams for this project
    let token = CancellationToken::new();
    let key = format!("logs:{}", project_id);
    {
        let mut tokens = projects_state.cancel_tokens.write().await;
        // Cancel existing log stream if any
        if let Some(old_token) = tokens.get(&key) {
            old_token.cancel();
        }
        tokens.insert(key.clone(), token.clone());
    }

    let laravel_log_path = format!("{}/storage/logs/laravel.log", project_path);

    // Spawn docker log stream (stdout/stderr of the app container)
    let docker_clone = docker.clone();
    let token_docker = token.clone();
    let on_line_docker = on_line.clone();
    let app_container_clone = app_container.clone();
    tokio::spawn(async move {
        let _ = docker_log_stream(
            &docker_clone,
            &app_container_clone,
            LogSource::Docker,
            on_line_docker,
            token_docker,
        )
        .await;
    });

    // Spawn laravel.log file tail (1s polling, seeks to EOF on open)
    let token_file = token.clone();
    let on_line_file = on_line.clone();
    tokio::spawn(async move {
        let _ = file_tail_stream(
            laravel_log_path,
            LogSource::Laravel,
            on_line_file,
            token_file,
        )
        .await;
    });

    // Block until parent token is cancelled (stop_log_stream cancels it)
    token.cancelled().await;

    // Clean up token entry after cancellation
    {
        let mut tokens = projects_state.cancel_tokens.write().await;
        tokens.remove(&key);
    }

    Ok(())
}

/// Cancel all log streams for the given project.
#[tauri::command]
#[specta::specta]
pub async fn stop_log_stream(
    project_id: String,
    projects_state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    let key = format!("logs:{}", project_id);
    let tokens = projects_state.cancel_tokens.read().await;
    if let Some(token) = tokens.get(&key) {
        token.cancel();
    }
    Ok(())
}
