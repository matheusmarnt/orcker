//! Typed errors for the crate's I/O edge.

use thiserror::Error;

/// Something went wrong while probing the Docker environment.
///
/// Detection never fails as a whole: the daemon turns these into
/// [`orcker_ipc::EngineProblem`] entries so `orcker status` still reports.
#[derive(Debug, Error)]
pub enum EngineError {
    /// The Docker daemon did not answer on the resolved endpoint.
    #[error("docker engine unreachable on {endpoint}: {source_message}")]
    Unreachable {
        /// The endpoint that was tried.
        endpoint: String,
        /// The underlying transport failure, already rendered.
        source_message: String,
    },
    /// The `docker` binary is absent, or `docker compose version` failed.
    #[error("`docker compose version` failed: {0}")]
    ComposeUnavailable(String),
    /// This platform has no supported Docker endpoint.
    #[error("no supported docker endpoint on this platform")]
    Unsupported,
}
