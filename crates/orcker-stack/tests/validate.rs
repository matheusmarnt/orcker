//! Acceptance tests for constructor validation (SPEC-0003 AC4).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::str::FromStr;

use orcker_stack::{
    PhpVersion, PhpVersionErrorReason, PortErrorReason, PortField, Ports, SiteName,
    SiteNameErrorReason, StackError,
};

#[test]
fn rejects_invalid_inputs() {
    let site_cases: &[(&str, SiteNameErrorReason)] = &[
        ("", SiteNameErrorReason::Empty),
        ("Acme", SiteNameErrorReason::InvalidCharacter),
        ("acme.test", SiteNameErrorReason::InvalidCharacter),
        ("acme_shop", SiteNameErrorReason::InvalidCharacter),
        ("acme shop", SiteNameErrorReason::InvalidCharacter),
        ("acmé", SiteNameErrorReason::InvalidCharacter),
        ("-acme", SiteNameErrorReason::LeadingOrTrailingHyphen),
        ("acme-", SiteNameErrorReason::LeadingOrTrailingHyphen),
    ];
    for (input, expected) in site_cases {
        match SiteName::new(input) {
            Err(StackError::InvalidSiteName { reason, .. }) => {
                assert_eq!(reason, *expected, "site name {input:?}");
            }
            other => panic!("site name {input:?}: expected {expected:?}, got {other:?}"),
        }
    }

    let too_long = "a".repeat(64);
    match SiteName::new(&too_long) {
        Err(StackError::InvalidSiteName {
            reason: SiteNameErrorReason::TooLong,
            ..
        }) => {}
        other => panic!("64-char site name: expected TooLong, got {other:?}"),
    }

    for accepted in ["acme", "acme-shop", "a1", "0shop9"] {
        assert_eq!(
            SiteName::new(accepted).unwrap().as_str(),
            accepted,
            "site name {accepted:?} should be accepted verbatim"
        );
    }

    let php_cases: &[(&str, PhpVersionErrorReason)] = &[
        ("", PhpVersionErrorReason::Empty),
        ("8", PhpVersionErrorReason::Malformed),
        ("eight.three", PhpVersionErrorReason::Malformed),
        ("8.0", PhpVersionErrorReason::Unsupported),
        ("8.6", PhpVersionErrorReason::Unsupported),
        ("7.4", PhpVersionErrorReason::Unsupported),
    ];
    for (input, expected) in php_cases {
        match PhpVersion::from_str(input) {
            Err(StackError::InvalidPhpVersion { reason, .. }) => {
                assert_eq!(reason, *expected, "php version {input:?}");
            }
            other => panic!("php version {input:?}: expected {expected:?}, got {other:?}"),
        }
    }

    for (input, expected) in [
        ("8.1", PhpVersion::V81),
        ("8.2", PhpVersion::V82),
        ("8.3", PhpVersion::V83),
        ("8.4", PhpVersion::V84),
        ("8.5", PhpVersion::V85),
    ] {
        assert_eq!(PhpVersion::from_str(input).unwrap(), expected);
        assert_eq!(expected.to_string(), input);
    }

    let port_cases: &[(u16, u16, PortField, PortErrorReason)] = &[
        (0, 15173, PortField::HttpLoopback, PortErrorReason::Zero),
        (18080, 0, PortField::Vite, PortErrorReason::Zero),
        (
            18080,
            18080,
            PortField::Vite,
            PortErrorReason::DuplicatesHttpLoopback,
        ),
    ];
    for (http, vite, field, reason) in port_cases {
        match Ports::new(*http, *vite) {
            Err(StackError::InvalidPort {
                field: got_field,
                reason: got_reason,
                ..
            }) => {
                assert_eq!(got_field, *field, "ports ({http}, {vite}) field");
                assert_eq!(got_reason, *reason, "ports ({http}, {vite}) reason");
            }
            other => panic!("ports ({http}, {vite}): expected {reason:?}, got {other:?}"),
        }
    }

    let ports = Ports::new(18080, 15173).unwrap();
    assert_eq!(ports.http_loopback(), 18080);
    assert_eq!(ports.vite(), 15173);
}
