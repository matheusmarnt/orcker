#![allow(deprecated)]

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::docker::images::{self, ImageInfo};
use crate::adapters::docker::volumes::{self, VolumeInfo};
use crate::core::error::AppError;
use crate::core::projects::ProjectsState;
use crate::core::state::AppState;

/// Aggregated CPU% and memory usage (MB) for all containers in a project.
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct ResourceStats {
    pub cpu_percent: f64,
    pub mem_mb: f64,
}

/// Collect stats for all running containers belonging to a project (identified by
/// `com.docker.compose.project.working_dir` label == project path).
#[tauri::command]
#[specta::specta]
pub async fn get_resource_stats(
    app_state: State<'_, AppState>,
    projects_state: State<'_, ProjectsState>,
    project_id: String,
) -> Result<ResourceStats, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;

    // Resolve project path from ProjectsState
    let project_path = {
        let projects = projects_state.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| AppError::Internal(format!("Project {} not found", project_id)))?
    };

    // List containers with the project working_dir label
    let label_filter = format!("com.docker.compose.project.working_dir={}", project_path);
    let list_opts = bollard::container::ListContainersOptions::<String> {
        all: false,
        filters: std::collections::HashMap::from([("label".to_string(), vec![label_filter])]),
        ..Default::default()
    };
    let containers = docker
        .client
        .list_containers(Some(list_opts))
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    let mut total_cpu = 0.0f64;
    let mut total_mem_mb = 0.0f64;

    for c in containers {
        let id = match c.id {
            Some(id) => id,
            None => continue,
        };
        let mut stream = docker.client.stats(
            &id,
            Some(bollard::container::StatsOptions {
                stream: false,
                one_shot: true,
            }),
        );
        if let Some(Ok(stats)) = stream.next().await {
            // CPU calculation using bollard 0.19 Option<ContainerCpuStats> fields
            if let (Some(cpu), Some(precpu)) = (&stats.cpu_stats, &stats.precpu_stats) {
                let cpu_total = cpu
                    .cpu_usage
                    .as_ref()
                    .and_then(|u| u.total_usage)
                    .unwrap_or(0);
                let precpu_total = precpu
                    .cpu_usage
                    .as_ref()
                    .and_then(|u| u.total_usage)
                    .unwrap_or(0);
                let cpu_delta = cpu_total.saturating_sub(precpu_total) as f64;

                let sys = cpu.system_cpu_usage.unwrap_or(0);
                let presys = precpu.system_cpu_usage.unwrap_or(0);
                let system_delta = sys.saturating_sub(presys) as f64;

                let num_cpus = cpu.online_cpus.unwrap_or(1) as f64;

                if system_delta > 0.0 {
                    total_cpu += (cpu_delta / system_delta) * num_cpus * 100.0;
                }
            }

            // Memory usage
            if let Some(mem) = &stats.memory_stats {
                if let Some(usage) = mem.usage {
                    // In bollard 0.19, memory_stats.stats is a HashMap<String, u64>
                    let cache = mem
                        .stats
                        .as_ref()
                        .and_then(|s| s.get("cache").copied())
                        .unwrap_or(0);
                    let used = usage.saturating_sub(cache);
                    total_mem_mb += used as f64 / (1024.0 * 1024.0);
                }
            }
        }
    }

    Ok(ResourceStats {
        cpu_percent: (total_cpu * 100.0).round() / 100.0,
        mem_mb: (total_mem_mb * 100.0).round() / 100.0,
    })
}

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
pub async fn prune_volumes(app_state: State<'_, AppState>) -> Result<f64, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    volumes::prune_volumes(&docker.client).await.map(|v| v as f64)
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
pub async fn prune_images(app_state: State<'_, AppState>) -> Result<f64, AppError> {
    let guard = app_state.docker.read().await;
    let docker = guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    images::prune_images(&docker.client).await.map(|v| v as f64)
}
