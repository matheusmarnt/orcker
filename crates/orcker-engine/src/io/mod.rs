//! The one real implementation of each trait in [`crate::traits`].
//!
//! Everything `bollard` and everything that spawns a process is confined here,
//! so `orcker-engine`'s policy stays testable with in-memory fakes and no other
//! crate needs `bollard` in its dependency graph.

use async_trait::async_trait;
use bollard::{Docker, API_DEFAULT_VERSION};
use orcker_ipc::SocketKind;
use tokio::process::Command;

use crate::error::EngineError;
use crate::traits::{ComposeCli, EngineApi};

/// Seconds bollard waits on a connect/read before giving up.
///
/// Short on purpose: `orcker status` runs this on a possibly dead engine, and a
/// user staring at a prompt would rather hear "not running" than wait.
const CONNECT_TIMEOUT_SECS: u64 = 4;

/// Probe the Docker environment using the process environment and the real
/// implementations.
///
/// The env reads (`DOCKER_HOST`, `HOME`) live here rather than in the daemon so
/// the daemon stays orchestration: it calls this and caches the answer.
pub async fn detect_from_env() -> orcker_ipc::DockerStatus {
    let candidates = crate::pure::resolve_socket(
        std::env::var("DOCKER_HOST").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
        crate::pure::HostOs::current(),
    );
    crate::detect(&BollardEngine, &DockerComposeCli, &candidates).await
}

/// The Docker Engine API, over bollard.
#[derive(Debug, Default, Clone, Copy)]
pub struct BollardEngine;

#[async_trait]
impl EngineApi for BollardEngine {
    async fn version(&self, socket: &SocketKind) -> Result<String, EngineError> {
        let docker = connect(socket)?;
        let reported = docker
            .version()
            .await
            .map_err(|e| EngineError::Unreachable {
                endpoint: crate::pure::endpoint_label(socket),
                source_message: e.to_string(),
            })?;
        reported.version.ok_or_else(|| EngineError::Unreachable {
            endpoint: crate::pure::endpoint_label(socket),
            source_message: "the engine answered /version without a version field".to_owned(),
        })
    }
}

fn connect(socket: &SocketKind) -> Result<Docker, EngineError> {
    let client = match socket {
        SocketKind::Unix { path } => {
            Docker::connect_with_socket(path, CONNECT_TIMEOUT_SECS, API_DEFAULT_VERSION)
        }
        SocketKind::Tcp { endpoint } => {
            Docker::connect_with_http(endpoint, CONNECT_TIMEOUT_SECS, API_DEFAULT_VERSION)
        }
        _ => return Err(EngineError::Unsupported),
    };
    client.map_err(|e| EngineError::Unreachable {
        endpoint: crate::pure::endpoint_label(socket),
        source_message: e.to_string(),
    })
}

/// The compose plugin, over the `docker` binary on `PATH`.
#[derive(Debug, Default, Clone, Copy)]
pub struct DockerComposeCli;

#[async_trait]
impl ComposeCli for DockerComposeCli {
    async fn version_output(&self) -> Result<String, EngineError> {
        let out = Command::new("docker")
            .args(["compose", "version", "--format", "json"])
            .output()
            .await
            .map_err(|e| EngineError::ComposeUnavailable(e.to_string()))?;
        if !out.status.success() {
            return Err(EngineError::ComposeUnavailable(
                String::from_utf8_lossy(&out.stderr).trim().to_owned(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
