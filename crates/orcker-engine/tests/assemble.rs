//! Detection driven through fake side effects: no Docker daemon is touched.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use async_trait::async_trait;
use orcker_engine::pure::{HostOs, DEFAULT_UNIX_SOCKET};
use orcker_engine::traits::{ComposeCli, EngineApi};
use orcker_engine::EngineError;
use orcker_ipc::{ComposeStatus, EngineProblemCode, SocketKind};

/// Answers with `version` for the endpoint at `answers_on`, and refuses
/// everywhere else. `None` means nothing answers anywhere.
struct FakeEngine {
    answers_on: Option<String>,
    version: String,
}

#[async_trait]
impl EngineApi for FakeEngine {
    async fn version(&self, socket: &SocketKind) -> Result<String, EngineError> {
        let path = match socket {
            SocketKind::Unix { path } => path.clone(),
            SocketKind::Tcp { endpoint } => endpoint.clone(),
            _ => return Err(EngineError::Unsupported),
        };
        if self.answers_on.as_deref() == Some(path.as_str()) {
            Ok(self.version.clone())
        } else {
            Err(EngineError::Unreachable {
                endpoint: path,
                source_message: "connection refused".to_owned(),
            })
        }
    }
}

struct FakeCompose(Result<String, ()>);

#[async_trait]
impl ComposeCli for FakeCompose {
    async fn version_output(&self) -> Result<String, EngineError> {
        self.0
            .clone()
            .map_err(|()| EngineError::ComposeUnavailable("no such file or directory".to_owned()))
    }
}

fn candidates() -> Vec<SocketKind> {
    orcker_engine::pure::resolve_socket(None, Some("/home/dev"), HostOs::MacOs)
}

#[tokio::test]
async fn healthy_environment_has_no_problems() {
    let status = orcker_engine::detect(
        &FakeEngine {
            answers_on: Some(DEFAULT_UNIX_SOCKET.to_owned()),
            version: "27.3.1".to_owned(),
        },
        &FakeCompose(Ok(r#"{"version":"v2.29.7"}"#.to_owned())),
        &candidates(),
    )
    .await;

    assert!(status.reachable);
    assert_eq!(
        status.socket,
        SocketKind::Unix {
            path: DEFAULT_UNIX_SOCKET.to_owned()
        }
    );
    assert_eq!(status.engine_version.as_deref(), Some("27.3.1"));
    assert_eq!(
        status.compose,
        ComposeStatus::Found {
            version: "2.29.7".to_owned()
        }
    );
    assert!(status.problems.is_empty(), "{:?}", status.problems);
}

#[tokio::test]
async fn falls_through_to_the_desktop_socket() {
    let status = orcker_engine::detect(
        &FakeEngine {
            answers_on: Some("/home/dev/.docker/run/docker.sock".to_owned()),
            version: "27.3.1".to_owned(),
        },
        &FakeCompose(Ok(r#"{"version":"v2.29.7"}"#.to_owned())),
        &candidates(),
    )
    .await;

    assert!(status.reachable);
    assert_eq!(
        status.socket,
        SocketKind::Unix {
            path: "/home/dev/.docker/run/docker.sock".to_owned()
        }
    );
}

#[tokio::test]
async fn engine_down_reports_a_problem_with_a_hint() {
    let status = orcker_engine::detect(
        &FakeEngine {
            answers_on: None,
            version: String::new(),
        },
        &FakeCompose(Ok(r#"{"version":"v2.29.7"}"#.to_owned())),
        &candidates(),
    )
    .await;

    assert!(!status.reachable);
    assert_eq!(status.engine_version, None);
    assert_eq!(
        status.socket,
        SocketKind::Unix {
            path: DEFAULT_UNIX_SOCKET.to_owned()
        },
        "the first candidate is the one worth naming in the hint"
    );
    let problem = status
        .problems
        .iter()
        .find(|p| p.code == EngineProblemCode::EngineUnreachable)
        .expect("an unreachable engine must be reported");
    assert!(!problem.hint.is_empty());
    assert!(!problem.message.is_empty());
}

#[tokio::test]
async fn compose_absent_is_its_own_problem() {
    let status = orcker_engine::detect(
        &FakeEngine {
            answers_on: Some(DEFAULT_UNIX_SOCKET.to_owned()),
            version: "27.3.1".to_owned(),
        },
        &FakeCompose(Err(())),
        &candidates(),
    )
    .await;

    assert!(status.reachable);
    assert_eq!(status.compose, ComposeStatus::Missing);
    assert_eq!(
        status
            .problems
            .iter()
            .map(|p| p.code)
            .collect::<Vec<EngineProblemCode>>(),
        vec![EngineProblemCode::ComposeMissing]
    );
}

#[tokio::test]
async fn unsupported_platform_reports_unsupported() {
    let status = orcker_engine::detect(
        &FakeEngine {
            answers_on: None,
            version: String::new(),
        },
        &FakeCompose(Err(())),
        &orcker_engine::pure::resolve_socket(None, None, HostOs::Unsupported),
    )
    .await;

    assert_eq!(status.socket, SocketKind::Unsupported);
    assert!(status
        .problems
        .iter()
        .any(|p| p.code == EngineProblemCode::PlatformUnsupported));
}
