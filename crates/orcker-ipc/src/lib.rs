//! IPC protocol, framing, and codec used between `orckerd` and its
//! clients (the `orcker` CLI and the Tauri GUI).
//!
//! The default build is pure: no sockets, no async, no I/O. Enable
//! the `transport` feature to pull in `tokio`-based async helpers
//! shared by the daemon and the CLI.
//!
//! ## Wire format
//!
//! Every message is a length-prefixed JSON frame: a 4-byte big-endian
//! `u32` length followed by `length` bytes of UTF-8 JSON. The frame
//! codec is byte-agnostic; the JSON shape is pinned in
//! `tests/wire_stability.rs` and the framing edges are pinned in
//! `tests/frame_codec.rs`.
//!
//! ## Version compatibility
//!
//! [`PROTOCOL_VERSION`] travels on the wire via a connect-time handshake: a
//! client sends [`Request::Hello`] with its own version, and the daemon
//! replies [`Response::Welcome`] (compatible) or [`Response::Error`] with
//! [`ErrorCode::VersionMismatch`] (not). The same typed error also covers a
//! request the daemon cannot decode at all - an unrecognized `type` tag,
//! e.g. from a client speaking a protocol this daemon build predates -
//! instead of the connection silently closing, which is what happened
//! before this handshake existed.

mod create;
mod engine;
mod error;
mod frame;
mod message;
mod request;
mod response;
mod status;
mod update;

#[cfg(feature = "transport")]
mod transport;

/// The current IPC protocol version. Bump on any breaking change.
///
/// `2` marks the fork's one authorized contract reset: SPEC-0002 removed the
/// native-runtime requests rather than deprecating them additively, which is
/// safe only because no released daemon speaks version `1` under this name.
/// SPEC-0034 put this constant on the wire: [`Request::Hello`] carries it on
/// connect and the daemon compares it against its own copy.
pub const PROTOCOL_VERSION: u32 = 2;

pub use create::{JobId, JobState};
pub use engine::{ComposeStatus, DockerStatus, EngineProblem, EngineProblemCode, SocketKind};
pub use error::{FrameError, IpcError, IpcErrorKind};
pub use frame::{encode_frame, FrameDecoder, DEFAULT_MAX_FRAME};
pub use message::{decode_message, encode_message};
pub use request::Request;
pub use response::{
    ErrorCode, ProjectEntry, ProxyEntry, ProxyRuleEntry, Response, RouteRuleEntry, SiteEntry,
};
pub use status::{
    BrowserTrust, CaStatus, CloudflaredSource, CloudflaredStatus, Diagnosis, DiagnosisCode,
    DomainShadow, FixReport, FixResult, MailAttachment, MailDetail, MailHeader, MailStatus,
    MailSummary, NamedTunnelMeta, PortRedirectTargets, PortStatus, Severity, SiteCounts,
    SiteHostname, StatusReport, ToolStatus, TunnelInfo, TunnelKind, TunnelRunState, UnboundWeb,
};
pub use update::{Channel, StagedArtifact, UpdateSource};

/// Re-exports of the shared types that travel on the wire. Consumers
/// that need only the IPC surface should `use orcker_ipc::types::*;`
/// instead of depending on `orcker-core` directly.
pub mod types {
    pub use orcker_core::{PhpVersion, Site, SiteKind};
}

#[cfg(feature = "transport")]
pub use transport::{read_frame, read_message, write_message};

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_pinned() {
        assert_eq!(PROTOCOL_VERSION, 2);
    }
}
