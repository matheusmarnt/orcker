//! Short-TTL cache for the Docker environment snapshot.
//!
//! Detection connects to the Docker daemon and spawns `docker compose version`.
//! `orcker status`, the GUI poll and (later) the doctor all want the same
//! answer, and on a *stopped* engine each probe pays the connect timeout - so
//! the daemon holds the result for [`TTL`] rather than re-probing per request.
//!
//! The TTL lives here, not in `orcker-engine`: the crate reports what it finds,
//! the daemon decides how stale an answer may be.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use orcker_ipc::DockerStatus;

/// How long a snapshot is served before Docker is probed again.
///
/// Short enough that starting Docker shows up in the next `orcker status`
/// without feeling stuck, long enough that a GUI poll loop does not hammer a
/// dead socket.
pub const TTL: Duration = Duration::from_secs(10);

/// Caches the last [`DockerStatus`] with a wall-clock expiry.
#[derive(Default)]
pub struct EngineStatusCache {
    inner: Mutex<Option<(Instant, DockerStatus)>>,
}

impl EngineStatusCache {
    /// A fresh, empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached snapshot while it is younger than [`TTL`], otherwise a new
    /// probe of the real Docker environment.
    ///
    /// Two callers arriving on an expired entry both probe; the loser's result
    /// simply overwrites the winner's. A lock held across the probe would be
    /// worse - it would serialize every client behind one connect timeout.
    pub async fn get(&self) -> DockerStatus {
        if let Some(fresh) = self.fresh() {
            return fresh;
        }
        let status = orcker_engine::io::detect_from_env().await;
        *self.lock() = Some((Instant::now(), status.clone()));
        status
    }

    fn fresh(&self) -> Option<DockerStatus> {
        let guard = self.lock();
        let (at, status) = guard.as_ref()?;
        (at.elapsed() < TTL).then(|| status.clone())
    }

    /// Lock helper that recovers from a poisoned mutex rather than panicking
    /// (the crate forbids `unwrap`/`expect`). A poisoned cache is harmless: the
    /// worst case is one stale-but-valid snapshot.
    fn lock(&self) -> MutexGuard<'_, Option<(Instant, DockerStatus)>> {
        match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use orcker_ipc::{ComposeStatus, SocketKind};

    fn sample() -> DockerStatus {
        DockerStatus {
            socket: SocketKind::Unsupported,
            reachable: true,
            engine_version: Some("27.3.1".to_owned()),
            compose: ComposeStatus::Missing,
            problems: vec![],
        }
    }

    #[test]
    fn a_fresh_entry_is_served_without_probing() {
        let cache = EngineStatusCache::new();
        *cache.lock() = Some((Instant::now(), sample()));
        assert_eq!(cache.fresh(), Some(sample()));
    }

    #[test]
    fn an_expired_entry_is_not_served() {
        let cache = EngineStatusCache::new();
        let stale = Instant::now()
            .checked_sub(TTL + Duration::from_secs(1))
            .expect("the test clock is well past the TTL");
        *cache.lock() = Some((stale, sample()));
        assert_eq!(cache.fresh(), None);
    }

    #[test]
    fn an_empty_cache_serves_nothing() {
        assert_eq!(EngineStatusCache::new().fresh(), None);
    }
}
