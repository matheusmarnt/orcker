//! Turning probe results into an [`orcker_ipc::DockerStatus`].
//!
//! Every problem this module reports carries a hint (NFR-08). `orcker status`
//! reports, it never fails: a stopped engine is a populated `problems` list,
//! not an error.

use orcker_ipc::{ComposeStatus, DockerStatus, EngineProblem, EngineProblemCode, SocketKind};

use super::version::Version;
use super::{MIN_COMPOSE_VERSION, MIN_ENGINE_VERSION};

/// What the two probes came back with, before any judgement is applied.
#[derive(Debug, Clone, Default)]
pub struct ProbeOutcome {
    /// The engine's reported version string, `None` when nothing answered.
    pub engine_version: Option<String>,
    /// Raw stdout of `docker compose version --format json`, `None` when the
    /// command could not be run at all.
    pub compose_output: Option<String>,
}

/// Read a version out of `docker compose version --format json`.
///
/// Accepts the documented JSON shape (`{"version":"v2.29.7"}`), the same
/// payload under a capitalised key as older plugins emit, and the plain
/// `Docker Compose version v2.29.7` line a plugin prints when it does not
/// understand `--format json`.
#[must_use]
pub fn parse_compose_version(output: &str) -> Option<Version> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        let field = json
            .get("version")
            .or_else(|| json.get("Version"))
            .and_then(serde_json::Value::as_str);
        if let Some(raw) = field {
            return Version::parse(raw);
        }
    }
    output.split_whitespace().find_map(Version::parse)
}

/// Assemble the client-facing snapshot from a resolved endpoint and the probes.
#[must_use]
pub fn assemble(socket: SocketKind, outcome: &ProbeOutcome) -> DockerStatus {
    let reachable = outcome.engine_version.is_some();
    let mut problems = Vec::new();

    if matches!(socket, SocketKind::Unsupported) {
        problems.push(problem(
            EngineProblemCode::PlatformUnsupported,
            "no Docker endpoint Orcker can reach on this platform".to_owned(),
        ));
    } else if let Some(raw) = &outcome.engine_version {
        if let Some(found) = Version::parse(raw) {
            if !found.satisfies(MIN_ENGINE_VERSION) {
                problems.push(problem(
                    EngineProblemCode::EngineTooOld,
                    format!(
                        "docker engine {found} is older than the supported {MIN_ENGINE_VERSION}"
                    ),
                ));
            }
        }
    } else {
        problems.push(problem(
            EngineProblemCode::EngineUnreachable,
            format!("docker engine unreachable on {}", endpoint_label(&socket)),
        ));
    }

    let compose = match outcome
        .compose_output
        .as_deref()
        .and_then(parse_compose_version)
    {
        Some(found) if found.satisfies(MIN_COMPOSE_VERSION) => ComposeStatus::Found {
            version: found.to_string(),
        },
        Some(found) => {
            problems.push(problem(
                EngineProblemCode::ComposeTooOld,
                format!("docker compose {found} is older than the supported {MIN_COMPOSE_VERSION}"),
            ));
            ComposeStatus::TooOld {
                found: found.to_string(),
                min: MIN_COMPOSE_VERSION.to_string(),
            }
        }
        None => {
            problems.push(problem(
                EngineProblemCode::ComposeMissing,
                "the docker compose plugin is not installed".to_owned(),
            ));
            ComposeStatus::Missing
        }
    };

    DockerStatus {
        socket,
        reachable,
        engine_version: outcome.engine_version.clone(),
        compose,
        problems,
    }
}

/// How an endpoint reads inside a problem message.
#[must_use]
pub fn endpoint_label(socket: &SocketKind) -> String {
    match socket {
        SocketKind::Unix { path } => path.clone(),
        SocketKind::Tcp { endpoint } => endpoint.clone(),
        _ => "no supported endpoint".to_owned(),
    }
}

/// The hint that resolves `code`.
///
/// The two "too old" hints interpolate [`MIN_ENGINE_VERSION`] /
/// [`MIN_COMPOSE_VERSION`] rather than spelling the numbers out, so the floor
/// has exactly one source of truth (R5): bumping a constant cannot leave a hint
/// telling the user to upgrade to the version they already have.
///
/// `EngineProblemCode` is `#[non_exhaustive]` from this crate's side, so the
/// wildcard arm is required; it points at `orcker doctor` rather than pretending
/// to know a code added after this build.
#[must_use]
pub fn hint_for(code: EngineProblemCode) -> String {
    match code {
        EngineProblemCode::EngineUnreachable => {
            "start Docker (`systemctl --user start docker`, or launch Docker Desktop) and re-run `orcker status`".to_owned()
        }
        EngineProblemCode::EngineTooOld => {
            format!("upgrade Docker Engine to {MIN_ENGINE_VERSION} or newer, then re-run `orcker status`")
        }
        EngineProblemCode::ComposeMissing => {
            "install the docker compose plugin (`docker-compose-plugin` on Debian/Ubuntu; bundled with Docker Desktop)".to_owned()
        }
        EngineProblemCode::ComposeTooOld => {
            format!("upgrade the docker compose plugin to {MIN_COMPOSE_VERSION} or newer")
        }
        EngineProblemCode::PlatformUnsupported => {
            "point DOCKER_HOST at a unix socket or `tcp://` endpoint Orcker can reach".to_owned()
        }
        _ => "run `orcker doctor` for the details of this finding".to_owned(),
    }
}

/// Build a problem with its hint attached.
#[must_use]
pub fn problem(code: EngineProblemCode, message: String) -> EngineProblem {
    EngineProblem {
        code,
        message,
        hint: hint_for(code),
    }
}
