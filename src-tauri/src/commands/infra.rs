use tauri::State;

use crate::adapters::docker::images::{self, ImageInfo};
use crate::adapters::docker::volumes::{self, VolumeInfo};
use crate::core::error::AppError;
use crate::core::state::AppState;

/// List all Docker volumes with name, driver, mountpoint, and size in MB.
#[tauri::command]
#[specta::specta]
pub async fn list_volumes(app_state: State<'_, AppState>) -> Result<Vec<VolumeInfo>, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    volumes::list_volumes(&docker.client).await
}

/// Prune dangling (unused) volumes. Returns reclaimed space in bytes.
#[tauri::command]
#[specta::specta]
pub async fn prune_volumes(app_state: State<'_, AppState>) -> Result<u64, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    volumes::prune_volumes(&docker.client).await
}

/// List all local Docker images with tags and size.
#[tauri::command]
#[specta::specta]
pub async fn list_images(app_state: State<'_, AppState>) -> Result<Vec<ImageInfo>, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    images::list_images(&docker.client).await
}

/// Pull a Docker image by name and tag. Drains the stream fully before returning.
#[tauri::command]
#[specta::specta]
pub async fn pull_image(
    image: String,
    tag: String,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    images::pull_image(&docker.client, &image, &tag).await
}

/// Remove a Docker image by ID.
#[tauri::command]
#[specta::specta]
pub async fn remove_image(
    image_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    images::remove_image(&docker.client, &image_id).await
}

/// Prune dangling (unused) images. Returns reclaimed space in bytes.
#[tauri::command]
#[specta::specta]
pub async fn prune_images(app_state: State<'_, AppState>) -> Result<u64, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    images::prune_images(&docker.client).await
}
