//! The pure layer: no I/O, no clock, no env reads, no async.
//!
//! Everything here takes injected data and returns data, so the whole
//! detection policy is table-testable without a Docker daemon in sight.

pub mod detect;
pub mod socket;
pub mod version;

pub use detect::{
    assemble, endpoint_label, hint_for, parse_compose_version, problem, ProbeOutcome,
};
pub use socket::{resolve_socket, HostOs, DEFAULT_UNIX_SOCKET, DESKTOP_USER_SOCKET};
pub use version::Version;

/// Oldest Docker Engine Orcker supports (PRD NFR-04).
///
/// Adjust the supported floor **only** here and in [`MIN_COMPOSE_VERSION`]; the
/// comparison itself is generic and lives in [`version::Version::satisfies`].
pub const MIN_ENGINE_VERSION: Version = Version::new(24, 0, 0);

/// Oldest `docker compose` plugin Orcker supports (PRD NFR-04).
pub const MIN_COMPOSE_VERSION: Version = Version::new(2, 20, 0);

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;
    use orcker_ipc::{ComposeStatus, EngineProblemCode, SocketKind};

    /// `DOCKER_HOST`, `HOME`, the host OS, and the candidates they must yield.
    type SocketCase = (
        Option<&'static str>,
        Option<&'static str>,
        HostOs,
        Vec<SocketKind>,
    );

    fn unix(path: &str) -> SocketKind {
        SocketKind::Unix {
            path: path.to_owned(),
        }
    }

    #[test]
    fn socket_resolution_matrix() {
        let home = Some("/home/dev");
        let cases: &[SocketCase] = &[
            (None, home, HostOs::Linux, vec![unix(DEFAULT_UNIX_SOCKET)]),
            (
                None,
                home,
                HostOs::MacOs,
                vec![
                    unix(DEFAULT_UNIX_SOCKET),
                    unix("/home/dev/.docker/run/docker.sock"),
                ],
            ),
            (None, None, HostOs::MacOs, vec![unix(DEFAULT_UNIX_SOCKET)]),
            (
                None,
                home,
                HostOs::Unsupported,
                vec![SocketKind::Unsupported],
            ),
            (
                Some("unix:///run/user/1000/docker.sock"),
                home,
                HostOs::Linux,
                vec![unix("/run/user/1000/docker.sock")],
            ),
            (
                Some("/run/podman/podman.sock"),
                home,
                HostOs::Linux,
                vec![unix("/run/podman/podman.sock")],
            ),
            (
                Some("tcp://10.0.0.5:2375"),
                home,
                HostOs::Linux,
                vec![SocketKind::Tcp {
                    endpoint: "tcp://10.0.0.5:2375".to_owned(),
                }],
            ),
            (
                Some("ssh://dev@buildbox"),
                home,
                HostOs::Linux,
                vec![SocketKind::Unsupported],
            ),
            (
                Some("unix:///run/user/1000/docker.sock"),
                home,
                HostOs::Unsupported,
                vec![unix("/run/user/1000/docker.sock")],
            ),
        ];
        for (docker_host, home, os, want) in cases {
            let got = resolve_socket(*docker_host, *home, *os);
            assert_eq!(
                &got, want,
                "DOCKER_HOST={docker_host:?} HOME={home:?} os={os:?}"
            );
        }
    }

    #[test]
    fn compose_version_parsing() {
        let cases: &[(&str, Option<Version>)] = &[
            (r#"{"version":"v2.29.7"}"#, Some(Version::new(2, 29, 7))),
            (r#"{"version": "2.24.5"}"#, Some(Version::new(2, 24, 5))),
            (
                "{\n  \"version\": \"v2.20.2\"\n}\n",
                Some(Version::new(2, 20, 2)),
            ),
            (r#"{"Version":"v2.17.3"}"#, Some(Version::new(2, 17, 3))),
            (
                "Docker Compose version v2.29.7\n",
                Some(Version::new(2, 29, 7)),
            ),
            (
                r#"{"version":"v2.30.0-desktop.1"}"#,
                Some(Version::new(2, 30, 0)),
            ),
            (r#"{"version":"v2.31"}"#, Some(Version::new(2, 31, 0))),
            ("", None),
            ("docker: 'compose' is not a docker command.\n", None),
        ];
        for (output, want) in cases {
            assert_eq!(&parse_compose_version(output), want, "output={output:?}");
        }
    }

    #[test]
    fn minimum_version_policy() {
        let socket = unix(DEFAULT_UNIX_SOCKET);

        let healthy = assemble(
            socket.clone(),
            &ProbeOutcome {
                engine_version: Some("27.3.1".to_owned()),
                compose_output: Some(r#"{"version":"v2.29.7"}"#.to_owned()),
            },
        );
        assert!(healthy.reachable);
        assert_eq!(healthy.engine_version.as_deref(), Some("27.3.1"));
        assert_eq!(
            healthy.compose,
            ComposeStatus::Found {
                version: "2.29.7".to_owned()
            }
        );
        assert!(
            healthy.problems.is_empty(),
            "healthy environment reported {:?}",
            healthy.problems
        );

        let old = assemble(
            socket.clone(),
            &ProbeOutcome {
                engine_version: Some("23.0.6".to_owned()),
                compose_output: Some(r#"{"version":"v2.10.2"}"#.to_owned()),
            },
        );
        assert_eq!(
            old.compose,
            ComposeStatus::TooOld {
                found: "2.10.2".to_owned(),
                min: MIN_COMPOSE_VERSION.to_string(),
            }
        );
        let codes: Vec<EngineProblemCode> = old.problems.iter().map(|p| p.code).collect();
        assert_eq!(
            codes,
            vec![
                EngineProblemCode::EngineTooOld,
                EngineProblemCode::ComposeTooOld
            ]
        );
        assert!(
            old.problems.iter().all(|p| !p.hint.is_empty()),
            "every problem must carry an actionable hint: {:?}",
            old.problems
        );
        assert!(
            hint_for(EngineProblemCode::EngineTooOld).contains(&MIN_ENGINE_VERSION.to_string()),
            "the floor has one source of truth: the hint must name MIN_ENGINE_VERSION, not a \
             hardcoded copy that a constant bump would leave stale"
        );
        assert!(
            hint_for(EngineProblemCode::ComposeTooOld).contains(&MIN_COMPOSE_VERSION.to_string()),
            "the floor has one source of truth: the hint must name MIN_COMPOSE_VERSION"
        );

        let missing = assemble(
            socket,
            &ProbeOutcome {
                engine_version: Some("27.3.1".to_owned()),
                compose_output: None,
            },
        );
        assert_eq!(missing.compose, ComposeStatus::Missing);
        assert_eq!(
            missing
                .problems
                .iter()
                .map(|p| p.code)
                .collect::<Vec<EngineProblemCode>>(),
            vec![EngineProblemCode::ComposeMissing]
        );

        assert!(Version::new(24, 0, 0).satisfies(MIN_ENGINE_VERSION));
        assert!(!Version::new(23, 12, 9).satisfies(MIN_ENGINE_VERSION));
        assert!(Version::new(2, 20, 0).satisfies(MIN_COMPOSE_VERSION));
        assert!(!Version::new(2, 19, 99).satisfies(MIN_COMPOSE_VERSION));
    }
}
