#![allow(deprecated)]
use bollard::volume::{ListVolumesOptions, PruneVolumesOptions};
use bollard::Docker;
use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    /// Size in MB; None if Docker daemon has not computed usage data (size == -1).
    pub size_mb: Option<i64>,
}

pub async fn list_volumes(docker: &Docker) -> Result<Vec<VolumeInfo>, AppError> {
    let resp = docker
        .list_volumes(None::<ListVolumesOptions<String>>)
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;
    // bollard 0.19: volumes is Vec<_> (non-optional)
    let volumes = resp
        .volumes
        .into_iter()
        .flatten()
        .map(|v| {
            let size_mb = v.usage_data.and_then(|u| {
                if u.size >= 0 {
                    Some(u.size / 1_048_576)
                } else {
                    None
                }
            });
            VolumeInfo {
                name: v.name,
                driver: v.driver,
                mountpoint: v.mountpoint,
                size_mb,
            }
        })
        .collect();
    Ok(volumes)
}

pub async fn prune_volumes(docker: &Docker) -> Result<u64, AppError> {
    let resp = docker
        .prune_volumes(None::<PruneVolumesOptions<String>>)
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;
    // space_reclaimed is i64 in bollard 0.19; cast to u64 safely
    let reclaimed = resp.space_reclaimed.max(0) as u64;
    Ok(reclaimed)
}
