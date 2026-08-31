//! Docker environment payload types (`orcker status`, `Request::EngineStatus`).
//!
//! These travel inside [`crate::Response::EngineStatus`] and are assembled by
//! `orcker-engine`'s pure layer, the same way `orcker-doctor` assembles
//! [`crate::Diagnosis`] values defined here. Keeping the model in this crate is
//! what lets the CLI and the GUI read it without ever pulling `bollard`.
//!
//! As with the rest of the crate this is a published contract: add
//! fields/variants additively, never rename, and let `rename_all` handle
//! casing. `tests/wire_stability.rs` pins the byte-exact shape.
//!
//! ## Versions cross as strings
//!
//! Docker and compose do not report strict semver (`v2.29.7`,
//! `27.4.0-rc.1`), and the supported minimums are two-component (`24.0`,
//! `2.20`). Parsing and ordering belong to `orcker_engine::pure::Version`; the
//! wire carries the rendered `major.minor.patch` string so this crate needs no
//! version dependency.

use serde::{Deserialize, Serialize};

/// The Docker endpoint the daemon resolved and probed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SocketKind {
    /// A unix socket, either from `DOCKER_HOST` or the platform default.
    Unix {
        /// Absolute path to the socket.
        path: String,
    },
    /// A `tcp://host:port` endpoint taken from `DOCKER_HOST`.
    Tcp {
        /// The endpoint as `DOCKER_HOST` spelled it.
        endpoint: String,
    },
    /// The platform has no supported default endpoint and `DOCKER_HOST` is
    /// unset (Windows, which compiles against the unsupported stub).
    Unsupported,
}

/// Verdict on the `docker compose` plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ComposeStatus {
    /// The plugin is installed and new enough.
    Found {
        /// The reported version.
        version: String,
    },
    /// `docker compose version` did not run, or reported nothing usable.
    Missing,
    /// The plugin is installed but older than the supported minimum.
    TooOld {
        /// The reported version.
        found: String,
        /// The oldest version Orcker supports.
        min: String,
    },
}

/// Machine-readable identifier for an [`EngineProblem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EngineProblemCode {
    /// The Docker daemon did not answer on the resolved endpoint.
    EngineUnreachable,
    /// The Docker daemon answered but is older than the supported minimum.
    EngineTooOld,
    /// The `docker compose` plugin is not installed.
    ComposeMissing,
    /// The `docker compose` plugin is older than the supported minimum.
    ComposeTooOld,
    /// This platform has no supported Docker endpoint.
    PlatformUnsupported,
}

/// One thing wrong with the Docker environment, with the action that fixes it.
///
/// Every problem carries a `hint` (NFR-08): status reports, it never leaves the
/// user guessing what to type next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineProblem {
    /// Stable identifier, for clients that branch on the problem.
    pub code: EngineProblemCode,
    /// What is wrong, in one sentence.
    pub message: String,
    /// What the user should do about it.
    pub hint: String,
}

/// A read-only snapshot of the Docker environment, returned for
/// [`crate::Request::EngineStatus`].
///
/// `problems` is empty exactly when Docker is usable: reachable, new enough,
/// and with a compose plugin at or above the minimum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerStatus {
    /// The endpoint that was probed.
    pub socket: SocketKind,
    /// Whether the Docker daemon answered a ping on that endpoint.
    pub reachable: bool,
    /// Engine version as reported by `/version`, `None` when unreachable.
    pub engine_version: Option<String>,
    /// Verdict on the `docker compose` plugin.
    pub compose: ComposeStatus,
    /// Everything blocking use of Docker, each with an actionable hint.
    pub problems: Vec<EngineProblem>,
}
