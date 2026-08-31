//! The edge that drives the two side effects and hands the result to [`crate::pure`].
//!
//! Async, because both probes are I/O. It owns no policy: which endpoints to
//! try comes from [`crate::pure::resolve_socket`], and every verdict comes from
//! [`crate::pure::assemble`].

use orcker_ipc::{DockerStatus, SocketKind};

use crate::pure::{assemble, ProbeOutcome};
use crate::traits::{ComposeCli, EngineApi};

/// Probe `candidates` in order and assemble the snapshot.
///
/// The first endpoint that answers wins. When none answers, the snapshot
/// reports the first candidate (the one the user most likely meant) as
/// unreachable, so the hint names a concrete path rather than a list.
pub async fn detect(
    engine: &dyn EngineApi,
    compose: &dyn ComposeCli,
    candidates: &[SocketKind],
) -> DockerStatus {
    let compose_output = compose.version_output().await.ok();
    let fallback = candidates
        .first()
        .cloned()
        .unwrap_or(SocketKind::Unsupported);

    for candidate in candidates {
        if let Ok(version) = engine.version(candidate).await {
            return assemble(
                candidate.clone(),
                &ProbeOutcome {
                    engine_version: Some(version),
                    compose_output,
                },
            );
        }
    }

    assemble(
        fallback,
        &ProbeOutcome {
            engine_version: None,
            compose_output,
        },
    )
}
