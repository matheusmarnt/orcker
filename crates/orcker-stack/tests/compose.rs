//! Acceptance tests for the reference compose renderer (SPEC-0003 AC1..AC3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use orcker_stack::{render_compose, DbEngine, PhpVersion, Ports, Preset, SiteName, StackConfig};

/// The reference topology the committed snapshot was rendered from.
fn reference_config() -> StackConfig {
    StackConfig::new(
        SiteName::new("acme").unwrap(),
        PhpVersion::V83,
        DbEngine::Postgres,
        Preset::Reference,
        Ports::new(18080, 15173).unwrap(),
        1000,
        1000,
    )
    .unwrap()
}

#[test]
fn reference_postgres_snapshot() {
    let rendered = render_compose(&reference_config()).unwrap();
    let expected = include_str!("fixtures/compose_reference_postgres.yml");
    assert_eq!(
        rendered, expected,
        "rendered compose drifted from tests/fixtures/compose_reference_postgres.yml"
    );
}

#[test]
fn deterministic_output() {
    let cfg = reference_config();
    let first = render_compose(&cfg).unwrap();
    let second = render_compose(&cfg).unwrap();
    assert_eq!(
        first.as_bytes(),
        second.as_bytes(),
        "two renders of the same config must be byte-identical"
    );

    let twin = render_compose(&reference_config()).unwrap();
    assert_eq!(
        first.as_bytes(),
        twin.as_bytes(),
        "an equal config built independently must render identically"
    );
}

#[test]
fn loopback_only_ports() {
    let rendered = render_compose(&reference_config()).unwrap();

    assert!(
        !rendered.contains("0.0.0.0"),
        "no project port may be published on 0.0.0.0:\n{rendered}"
    );
    assert!(
        !rendered.contains("privileged"),
        "generated stacks must never be privileged:\n{rendered}"
    );
    assert!(
        !rendered.contains("docker.sock"),
        "generated stacks must never mount the Docker socket:\n{rendered}"
    );

    let published = published_ports(&rendered);
    assert_eq!(
        published,
        vec!["127.0.0.1:15173:5173", "127.0.0.1:18080:80"],
        "unexpected published port set"
    );
    for entry in &published {
        assert!(
            entry.starts_with("127.0.0.1:"),
            "port {entry:?} is not bound to the loopback interface"
        );
    }
}

/// Collects every published port entry, sorted so the assertion does not depend
/// on service order.
fn published_ports(rendered: &str) -> Vec<String> {
    let mut inside = false;
    let mut found: Vec<String> = Vec::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if trimmed == "ports:" {
            inside = true;
            continue;
        }
        if inside {
            match trimmed.strip_prefix("- ") {
                Some(entry) => found.push(entry.trim_matches('"').to_owned()),
                None => inside = false,
            }
        }
    }
    found.sort();
    found
}
