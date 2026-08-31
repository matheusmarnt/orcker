//! Where to look for the Docker daemon, decided from injected values.
//!
//! Pure on purpose: the caller reads `DOCKER_HOST` and `HOME` and passes them
//! in, then walks the returned candidates in order and keeps the first that
//! answers. Nothing here touches the environment or the filesystem, so the
//! whole resolution order is table-testable on any OS.

use orcker_ipc::SocketKind;

/// The host this build targets. Mirrors the `linux` / `macos` / `unsupported`
/// split in `orcker-platform`; the caller passes [`HostOs::current`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOs {
    /// Linux, where the daemon socket is the distro default.
    Linux,
    /// macOS, where Docker Desktop also exposes a per-user socket.
    MacOs,
    /// Anything else (Windows), which has no supported endpoint here.
    Unsupported,
}

impl HostOs {
    /// The OS this binary was compiled for.
    #[must_use]
    pub const fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self::Linux
        }
        #[cfg(target_os = "macos")]
        {
            Self::MacOs
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Self::Unsupported
        }
    }
}

/// The platform default unix socket, on every OS that has one.
pub const DEFAULT_UNIX_SOCKET: &str = "/var/run/docker.sock";

/// Docker Desktop's per-user socket, relative to `$HOME`.
pub const DESKTOP_USER_SOCKET: &str = ".docker/run/docker.sock";

/// Endpoints to try, most preferred first.
///
/// `DOCKER_HOST` wins outright when set: the user named an endpoint, so a
/// silent fallback to a different daemon would be worse than failing. Otherwise
/// the platform defaults apply, and on macOS the Docker Desktop user socket is
/// appended for the case where the default socket link is turned off.
#[must_use]
pub fn resolve_socket(
    docker_host: Option<&str>,
    home: Option<&str>,
    os: HostOs,
) -> Vec<SocketKind> {
    if let Some(raw) = docker_host.map(str::trim).filter(|s| !s.is_empty()) {
        return vec![from_docker_host(raw)];
    }
    match os {
        HostOs::Linux => vec![unix(DEFAULT_UNIX_SOCKET)],
        HostOs::MacOs => {
            let mut candidates = vec![unix(DEFAULT_UNIX_SOCKET)];
            if let Some(home) = home
                .map(|h| h.trim_end_matches('/'))
                .filter(|h| !h.is_empty())
            {
                candidates.push(unix(&format!("{home}/{DESKTOP_USER_SOCKET}")));
            }
            candidates
        }
        HostOs::Unsupported => vec![SocketKind::Unsupported],
    }
}

/// Read one `DOCKER_HOST` value. Anything Orcker cannot connect to (`ssh://`,
/// `npipe://`, a bare hostname) resolves to [`SocketKind::Unsupported`] rather
/// than being coerced into a guess.
fn from_docker_host(raw: &str) -> SocketKind {
    if let Some(path) = raw.strip_prefix("unix://") {
        return unix(path);
    }
    if raw.starts_with("tcp://") || raw.starts_with("http://") || raw.starts_with("https://") {
        return SocketKind::Tcp {
            endpoint: raw.to_owned(),
        };
    }
    if raw.starts_with('/') {
        return unix(raw);
    }
    SocketKind::Unsupported
}

fn unix(path: &str) -> SocketKind {
    SocketKind::Unix {
        path: path.to_owned(),
    }
}
