//! Byte-exact wire-stability assertions for every `Request`,
//! `Response`, and `ErrorCode` variant.
//!
//! These literals are the published contract. A rename, reorder, or
//! casing change of any field or variant fails this file, which fails
//! CI before any downstream client sees a divergent wire format.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_names
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use orcker_ipc::{
    types::{PhpVersion, Site},
    CaStatus, Channel, CloudflaredSource, CloudflaredStatus, Diagnosis, DiagnosisCode, ErrorCode,
    FixReport, FixResult, MailAttachment, MailDetail, MailHeader, MailStatus, MailSummary,
    NamedTunnelMeta, PortRedirectTargets, PortStatus, ProxyEntry, ProxyRuleEntry, Request,
    Response, Severity, SiteCounts, SiteHostname, StagedArtifact, StatusReport, ToolStatus,
    TunnelInfo, TunnelKind, TunnelRunState, UpdateSource,
};

// ---------- Request ----------

#[test]
fn request_ping_byte_shape() {
    let s = serde_json::to_string(&Request::Ping).unwrap();
    assert_eq!(s, r#"{"type":"ping"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::Ping);
}

#[test]
fn request_list_sites_byte_shape() {
    let s = serde_json::to_string(&Request::ListSites).unwrap();
    assert_eq!(s, r#"{"type":"list_sites"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::ListSites);
}

#[test]
fn request_set_lan_enabled_byte_shape() {
    let s = serde_json::to_string(&Request::SetLanEnabled { enabled: true }).unwrap();
    assert_eq!(s, r#"{"type":"set_lan_enabled","enabled":true}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::SetLanEnabled { enabled: true });
}

#[test]
fn request_mint_remote_setup_code_byte_shape() {
    let s = serde_json::to_string(&Request::MintRemoteSetupCode).unwrap();
    assert_eq!(s, r#"{"type":"mint_remote_setup_code"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::MintRemoteSetupCode);
}

#[test]
fn request_trust_browsers_byte_shape() {
    let s = serde_json::to_string(&Request::TrustBrowsers { uninstall: false }).unwrap();
    assert_eq!(s, r#"{"type":"trust_browsers","uninstall":false}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::TrustBrowsers { uninstall: false });
}

#[test]
fn response_browser_trust_byte_shape() {
    let r = Response::BrowserTrust {
        attempted: 3,
        succeeded: 2,
        certutil_missing: false,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"browser_trust","attempted":3,"succeeded":2,"certutil_missing":false}"#
    );
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn browser_trust_enum_byte_shape() {
    for (v, tag) in [
        (orcker_ipc::BrowserTrust::Trusted, "trusted"),
        (orcker_ipc::BrowserTrust::Untrusted, "untrusted"),
        (orcker_ipc::BrowserTrust::ToolMissing, "tool_missing"),
    ] {
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, format!("\"{tag}\""));
        let back: orcker_ipc::BrowserTrust = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn response_remote_setup_byte_shape() {
    let r = Response::RemoteSetup {
        code: "deadbeef".into(),
        url: "http://192.168.1.42:7073/remote-setup?code=deadbeef".into(),
        script_sha256: "ab".repeat(32),
        expires_in_secs: 900,
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = format!(
        r#"{{"type":"remote_setup","code":"deadbeef","url":"http://192.168.1.42:7073/remote-setup?code=deadbeef","script_sha256":"{}","expires_in_secs":900}}"#,
        "ab".repeat(32)
    );
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_park_byte_shape() {
    let r = Request::Park {
        path: PathBuf::from("/srv/foo"),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"park","path":"/srv/foo"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_link_byte_shape() {
    let r = Request::Link {
        name: "foo".into(),
        path: PathBuf::from("/srv/foo"),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"link","name":"foo","path":"/srv/foo"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_unlink_byte_shape() {
    let r = Request::Unlink { name: "foo".into() };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"unlink","name":"foo"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_add_domain_byte_shape() {
    let r = Request::AddDomain {
        name: "foo".into(),
        domain: "api.foo.test".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"add_domain","name":"foo","domain":"api.foo.test"}"#
    );
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_remove_domain_byte_shape() {
    let r = Request::RemoveDomain {
        name: "foo".into(),
        domain: "*.foo.test".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"remove_domain","name":"foo","domain":"*.foo.test"}"#
    );
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_set_primary_domain_byte_shape() {
    let r = Request::SetPrimaryDomain {
        name: "foo".into(),
        domain: "corp.test".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_primary_domain","name":"foo","domain":"corp.test"}"#
    );
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_reset_domains_byte_shape() {
    let r = Request::ResetDomains { name: "foo".into() };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"reset_domains","name":"foo"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_list_parked_byte_shape() {
    let s = serde_json::to_string(&Request::ListParked).unwrap();
    assert_eq!(s, r#"{"type":"list_parked"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::ListParked);
}

#[test]
fn request_unpark_byte_shape() {
    let r = Request::Unpark {
        path: "/srv/sites".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"unpark","path":"/srv/sites"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_set_secure_byte_shape() {
    let r = Request::SetSecure {
        name: "foo".into(),
        secure: true,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_secure","name":"foo","secure":true}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn request_set_web_root_byte_shape() {
    let some = Request::SetWebRoot {
        name: "foo".into(),
        path: Some("public".into()),
    };
    let s = serde_json::to_string(&some).unwrap();
    assert_eq!(s, r#"{"type":"set_web_root","name":"foo","path":"public"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), some);

    let none = Request::SetWebRoot {
        name: "foo".into(),
        path: None,
    };
    let s = serde_json::to_string(&none).unwrap();
    assert_eq!(s, r#"{"type":"set_web_root","name":"foo","path":null}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), none);
}

#[test]
fn request_daemon_info_byte_shape() {
    let s = serde_json::to_string(&Request::DaemonInfo).unwrap();
    assert_eq!(s, r#"{"type":"daemon_info"}"#);
    let back: Request = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Request::DaemonInfo);
}

#[test]
fn request_status_byte_shape() {
    let s = serde_json::to_string(&Request::Status).unwrap();
    assert_eq!(s, r#"{"type":"status"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::Status
    );
}

#[test]
fn request_diagnose_byte_shape() {
    let s = serde_json::to_string(&Request::Diagnose).unwrap();
    assert_eq!(s, r#"{"type":"diagnose"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::Diagnose
    );
}

#[test]
fn request_restart_daemon_byte_shape() {
    let s = serde_json::to_string(&Request::RestartDaemon).unwrap();
    assert_eq!(s, r#"{"type":"restart_daemon"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::RestartDaemon
    );
}

#[test]
fn request_doctor_fix_byte_shape() {
    let s = serde_json::to_string(&Request::DoctorFix).unwrap();
    assert_eq!(s, r#"{"type":"doctor_fix"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::DoctorFix
    );
}

// ---------- Response ----------

#[test]
fn response_pong_byte_shape() {
    let s = serde_json::to_string(&Response::Pong).unwrap();
    assert_eq!(s, r#"{"type":"pong"}"#);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Response::Pong);
}

#[test]
fn response_ok_byte_shape() {
    let s = serde_json::to_string(&Response::Ok).unwrap();
    assert_eq!(s, r#"{"type":"ok"}"#);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, Response::Ok);
}

#[test]
fn response_sites_zero_byte_shape() {
    let r = Response::Sites { sites: vec![] };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"sites","sites":[]}"#);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

/// A non-WordPress `SiteEntry` - `is_wordpress` is omitted from the wire
/// (`skip_serializing_if`), so this is what most sites look like.
fn plain(site: Site) -> orcker_ipc::SiteEntry {
    orcker_ipc::SiteEntry {
        site,
        is_wordpress: false,
        primary_domain: None,
        domains: vec![],
        apex_shadowed_by: None,
        uses_front_controller: false,
        is_laravel: false,
    }
}

#[test]
fn response_sites_one_byte_shape() {
    let foo = Site::parked("foo", "/srv/foo", PhpVersion::new(8, 3)).unwrap();
    let r = Response::Sites {
        sites: vec![plain(foo)],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"foo","document_root":"/srv/foo","php":"8.3","secure":false,"kind":"parked","uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_sites_two_byte_shape() {
    let alpha = Site::parked("alpha", "/srv/alpha", PhpVersion::new(8, 3)).unwrap();
    let mut beta = Site::linked("beta", "/srv/beta", PhpVersion::new(7, 4)).unwrap();
    beta.set_secure(true);
    let r = Response::Sites {
        sites: vec![plain(alpha), plain(beta)],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"alpha","document_root":"/srv/alpha","php":"8.3","secure":false,"kind":"parked","uses_front_controller":false},{"name":"beta","document_root":"/srv/beta","php":"7.4","secure":true,"kind":"linked","uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_sites_with_web_subpath_byte_shape() {
    let mut app = Site::linked("app", "/srv/app", PhpVersion::new(8, 3)).unwrap();
    app.set_web_subpath("public");
    let r = Response::Sites {
        sites: vec![plain(app)],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"app","document_root":"/srv/app","web_subpath":"public","php":"8.3","secure":false,"kind":"linked","uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_sites_wordpress_byte_shape() {
    let blog = Site::parked("blog", "/srv/blog", PhpVersion::new(8, 3)).unwrap();
    let r = Response::Sites {
        sites: vec![orcker_ipc::SiteEntry {
            site: blog,
            is_wordpress: true,
            primary_domain: None,
            domains: vec![],
            apex_shadowed_by: None,
            uses_front_controller: false,
            is_laravel: false,
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"blog","document_root":"/srv/blog","php":"8.3","secure":false,"kind":"parked","is_wordpress":true,"uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_sites_customized_domains_byte_shape() {
    let blog = Site::parked("blog", "/srv/blog", PhpVersion::new(8, 3)).unwrap();
    let r = Response::Sites {
        sites: vec![orcker_ipc::SiteEntry {
            site: blog,
            is_wordpress: false,
            primary_domain: Some("corp.test".into()),
            domains: vec!["corp.test".into(), "*.blog.test".into()],
            apex_shadowed_by: Some("shop".into()),
            uses_front_controller: false,
            is_laravel: false,
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"blog","document_root":"/srv/blog","php":"8.3","secure":false,"kind":"parked","primary_domain":"corp.test","domains":["corp.test","*.blog.test"],"apex_shadowed_by":"shop","uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_sites_wp_auto_login_byte_shape() {
    let mut blog = Site::parked("blog", "/srv/blog", PhpVersion::new(8, 3)).unwrap();
    blog.set_wp_auto_login(true);
    blog.set_wp_auto_login_user(Some("admin".into()));
    let r = Response::Sites {
        sites: vec![orcker_ipc::SiteEntry {
            site: blog,
            is_wordpress: true,
            primary_domain: None,
            domains: vec![],
            apex_shadowed_by: None,
            uses_front_controller: false,
            is_laravel: false,
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"sites","sites":[{"name":"blog","document_root":"/srv/blog","php":"8.3","secure":false,"kind":"parked","wp_auto_login":true,"wp_auto_login_user":"admin","is_wordpress":true,"uses_front_controller":false}]}"#;
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_parked_byte_shape() {
    let r = Response::Parked {
        paths: vec!["/a".into(), "/b".into()],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"parked","paths":["/a","/b"]}"#);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_parked_empty_byte_shape() {
    let r = Response::Parked { paths: vec![] };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"parked","paths":[]}"#);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_info_byte_shape() {
    let r = Response::Info {
        dns_addr: "127.0.0.1:1053".parse().unwrap(),
        tld: "test".into(),
        ca_path: std::path::PathBuf::from("/home/u/.local/share/orcker/ca.cert.pem"),
        ca_fingerprint: "ab".repeat(32),
        http_port: 8080,
        https_port: 8443,
        fallback_http: 8080,
        fallback_https: 8443,
        dns_port: 1053,
        lan_ip: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    // `lan_ip: None` is skipped, so the byte shape is unchanged from before v18.
    let expected = format!(
        r#"{{"type":"info","dns_addr":"127.0.0.1:1053","tld":"test","ca_path":"/home/u/.local/share/orcker/ca.cert.pem","ca_fingerprint":"{}","http_port":8080,"https_port":8443,"fallback_http":8080,"fallback_https":8443,"dns_port":1053}}"#,
        "ab".repeat(32)
    );
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);

    // With a LAN IP present it appears as a trailing field.
    let with_lan = Response::Info {
        dns_addr: "127.0.0.1:1053".parse().unwrap(),
        tld: "test".into(),
        ca_path: std::path::PathBuf::from("/x"),
        ca_fingerprint: "ab".repeat(32),
        http_port: 8080,
        https_port: 8443,
        fallback_http: 8080,
        fallback_https: 8443,
        dns_port: 1053,
        lan_ip: Some("192.168.1.42".parse().unwrap()),
    };
    let s2 = serde_json::to_string(&with_lan).unwrap();
    let expected_lan = format!(
        r#"{{"type":"info","dns_addr":"127.0.0.1:1053","tld":"test","ca_path":"/x","ca_fingerprint":"{}","http_port":8080,"https_port":8443,"fallback_http":8080,"fallback_https":8443,"dns_port":1053,"lan_ip":"192.168.1.42"}}"#,
        "ab".repeat(32)
    );
    assert_eq!(s2, expected_lan);
    assert_eq!(serde_json::from_str::<Response>(&s2).unwrap(), with_lan);

    let legacy = format!(
        r#"{{"type":"info","dns_addr":"127.0.0.1:1053","tld":"test","ca_path":"/x","ca_fingerprint":"{}"}}"#,
        "ab".repeat(32)
    );
    let decoded: Response = serde_json::from_str(&legacy).unwrap();
    assert!(matches!(
        decoded,
        Response::Info {
            http_port: 0,
            https_port: 0,
            fallback_http: 0,
            fallback_https: 0,
            dns_port: 0,
            ..
        }
    ));
}

#[test]
fn response_error_each_code_byte_shape() {
    for (code, text) in [
        (ErrorCode::NotFound, "not_found"),
        (ErrorCode::AlreadyExists, "already_exists"),
        (ErrorCode::InvalidPath, "invalid_path"),
        (ErrorCode::PortInUse, "port_in_use"),
        (ErrorCode::ExtensionLoadFailed, "extension_load_failed"),
        (ErrorCode::LegacyRestricted, "legacy_restricted"),
        (ErrorCode::Internal, "internal"),
    ] {
        let r = Response::Error {
            code,
            message: "x".into(),
        };
        let s = serde_json::to_string(&r).unwrap();
        let expected = format!(r#"{{"type":"error","code":"{text}","message":"x"}}"#);
        assert_eq!(s, expected, "code = {code:?}");
        let back: Response = serde_json::from_str(&s).unwrap();
        assert_eq!(back, r, "code = {code:?}");
    }
}

#[test]
fn response_status_byte_shape() {
    let r = Response::Status {
        report: Box::new(StatusReport {
            daemon_pid: 4242,
            uptime_secs: 7,
            daemon_rss_bytes: Some(2048),
            tld: "test".into(),
            http: PortStatus {
                requested: 80,
                bound: 8080,
                fell_back: true,
            },
            https: PortStatus {
                requested: 443,
                bound: 8443,
                fell_back: true,
            },
            dns_addr: "127.0.0.1:1053".parse().unwrap(),
            ca: CaStatus {
                path: PathBuf::from("/x/ca.cert.pem"),
                fingerprint: "ab".repeat(32),
                trusted_system: Some(false),
                browser_trust: None,
            },
            resolver_installed: Some(true),
            port_redirect: None,
            foreign_web_listener: None,
            resolver_backup: None,
            sites: SiteCounts {
                parked: 1,
                linked: 2,
                secured: 1,
            },
            load_avg: Some([100, 50, 25]),
            daemon_version: "2.0.1".into(),
            mail: None,
            web_unbound: None,
            dns_unbound: None,
            boot_id: None,
            shared_sites: 0,
            symlink_protection: true,
            shadows: vec![],
            mcp_enabled: false,
            lan_enabled: false,
            lan_ip: None,
            lan_setup_bound: None,
            port_redirect_targets: None,
            lan_redirect_targets: None,
        }),
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = format!(
        r#"{{"type":"status","report":{{"daemon_pid":4242,"uptime_secs":7,"daemon_rss_bytes":2048,"tld":"test","http":{{"requested":80,"bound":8080,"fell_back":true}},"https":{{"requested":443,"bound":8443,"fell_back":true}},"dns_addr":"127.0.0.1:1053","ca":{{"path":"/x/ca.cert.pem","fingerprint":"{}","trusted_system":false}},"resolver_installed":true,"sites":{{"parked":1,"linked":2,"secured":1}},"load_avg":[100,50,25],"daemon_version":"2.0.1","symlink_protection":true,"mcp_enabled":false,"lan_enabled":false}}}}"#,
        "ab".repeat(32)
    );
    assert_eq!(s, expected);
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn status_port_redirect_appears_only_when_some() {
    let mut report = sample_status_report();
    report.port_redirect = Some(true);
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        s.contains(r#""resolver_installed":true,"port_redirect":true"#),
        "{s}"
    );

    report.port_redirect = None;
    let s = serde_json::to_string(&report).unwrap();
    assert!(!s.contains("port_redirect"), "{s}");
}

#[test]
fn status_redirect_targets_appear_only_when_some() {
    let mut report = sample_status_report();
    report.port_redirect_targets = Some(PortRedirectTargets {
        http: 8080,
        https: 8443,
    });
    report.lan_redirect_targets = Some(PortRedirectTargets {
        http: 8081,
        https: 8444,
    });
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        s.contains(r#""port_redirect_targets":{"http":8080,"https":8443}"#),
        "{s}"
    );
    assert!(
        s.contains(r#""lan_redirect_targets":{"http":8081,"https":8444}"#),
        "{s}"
    );

    report.port_redirect_targets = None;
    report.lan_redirect_targets = None;
    let s = serde_json::to_string(&report).unwrap();
    assert!(!s.contains("redirect_targets"), "{s}");
}

#[test]
fn status_foreign_web_listener_appears_only_when_some() {
    let mut report = sample_status_report();
    report.port_redirect = Some(true);
    report.foreign_web_listener = Some(true);
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        s.contains(r#""port_redirect":true,"foreign_web_listener":true"#),
        "{s}"
    );

    report.foreign_web_listener = None;
    let s = serde_json::to_string(&report).unwrap();
    assert!(!s.contains("foreign_web_listener"), "{s}");
}

#[test]
fn status_resolver_backup_appears_only_when_some() {
    let mut report = sample_status_report();
    report.port_redirect = Some(true);
    report.resolver_backup = Some(
        "/Library/Application Support/io.orcker.Orcker/resolver-backups/test-1.conf".to_owned(),
    );
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        s.contains(r#""port_redirect":true,"resolver_backup":"/Library"#),
        "{s}"
    );

    report.resolver_backup = None;
    let s = serde_json::to_string(&report).unwrap();
    assert!(!s.contains("resolver_backup"), "{s}");
}

/// A minimal healthy report for field-presence assertions.
fn sample_status_report() -> StatusReport {
    StatusReport {
        daemon_pid: 1,
        uptime_secs: 0,
        daemon_rss_bytes: None,
        tld: "test".into(),
        http: PortStatus {
            requested: 80,
            bound: 8080,
            fell_back: true,
        },
        https: PortStatus {
            requested: 443,
            bound: 8443,
            fell_back: true,
        },
        dns_addr: "127.0.0.1:1053".parse().unwrap(),
        ca: CaStatus {
            path: PathBuf::from("/x/ca.cert.pem"),
            fingerprint: "ab".repeat(32),
            trusted_system: Some(true),
            browser_trust: None,
        },
        resolver_installed: Some(true),
        port_redirect: None,
        foreign_web_listener: None,
        resolver_backup: None,
        sites: SiteCounts::default(),
        load_avg: None,
        daemon_version: "2.0.1".into(),
        mail: None,
        web_unbound: None,
        dns_unbound: None,
        boot_id: None,
        shared_sites: 0,
        symlink_protection: true,
        shadows: vec![],
        mcp_enabled: false,
        lan_enabled: false,
        lan_ip: None,
        lan_setup_bound: None,
        port_redirect_targets: None,
        lan_redirect_targets: None,
    }
}

#[test]
fn response_diagnoses_byte_shape() {
    let r = Response::Diagnoses {
        items: vec![Diagnosis {
            code: DiagnosisCode::PortFallback,
            severity: Severity::Warn,
            title: "t".into(),
            detail: "d".into(),
            remedy: Some("sudo orcker elevate ports".into()),
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"diagnoses","items":[{"code":"port_fallback","severity":"warn","title":"t","detail":"d","remedy":"sudo orcker elevate ports"}]}"#
    );
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn response_doctor_fix_byte_shape() {
    let r = Response::DoctorFix {
        report: FixReport {
            performed: vec![FixResult {
                code: DiagnosisCode::ResolverNotInstalled,
                ok: true,
                message: "installed the resolver".into(),
            }],
            manual: vec![Diagnosis {
                code: DiagnosisCode::CaNotTrusted,
                severity: Severity::Warn,
                title: "t".into(),
                detail: "d".into(),
                remedy: Some("sudo orcker elevate trust".into()),
            }],
        },
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"doctor_fix","report":{"performed":[{"code":"resolver_not_installed","ok":true,"message":"installed the resolver"}],"manual":[{"code":"ca_not_trusted","severity":"warn","title":"t","detail":"d","remedy":"sudo orcker elevate trust"}]}}"#
    );
    let back: Response = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn severity_each_variant_byte_shape() {
    for (sv, expected) in [
        (Severity::Ok, r#""ok""#),
        (Severity::Warn, r#""warn""#),
        (Severity::Fail, r#""fail""#),
    ] {
        assert_eq!(serde_json::to_string(&sv).unwrap(), expected);
    }
}

#[test]
fn diagnosis_code_each_variant_byte_shape() {
    let cases: &[(DiagnosisCode, &str)] = &[
        (DiagnosisCode::DaemonDown, r#""daemon_down""#),
        (DiagnosisCode::PortFallback, r#""port_fallback""#),
        (DiagnosisCode::WebPortsUnbound, r#""web_ports_unbound""#),
        (
            DiagnosisCode::ForeignWebListener,
            r#""foreign_web_listener""#,
        ),
        (DiagnosisCode::DnsPortUnbound, r#""dns_port_unbound""#),
        (DiagnosisCode::CaNotTrusted, r#""ca_not_trusted""#),
        (
            DiagnosisCode::ResolverNotInstalled,
            r#""resolver_not_installed""#,
        ),
        (DiagnosisCode::NoSites, r#""no_sites""#),
        (
            DiagnosisCode::ResolverBackupSaved,
            r#""resolver_backup_saved""#,
        ),
        (DiagnosisCode::ServiceFailed, r#""service_failed""#),
        (DiagnosisCode::BinDirNotOnPath, r#""bin_dir_not_on_path""#),
        (
            DiagnosisCode::SymlinkProtectionDisabled,
            r#""symlink_protection_disabled""#,
        ),
        (DiagnosisCode::DomainShadowed, r#""domain_shadowed""#),
        (DiagnosisCode::PortRedirectStale, r#""port_redirect_stale""#),
        (DiagnosisCode::LanRedirectStale, r#""lan_redirect_stale""#),
        (DiagnosisCode::AllGood, r#""all_good""#),
    ];
    for (code, expected) in cases {
        assert_eq!(&serde_json::to_string(code).unwrap(), expected, "{code:?}");
    }
}

// ---------- ErrorCode (standalone) ----------

#[test]
fn error_code_each_variant_byte_shape() {
    let cases: &[(ErrorCode, &str)] = &[
        (ErrorCode::NotFound, r#""not_found""#),
        (ErrorCode::AlreadyExists, r#""already_exists""#),
        (ErrorCode::InvalidPath, r#""invalid_path""#),
        (ErrorCode::PortInUse, r#""port_in_use""#),
        (ErrorCode::ExtensionLoadFailed, r#""extension_load_failed""#),
        (ErrorCode::PortReserved, r#""port_reserved""#),
        (ErrorCode::SiteNotFound, r#""site_not_found""#),
        (ErrorCode::SiteNotLaravel, r#""site_not_laravel""#),
        (ErrorCode::UnknownServiceType, r#""unknown_service_type""#),
        (
            ErrorCode::InstanceAlreadyExists,
            r#""instance_already_exists""#,
        ),
        (ErrorCode::LanNotReady, r#""lan_not_ready""#),
        (ErrorCode::LegacyRestricted, r#""legacy_restricted""#),
        (ErrorCode::Internal, r#""internal""#),
    ];
    for (code, expected) in cases {
        let s = serde_json::to_string(code).unwrap();
        assert_eq!(&s, expected, "code = {code:?}");
        let back: ErrorCode = serde_json::from_str(&s).unwrap();
        assert_eq!(back, *code);
    }
}

// ---------- Services (request + response) ----------

#[test]
fn request_set_front_controller_byte_shape() {
    let r = Request::SetFrontController {
        name: "blog".into(),
        enabled: true,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_front_controller","name":"blog","enabled":true}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_add_route_rule_byte_shape() {
    let r = Request::AddRouteRule {
        site: "portal".into(),
        prefix: "/api".into(),
        target: "api/index.php".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"add_route_rule","site":"portal","prefix":"/api","target":"api/index.php"}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_remove_route_rule_byte_shape() {
    let r = Request::RemoveRouteRule {
        site: "portal".into(),
        prefix: "/api".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"remove_route_rule","site":"portal","prefix":"/api"}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_list_routes_byte_shape() {
    let r = Request::ListRoutes;
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"list_routes"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_routes_byte_shape() {
    let r = Response::Routes {
        rules: vec![orcker_ipc::RouteRuleEntry {
            site: "portal".into(),
            prefix: "/api".into(),
            target: "api/index.php".into(),
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"routes","rules":[{"site":"portal","prefix":"/api","target":"api/index.php"}]}"#
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

/// A per-site instance status with the additive fields populated: `type_id`,
/// `site`, and `error` appear after `supports_databases`, in that order.
/// `supports_overrides` is additive: omitted from the wire when `false` (the
/// shape older clients already parse) and emitted last when `true`.
/// An older daemon's payload, which has no `supports_overrides` key at all,
/// still decodes and reads as "no overrides".
// ---------- Dumps ----------

// ---------- Mail ----------

#[test]
fn request_list_mails_byte_shape() {
    let s = serde_json::to_string(&Request::ListMails).unwrap();
    assert_eq!(s, r#"{"type":"list_mails"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::ListMails
    );
}

#[test]
fn request_get_mail_byte_shape() {
    let r = Request::GetMail {
        id: "000001".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"get_mail","id":"000001"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_clear_mails_byte_shape() {
    let s = serde_json::to_string(&Request::ClearMails).unwrap();
    assert_eq!(s, r#"{"type":"clear_mails"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::ClearMails
    );
}

#[test]
fn request_delete_mails_byte_shape() {
    let r = Request::DeleteMails {
        ids: vec!["000001".into(), "000002".into()],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"delete_mails","ids":["000001","000002"]}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_mark_mails_read_byte_shape() {
    let r = Request::MarkMailsRead {
        ids: vec!["000001".into()],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"mark_mails_read","ids":["000001"]}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_mail_port_byte_shape() {
    let r = Request::SetMailPort { port: 2525 };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_mail_port","port":2525}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_fallback_ports_byte_shape() {
    let r = Request::SetFallbackPorts {
        http: 8080,
        https: 8443,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_fallback_ports","http":8080,"https":8443}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_dns_port_byte_shape() {
    let r = Request::SetDnsPort { port: 1053 };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_dns_port","port":1053}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_mail_enabled_byte_shape() {
    let r = Request::SetMailEnabled { enabled: true };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_mail_enabled","enabled":true}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_symlink_protection_byte_shape() {
    let r = Request::SetSymlinkProtection { enabled: true };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_symlink_protection","enabled":true}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_mcp_enabled_byte_shape() {
    let r = Request::SetMcpEnabled { enabled: true };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_mcp_enabled","enabled":true}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_mails_byte_shape() {
    let r = Response::Mails {
        mails: vec![MailSummary {
            id: "000001".into(),
            from: "Example <hello@example.com>".into(),
            to: vec!["test@test.com".into()],
            subject: "Hi".into(),
            date_epoch: 1_700_000_000,
            read: false,
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"mails","mails":[{"id":"000001","from":"Example <hello@example.com>","to":["test@test.com"],"subject":"Hi","date_epoch":1700000000,"read":false}]}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_mails_legacy_without_read_decodes_default() {
    let legacy = r#"{"type":"mails","mails":[{"id":"000001","from":"Example <hello@example.com>","to":["test@test.com"],"subject":"Hi","date_epoch":1700000000}]}"#;
    match serde_json::from_str::<Response>(legacy).unwrap() {
        Response::Mails { mails } => {
            assert_eq!(mails.len(), 1);
            assert!(!mails[0].read);
        }
        other => panic!("expected Mails, got {other:?}"),
    }
}

#[test]
fn response_mail_byte_shape() {
    let r = Response::Mail {
        mail: Box::new(MailDetail {
            id: "000001".into(),
            from: "Example <hello@example.com>".into(),
            to: vec!["test@test.com".into()],
            subject: "Hi".into(),
            date_epoch: 1_700_000_000,
            headers: vec![MailHeader {
                name: "Subject".into(),
                value: "Hi".into(),
            }],
            html_body: Some("<p>Hi</p>".into()),
            text_body: None,
            attachments: vec![],
        }),
    };
    let s = serde_json::to_string(&r).unwrap();
    // `attachments` is omitted from the wire when empty (`skip_serializing_if`).
    let expected = r#"{"type":"mail","mail":{"id":"000001","from":"Example <hello@example.com>","to":["test@test.com"],"subject":"Hi","date_epoch":1700000000,"headers":[{"name":"Subject","value":"Hi"}],"html_body":"<p>Hi</p>","text_body":null}}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_mail_with_attachment_byte_shape() {
    let r = Response::Mail {
        mail: Box::new(MailDetail {
            id: "000002".into(),
            from: "a@b.c".into(),
            to: vec!["d@e.f".into()],
            subject: "Invoice".into(),
            date_epoch: 1_700_000_000,
            headers: vec![],
            html_body: None,
            text_body: Some("See attached.".into()),
            attachments: vec![MailAttachment {
                filename: "invoice.pdf".into(),
                content_type: "application/pdf".into(),
                size: 8,
                data: "ZmFrZS1wZGY=".into(),
            }],
        }),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(
        s.contains(r#""attachments":[{"filename":"invoice.pdf","content_type":"application/pdf","size":8,"data":"ZmFrZS1wZGY="}]"#),
        "attachment must appear on the wire: {s}"
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_mail_legacy_without_attachments_decodes_default() {
    // An older daemon that does not emit `attachments` must decode to an empty vec.
    let legacy = r#"{"type":"mail","mail":{"id":"000001","from":"Example <hello@example.com>","to":["test@test.com"],"subject":"Hi","date_epoch":1700000000,"headers":[],"html_body":null,"text_body":null}}"#;
    match serde_json::from_str::<Response>(legacy).unwrap() {
        Response::Mail { mail } => {
            assert!(
                mail.attachments.is_empty(),
                "legacy mail without attachments field must decode as empty"
            );
        }
        other => panic!("expected Mail, got {other:?}"),
    }
}

#[test]
fn status_mail_appears_only_when_some() {
    let mut report = sample_status_report();
    let s = serde_json::to_string(&report).unwrap();
    assert!(!s.contains("mail"), "empty mail must be omitted: {s}");

    report.mail = Some(MailStatus {
        enabled: true,
        port: 2525,
        listening: true,
        count: 3,
        unread: 2,
    });
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        s.contains(r#""mail":{"enabled":true,"port":2525,"listening":true,"count":3,"unread":2}"#),
        "{s}"
    );
    let back: StatusReport = serde_json::from_str(&s).unwrap();
    assert_eq!(back, report);
}

#[test]
fn status_mail_legacy_without_unread_decodes_default() {
    let legacy = r#"{"enabled":true,"port":2525,"listening":true,"count":3}"#;
    let mail: MailStatus = serde_json::from_str(legacy).unwrap();
    assert_eq!(mail.count, 3);
    assert_eq!(mail.unread, 0);
}

#[test]
fn status_dns_unbound_appears_only_when_some() {
    let mut report = sample_status_report();
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        !s.contains("dns_unbound"),
        "empty dns_unbound must be omitted: {s}"
    );

    report.dns_unbound = Some(1053);
    let s = serde_json::to_string(&report).unwrap();
    assert!(s.contains(r#""dns_unbound":1053"#), "{s}");
    let back: StatusReport = serde_json::from_str(&s).unwrap();
    assert_eq!(back, report);
}

#[test]
fn status_shared_sites_appears_only_when_nonzero() {
    let mut report = sample_status_report();
    let s = serde_json::to_string(&report).unwrap();
    assert!(
        !s.contains("shared_sites"),
        "zero shared_sites must be omitted: {s}"
    );

    report.shared_sites = 3;
    let s = serde_json::to_string(&report).unwrap();
    assert!(s.contains(r#""shared_sites":3"#), "{s}");
    let back: StatusReport = serde_json::from_str(&s).unwrap();
    assert_eq!(back, report);
}

// ---------- Tools ----------

#[test]
fn request_list_tools_byte_shape() {
    let r = Request::ListTools;
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"list_tools"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_install_tool_byte_shape() {
    let r = Request::InstallTool {
        tool: "node".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"install_tool","tool":"node"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_uninstall_tool_byte_shape() {
    let r = Request::UninstallTool { tool: "bun".into() };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"uninstall_tool","tool":"bun"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_tools_byte_shape() {
    let r = Response::Tools {
        tools: vec![ToolStatus {
            id: "node".into(),
            display_name: "Node.js".into(),
            installed: true,
            version: Some("v24.17.0".into()),
            binaries: vec!["node".into(), "npm".into(), "npx".into()],
            external: false,
            external_path: None,
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"tools","tools":[{"id":"node","display_name":"Node.js","installed":true,"version":"v24.17.0","binaries":["node","npm","npx"]}]}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_tools_external_byte_shape() {
    let r = Response::Tools {
        tools: vec![ToolStatus {
            id: "node".into(),
            display_name: "Node.js".into(),
            installed: false,
            version: None,
            binaries: vec!["node".into(), "npm".into(), "npx".into()],
            external: true,
            external_path: Some("/opt/homebrew/bin/node".into()),
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"tools","tools":[{"id":"node","display_name":"Node.js","installed":false,"version":null,"binaries":["node","npm","npx"],"external":true,"external_path":"/opt/homebrew/bin/node"}]}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

// ---------- CreateSite / job model ----------

#[test]
fn request_job_status_byte_shape() {
    let r = Request::JobStatus {
        job_id: "j1".into(),
        cursor: 7,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"job_status","job_id":"j1","cursor":7}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_job_cancel_byte_shape() {
    let r = Request::JobCancel {
        job_id: "j1".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"job_cancel","job_id":"j1"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_job_started_byte_shape() {
    let r = Response::JobStarted {
        job_id: "j1".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"job_started","job_id":"j1"}"#);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_job_progress_byte_shape() {
    use orcker_ipc::JobState;
    let r = Response::JobProgress {
        state: JobState::Running,
        phase: "Scaffolding".into(),
        log: vec!["line one".into()],
        next_cursor: 1,
        error: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"job_progress","state":"running","phase":"Scaffolding","log":["line one"],"next_cursor":1,"error":null}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn request_install_tool_streamed_byte_shape() {
    let r = Request::InstallToolStreamed {
        tool: "laravel".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"install_tool_streamed","tool":"laravel"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

// ---------- Self-update (Channel / CheckUpdate / SetUpdateChannel / UpdateStatus) ----------

#[test]
fn channel_each_variant_byte_shape() {
    assert_eq!(
        serde_json::to_string(&Channel::Stable).unwrap(),
        r#""stable""#
    );
    assert_eq!(serde_json::to_string(&Channel::Edge).unwrap(), r#""edge""#);
    assert_eq!(
        serde_json::from_str::<Channel>(r#""stable""#).unwrap(),
        Channel::Stable
    );
    assert_eq!(
        serde_json::from_str::<Channel>(r#""edge""#).unwrap(),
        Channel::Edge
    );
}

#[test]
fn update_source_each_variant_byte_shape() {
    assert_eq!(
        serde_json::to_string(&UpdateSource::Live).unwrap(),
        r#""live""#
    );
    assert_eq!(
        serde_json::to_string(&UpdateSource::Cached).unwrap(),
        r#""cached""#
    );
    assert_eq!(
        serde_json::from_str::<UpdateSource>(r#""cached""#).unwrap(),
        UpdateSource::Cached
    );
}

#[test]
fn request_check_update_byte_shape() {
    let r = Request::CheckUpdate {
        channel: Some(Channel::Edge),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"check_update","channel":"edge"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);

    let none = Request::CheckUpdate { channel: None };
    let s = serde_json::to_string(&none).unwrap();
    assert_eq!(s, r#"{"type":"check_update","channel":null}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), none);
}

#[test]
fn request_cached_update_status_byte_shape() {
    let r = Request::CachedUpdateStatus;
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"cached_update_status"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_update_channel_byte_shape() {
    let r = Request::SetUpdateChannel {
        channel: Channel::Stable,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_update_channel","channel":"stable"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn staged_artifact_each_variant_byte_shape() {
    assert_eq!(
        serde_json::to_string(&StagedArtifact::AppTarGz).unwrap(),
        r#""app_tar_gz""#
    );
    assert_eq!(
        serde_json::to_string(&StagedArtifact::Deb).unwrap(),
        r#""deb""#
    );
    assert_eq!(
        serde_json::from_str::<StagedArtifact>(r#""deb""#).unwrap(),
        StagedArtifact::Deb
    );
    assert_eq!(
        serde_json::to_string(&StagedArtifact::Pacman).unwrap(),
        r#""pacman""#
    );
    assert_eq!(
        serde_json::from_str::<StagedArtifact>(r#""pacman""#).unwrap(),
        StagedArtifact::Pacman
    );
    assert_eq!(
        serde_json::to_string(&StagedArtifact::Rpm).unwrap(),
        r#""rpm""#
    );
    assert_eq!(
        serde_json::from_str::<StagedArtifact>(r#""rpm""#).unwrap(),
        StagedArtifact::Rpm
    );
}

#[test]
fn request_stage_update_byte_shape() {
    let r = Request::StageUpdate {
        channel: Some(Channel::Stable),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"stage_update","channel":"stable"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_staged_byte_shape() {
    let r = Response::Staged {
        path: "/x/Orcker.app.tar.gz".into(),
        version: "2.0.5".into(),
        kind: StagedArtifact::AppTarGz,
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected =
        r#"{"type":"staged","path":"/x/Orcker.app.tar.gz","version":"2.0.5","kind":"app_tar_gz"}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_update_status_byte_shape() {
    let r = Response::UpdateStatus {
        current: "2.0.2-rc.3".into(),
        latest_stable: Some("2.0.1".into()),
        latest_edge: Some("2.0.2-rc.3".into()),
        channel: Channel::Stable,
        available: false,
        target: None,
        ahead_of_stable: true,
        source: UpdateSource::Live,
        checked_at_epoch: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"update_status","current":"2.0.2-rc.3","latest_stable":"2.0.1","latest_edge":"2.0.2-rc.3","channel":"stable","available":false,"target":null,"ahead_of_stable":true,"source":"live"}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);

    let with_ts = Response::UpdateStatus {
        current: "2.0.2-rc.3".into(),
        latest_stable: Some("2.0.1".into()),
        latest_edge: Some("2.0.2-rc.3".into()),
        channel: Channel::Stable,
        available: false,
        target: None,
        ahead_of_stable: true,
        source: UpdateSource::Cached,
        checked_at_epoch: Some(1_719_445_200),
    };
    let s = serde_json::to_string(&with_ts).unwrap();
    assert!(
        s.contains(r#""source":"cached","checked_at_epoch":1719445200"#),
        "{s}"
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), with_ts);
}

#[test]
fn request_install_cloudflared_streamed_byte_shape() {
    let s = serde_json::to_string(&Request::InstallCloudflaredStreamed).unwrap();
    assert_eq!(s, r#"{"type":"install_cloudflared_streamed"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::InstallCloudflaredStreamed
    );
}

#[test]
fn request_start_quick_tunnel_byte_shape() {
    let r = Request::StartQuickTunnel { site: "app".into() };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"start_quick_tunnel","site":"app"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_stop_tunnel_byte_shape() {
    let r = Request::StopTunnel { site: "app".into() };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"stop_tunnel","site":"app"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_tunnel_status_byte_shape() {
    let s = serde_json::to_string(&Request::TunnelStatus).unwrap();
    assert_eq!(s, r#"{"type":"tunnel_status"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::TunnelStatus
    );
}

#[test]
fn response_tunnels_byte_shape() {
    let r = Response::Tunnels {
        tunnels: vec![TunnelInfo {
            site: "app".into(),
            kind: TunnelKind::Quick,
            state: TunnelRunState::Running,
            url: Some("https://calm-river-1234.trycloudflare.com".into()),
            hostname: None,
        }],
        cloudflared: CloudflaredStatus {
            installed: true,
            version: Some("2026.6.1".into()),
            source: Some(CloudflaredSource::Managed),
            logged_in: false,
        },
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"tunnels","tunnels":[{"site":"app","kind":"quick","state":"running","url":"https://calm-river-1234.trycloudflare.com"}],"cloudflared":{"installed":true,"version":"2026.6.1","source":"managed","logged_in":false}}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

/// A named tunnel omits `url` and includes `hostname`; an empty tunnel list and
/// uninstalled `cloudflared` round-trip too.
#[test]
fn response_tunnels_named_and_empty_byte_shape() {
    let named = Response::Tunnels {
        tunnels: vec![TunnelInfo {
            site: "shop".into(),
            kind: TunnelKind::Named,
            state: TunnelRunState::Running,
            url: None,
            hostname: Some("shop.example.com".into()),
        }],
        cloudflared: CloudflaredStatus {
            installed: true,
            version: None,
            source: Some(CloudflaredSource::System),
            logged_in: true,
        },
    };
    let s = serde_json::to_string(&named).unwrap();
    assert!(s.contains(r#""kind":"named""#), "{s}");
    assert!(s.contains(r#""hostname":"shop.example.com""#), "{s}");
    assert!(!s.contains(r#""url""#), "{s}");
    assert!(s.contains(r#""source":"system""#), "{s}");
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), named);

    let empty = Response::Tunnels {
        tunnels: vec![],
        cloudflared: CloudflaredStatus {
            installed: false,
            version: None,
            source: None,
            logged_in: false,
        },
    };
    let s = serde_json::to_string(&empty).unwrap();
    assert_eq!(
        s,
        r#"{"type":"tunnels","tunnels":[],"cloudflared":{"installed":false,"logged_in":false}}"#
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), empty);
}

#[test]
fn tunnel_run_state_each_variant_byte_shape() {
    for (st, expected) in [
        (TunnelRunState::Running, r#""running""#),
        (TunnelRunState::Failed, r#""failed""#),
    ] {
        assert_eq!(serde_json::to_string(&st).unwrap(), expected);
    }
}

#[test]
fn tunnel_kind_each_variant_byte_shape() {
    for (k, expected) in [
        (TunnelKind::Quick, r#""quick""#),
        (TunnelKind::Named, r#""named""#),
    ] {
        assert_eq!(serde_json::to_string(&k).unwrap(), expected);
    }
}

#[test]
fn cloudflared_source_each_variant_byte_shape() {
    for (src, expected) in [
        (CloudflaredSource::Managed, r#""managed""#),
        (CloudflaredSource::System, r#""system""#),
    ] {
        assert_eq!(serde_json::to_string(&src).unwrap(), expected);
    }
}

#[test]
fn request_cloudflared_login_byte_shape() {
    let s = serde_json::to_string(&Request::CloudflaredLogin).unwrap();
    assert_eq!(s, r#"{"type":"cloudflared_login"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::CloudflaredLogin
    );
}

#[test]
fn request_create_named_tunnel_byte_shape() {
    let r = Request::CreateNamedTunnel {
        name: "mysite".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"create_named_tunnel","name":"mysite"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_list_named_tunnels_byte_shape() {
    let s = serde_json::to_string(&Request::ListNamedTunnels).unwrap();
    assert_eq!(s, r#"{"type":"list_named_tunnels"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::ListNamedTunnels
    );
}

#[test]
fn request_route_tunnel_dns_byte_shape() {
    let r = Request::RouteTunnelDns {
        tunnel: "mysite".into(),
        hostname: "app.example.com".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"route_tunnel_dns","tunnel":"mysite","hostname":"app.example.com"}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_site_tunnel_byte_shape() {
    let set = Request::SetSiteTunnel {
        site: "app".into(),
        hostname: Some("app.example.com".into()),
    };
    let s = serde_json::to_string(&set).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_site_tunnel","site":"app","hostname":"app.example.com"}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), set);

    let clear = Request::SetSiteTunnel {
        site: "app".into(),
        hostname: None,
    };
    let s = serde_json::to_string(&clear).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_site_tunnel","site":"app","hostname":null}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), clear);
}

#[test]
fn request_start_named_tunnel_byte_shape() {
    let s = serde_json::to_string(&Request::StartNamedTunnel).unwrap();
    assert_eq!(s, r#"{"type":"start_named_tunnel"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::StartNamedTunnel
    );
}

#[test]
fn request_stop_named_tunnel_byte_shape() {
    let s = serde_json::to_string(&Request::StopNamedTunnel).unwrap();
    assert_eq!(s, r#"{"type":"stop_named_tunnel"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::StopNamedTunnel
    );
}

#[test]
fn request_delete_named_tunnel_byte_shape() {
    let r = Request::DeleteNamedTunnel {
        name: "mysite".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"delete_named_tunnel","name":"mysite"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

/// `zone: None` serializes to nothing (the field is skipped), preserving the
/// byte shape for older clients.
#[test]
fn response_named_tunnels_with_none_zone_skips_field() {
    let r = Response::NamedTunnels {
        tunnels: vec![NamedTunnelMeta {
            name: "mysite".into(),
            uuid: "uuid-123".into(),
        }],
        sites: vec![SiteHostname {
            site: "app".into(),
            hostname: "app.example.com".into(),
        }],
        zone: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"named_tunnels","tunnels":[{"name":"mysite","uuid":"uuid-123"}],"sites":[{"site":"app","hostname":"app.example.com"}]}"#
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_named_tunnels_with_zone_byte_shape() {
    let r = Response::NamedTunnels {
        tunnels: vec![],
        sites: vec![],
        zone: Some("example.com".into()),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"named_tunnels","tunnels":[],"sites":[],"zone":"example.com"}"#
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

// ---------- Groups ----------

#[test]
fn request_list_groups_byte_shape() {
    let s = serde_json::to_string(&Request::ListGroups).unwrap();
    assert_eq!(s, r#"{"type":"list_groups"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&s).unwrap(),
        Request::ListGroups
    );
}

#[test]
fn request_create_group_byte_shape() {
    let r = Request::CreateGroup {
        name: "Blog".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"create_group","name":"Blog"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_delete_group_byte_shape() {
    let r = Request::DeleteGroup {
        name: "Blog".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"delete_group","name":"Blog"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_group_order_byte_shape() {
    let r = Request::SetGroupOrder {
        order: vec!["Blog".into(), "Shop".into()],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"set_group_order","order":["Blog","Shop"]}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn request_set_site_group_byte_shape() {
    let some = Request::SetSiteGroup {
        site: "app".into(),
        group: Some("Blog".into()),
    };
    let s = serde_json::to_string(&some).unwrap();
    assert_eq!(
        s,
        r#"{"type":"set_site_group","site":"app","group":"Blog"}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), some);

    let none = Request::SetSiteGroup {
        site: "app".into(),
        group: None,
    };
    let s = serde_json::to_string(&none).unwrap();
    assert_eq!(s, r#"{"type":"set_site_group","site":"app","group":null}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), none);
}

#[test]
fn request_rename_group_byte_shape() {
    let r = Request::RenameGroup {
        from: "Blog".into(),
        to: "Journal".into(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"rename_group","from":"Blog","to":"Journal"}"#);
    assert_eq!(serde_json::from_str::<Request>(&s).unwrap(), r);
}

#[test]
fn response_groups_byte_shape() {
    let r = Response::Groups {
        order: vec!["Blog".into(), "Shop".into()],
        members: BTreeMap::from([("app".to_string(), "Blog".to_string())]),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(
        s,
        r#"{"type":"groups","order":["Blog","Shop"],"members":{"app":"Blog"}}"#
    );
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

#[test]
fn response_groups_empty_byte_shape() {
    let r = Response::Groups {
        order: vec![],
        members: BTreeMap::new(),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(s, r#"{"type":"groups","order":[],"members":{}}"#);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

/// An uncustomised proxy carries no domain fields, so the wire bytes stay
/// identical to what daemons emitted before those fields existed.
#[test]
fn response_proxies_byte_shape() {
    let r = Response::Proxies {
        proxies: vec![ProxyEntry {
            name: "reverb".into(),
            target: "http://127.0.0.1:8080".into(),
            secure: false,
            primary_domain: None,
            domains: vec![],
        }],
        rules: vec![ProxyRuleEntry {
            site: "app".into(),
            prefix: "/app".into(),
            target: "http://127.0.0.1:9000".into(),
        }],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"proxies","proxies":[{"name":"reverb","target":"http://127.0.0.1:8080","secure":false}],"rules":[{"site":"app","prefix":"/app","target":"http://127.0.0.1:9000"}]}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}

/// A customized proxy appends its primary and full FQDN set after the
/// pre-existing fields, leaving their order untouched.
#[test]
fn response_proxies_customized_domains_byte_shape() {
    let r = Response::Proxies {
        proxies: vec![ProxyEntry {
            name: "reverb".into(),
            target: "http://127.0.0.1:8080".into(),
            secure: false,
            primary_domain: Some("corp.test".into()),
            domains: vec!["corp.test".into(), "*.reverb.test".into()],
        }],
        rules: vec![],
    };
    let s = serde_json::to_string(&r).unwrap();
    let expected = r#"{"type":"proxies","proxies":[{"name":"reverb","target":"http://127.0.0.1:8080","secure":false,"primary_domain":"corp.test","domains":["corp.test","*.reverb.test"]}],"rules":[]}"#;
    assert_eq!(s, expected);
    assert_eq!(serde_json::from_str::<Response>(&s).unwrap(), r);
}
