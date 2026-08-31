//! The crate's two side effects, declared so tests can replace them.
//!
//! One real implementation of each lives in `io`; unit and integration tests
//! inject in-memory fakes and never reach a Docker daemon.

use async_trait::async_trait;
use orcker_ipc::SocketKind;

use crate::error::EngineError;

/// Talks to the Docker Engine API.
#[async_trait]
pub trait EngineApi: Send + Sync {
    /// Ping `socket` and return the engine's reported version.
    ///
    /// One call rather than a separate ping: `/version` answers only when the
    /// daemon is up, so a successful read *is* the ping.
    async fn version(&self, socket: &SocketKind) -> Result<String, EngineError>;
}

/// Runs the `docker compose` plugin.
#[async_trait]
pub trait ComposeCli: Send + Sync {
    /// Run `docker compose version --format json` and return its stdout.
    async fn version_output(&self) -> Result<String, EngineError>;
}
