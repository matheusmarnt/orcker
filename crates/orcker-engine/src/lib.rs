//! Docker environment detection for Orcker.
//!
//! The crate answers one question before any lifecycle work exists: *is Docker
//! usable here?* It reports the resolved endpoint, whether the engine answers,
//! the engine and `docker compose` versions, and every problem with the hint
//! that fixes it.
//!
//! ## Layers
//!
//! [`pure`] is sync, runtime-free and does no I/O: it resolves the endpoint
//! from injected environment values, parses and compares versions, and
//! assembles the [`orcker_ipc::DockerStatus`] the daemon sends to clients.
//! [`traits`] declares the two side effects ([`traits::EngineApi`],
//! [`traits::ComposeCli`]); `io` holds the one real implementation of each.
//! Tests inject fakes and never touch a real Docker daemon.
//!
//! The wire model lives in `orcker-ipc` (as `Diagnosis` does for
//! `orcker-doctor`), so clients read a `DockerStatus` without pulling
//! `bollard` into their dependency graph.

#![forbid(unsafe_code)]

mod error;
pub mod io;
mod probe;
pub mod pure;
pub mod traits;

pub use error::EngineError;
pub use io::{BollardEngine, DockerComposeCli};
pub use probe::detect;
pub use pure::{MIN_COMPOSE_VERSION, MIN_ENGINE_VERSION};
