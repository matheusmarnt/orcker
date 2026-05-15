#![allow(deprecated)]
use bollard::image::{ListImagesOptions, PruneImagesOptions, RemoveImageOptions};
use bollard::query_parameters::CreateImageOptionsBuilder;
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct ImageInfo {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
}

pub async fn list_images(docker: &Docker) -> Result<Vec<ImageInfo>, AppError> {
    let images = docker
        .list_images(Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        }))
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;
    Ok(images
        .into_iter()
        .map(|i| ImageInfo {
            // bollard 0.19: id, repo_tags, size are non-Option on ImageSummary
            id: i.id,
            tags: i.repo_tags,
            size: i.size,
        })
        .collect())
}

pub async fn pull_image(docker: &Docker, image: &str, tag: &str) -> Result<(), AppError> {
    // bollard 0.19: use query_parameters builder API
    let opts = CreateImageOptionsBuilder::default()
        .from_image(image)
        .tag(tag)
        .build();
    let mut stream = docker.create_image(Some(opts), None, None);
    // MUST drain the stream fully (Phase 1 lesson: create_image() stream must be consumed)
    while let Some(msg) = stream.next().await {
        msg.map_err(|e| AppError::DockerApi(e.to_string()))?;
    }
    Ok(())
}

pub async fn remove_image(docker: &Docker, image_id: &str) -> Result<(), AppError> {
    docker
        .remove_image(
            image_id,
            Some(RemoveImageOptions {
                force: false,
                noprune: false,
            }),
            None,
        )
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;
    Ok(())
}

pub async fn prune_images(docker: &Docker) -> Result<u64, AppError> {
    let resp = docker
        .prune_images(None::<PruneImagesOptions<String>>)
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;
    // space_reclaimed is Option<i64> in bollard PruneImagesResponse
    let reclaimed = resp.space_reclaimed.unwrap_or(0).max(0) as u64;
    Ok(reclaimed)
}
