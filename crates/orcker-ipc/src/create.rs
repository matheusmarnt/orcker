//! Long-running-job identity and progress types.
//!
//! Work that takes far longer than a single request/response round-trip (a
//! streamed tool install, say) starts a background **job** on the daemon and
//! returns immediately with a [`crate::Response::JobStarted`]. The client then
//! polls [`crate::Request::JobStatus`] for the streamed log + phase until the
//! job reaches a terminal [`JobState`].
//!
//! The site-creation payloads that shared this module went with SPEC-0002's
//! native runtime; site creation returns over containers under PRD FR-020.

use serde::{Deserialize, Serialize};

/// Opaque identifier for a long-running daemon job. Allocated by the daemon and
/// echoed back by the client on every [`crate::Request::JobStatus`] poll.
pub type JobId = String;

/// Lifecycle state of a long-running job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Still working.
    Running,
    /// Finished successfully.
    Succeeded,
    /// Finished with an error (see [`crate::Response::JobProgress::error`]).
    Failed,
    /// Cancelled by the client.
    Cancelled,
}
