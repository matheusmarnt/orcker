//! Daemon → client response envelope and error-code enum.
//!
//! Internally tagged on `type`, `snake_case`. Wire-stability assertions
//! live in `tests/wire_stability.rs`.

use std::net::SocketAddr;
use std::path::PathBuf;

use orcker_core::Site;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::engine::DockerStatus;
use crate::status::{
    CloudflaredStatus, Diagnosis, FixReport, MailDetail, MailSummary, NamedTunnelMeta,
    SiteHostname, StatusReport, ToolStatus, TunnelInfo,
};

// Same rule: no per-field serde renames.
/// A response sent from the daemon to a client.
///
/// `serde_json::Value` payloads are only `PartialEq`. `PartialEq` is all the
/// wire-stability round-trips need.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Response {
    /// Reply to [`crate::Request::Ping`].
    Pong,
    /// Reply to [`crate::Request::ListSites`].
    Sites {
        /// The sites currently known to the daemon, in lexicographic
        /// name order.
        sites: Vec<SiteEntry>,
    },
    /// Generic success for mutating requests
    /// ([`crate::Request::Park`], [`crate::Request::Link`],
    /// [`crate::Request::Unlink`], /// [`crate::Request::SetSecure`]).
    Ok,
    /// Reply to [`crate::Request::ListProxies`] - whole-host proxies and
    /// per-site path-prefix rules.
    Proxies {
        /// Whole-host proxies, in config order.
        proxies: Vec<ProxyEntry>,
        /// Per-site path-prefix rules.
        rules: Vec<ProxyRuleEntry>,
    },
    /// A request failed. `code` is machine-readable; `message` is for
    /// human display.
    Error {
        /// Machine-readable error category.
        code: ErrorCode,
        /// Human-readable error message.
        message: String,
    },
    /// Reply to [`crate::Request::ListParked`] - the registered parked roots,
    /// in lexicographic order (the daemon stores them in a `BTreeSet`).
    Parked {
        /// Canonical parked root paths.
        paths: Vec<String>,
    },
    /// Reply to [`crate::Request::DaemonInfo`] - read-only runtime facts.
    Info {
        /// Address the embedded DNS responder is bound on (`127.0.0.1:<port>`).
        dns_addr: SocketAddr,
        /// The TLD served (e.g. `"test"`).
        tld: String,
        /// Absolute path to the local CA certificate PEM.
        ca_path: PathBuf,
        /// SHA-256 fingerprint of the CA cert, 64 lowercase hex chars.
        ca_fingerprint: String,
        /// The rootless HTTP port the daemon actually bound (e.g. 8080). The
        /// macOS `orcker elevate ports` flow redirects 80 → this. `#[serde(default)]`
        /// keeps older daemons (which omit it) decodable; defaults to 0.
        #[serde(default)]
        http_port: u16,
        /// The rootless HTTPS port the daemon actually bound (e.g. 8443).
        #[serde(default)]
        https_port: u16,
        /// The configured rootless HTTP fallback port (e.g. 8080) - what Settings
        /// edits. Distinct from `http_port`, which is the *bound* port and equals
        /// the desired port when privileged binding succeeds. `#[serde(default)]`
        /// keeps older daemons decodable; defaults to 0.
        #[serde(default)]
        fallback_http: u16,
        /// The configured rootless HTTPS fallback port (e.g. 8443).
        #[serde(default)]
        fallback_https: u16,
        /// The configured DNS responder port (`dns_port`, e.g. 1053) - what
        /// Settings edits. Distinct from `dns_addr`, which is the *bound* address
        /// (and stays the wanted addr when the DNS port couldn't bind).
        /// `#[serde(default)]` keeps older daemons (which omit it) decodable;
        /// defaults to 0.
        #[serde(default)]
        dns_port: u16,
        /// The host's LAN IPv4, present only when LAN mode is on and discovery
        /// succeeded. The macOS `orcker elevate lan` flow reads it (the sudo-side
        /// helper cannot discover it itself). `skip_serializing_if` keeps the
        /// absent-when-`None` bytes unchanged for older daemons/clients.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lan_ip: Option<std::net::Ipv4Addr>,
    },
    /// Reply to [`crate::Request::MintRemoteSetupCode`]: a freshly minted
    /// one-time bootstrap code and everything the CLI prints for the device.
    RemoteSetup {
        /// The one-time code (URL-safe), embedded in `url`.
        code: String,
        /// The plain-HTTP installer URL the remote device fetches
        /// (`http://<lan_ip>:<port>/remote-setup?code=<code>`). The installer is
        /// self-contained (it embeds the CA), and its integrity comes from
        /// `script_sha256`, not the transport.
        url: String,
        /// SHA-256 (64 lowercase hex) of the installer script served at `url`,
        /// which the user copy-pastes to the device so the pasted command can
        /// verify the script out-of-band before running it. This is the trust
        /// anchor.
        script_sha256: String,
        /// Seconds until the code expires.
        expires_in_secs: u64,
    },
    /// Reply to [`crate::Request::Status`] - a runtime health snapshot.
    ///
    /// Boxed so the (large) report does not bloat every `Response` value;
    /// `Box<T>` serializes transparently, so the wire bytes are unchanged.
    Status {
        /// The assembled health report.
        report: Box<StatusReport>,
    },
    /// Reply to [`crate::Request::EngineStatus`] - the Docker environment.
    ///
    /// Boxed for the same reason as [`Self::Status`]: inline it is the largest
    /// variant in the enum and pushes every `Result<_, Response>` in the daemon
    /// over clippy's `result_large_err` threshold. `Box<T>` serializes
    /// transparently, so the wire bytes are unchanged.
    EngineStatus {
        /// The assembled Docker snapshot.
        status: Box<DockerStatus>,
    },
    /// Reply to [`crate::Request::Diagnose`] - the doctor findings.
    Diagnoses {
        /// One entry per check that produced a finding.
        items: Vec<Diagnosis>,
    },
    /// Reply to [`crate::Request::DoctorFix`] - what was fixed + what remains.
    DoctorFix {
        /// The fix outcome.
        report: FixReport,
    },
    /// Reply to [`crate::Request::ListMails`] - captured email metadata, newest first.
    Mails {
        /// One entry per captured email.
        mails: Vec<MailSummary>,
    },
    /// Reply to [`crate::Request::GetMail`] - one captured email's full content.
    ///
    /// Boxed so the (large) `MailDetail` does not bloat every `Response` value -
    /// the same treatment as [`Self::Status`]. `Box<T>` serializes transparently,
    /// so the wire bytes are unchanged.
    Mail {
        /// The decoded email.
        mail: Box<MailDetail>,
    },
    /// Reply to [`crate::Request::ListTools`] - the installable dev tools.
    Tools {
        /// One entry per tool, with install status.
        tools: Vec<ToolStatus>,
    },
    /// Reply to a streamed install request - the background job was started.
    JobStarted {
        /// The job id to poll with [`crate::Request::JobStatus`].
        job_id: crate::JobId,
    },
    /// Reply to [`crate::Request::JobStatus`] - a job's current progress.
    JobProgress {
        /// The job's lifecycle state.
        state: crate::JobState,
        /// A short human label for the current phase (e.g. `"Scaffolding"`).
        phase: String,
        /// Log lines newer than the client's cursor, oldest first.
        log: Vec<String>,
        /// The cursor the client should send on its next poll.
        next_cursor: u64,
        /// Set when `state` is [`crate::JobState::Failed`]: the failure message.
        error: Option<String>,
    },
    /// Reply to [`crate::Request::CheckUpdate`] - the running version, both
    /// channel latests, the active channel preference, and whether an update is
    /// available. Versions are strings (e.g. `"2.0.2-rc.3"`) to keep this crate
    /// free of a semver dependency.
    UpdateStatus {
        /// The running Orcker version.
        current: String,
        /// Highest stable version available, or `None` if none / unknown.
        latest_stable: Option<String>,
        /// Highest edge (pre-release-inclusive) version available, or `None`.
        latest_edge: Option<String>,
        /// The channel this check resolved against (the preference, unless
        /// overridden for this check).
        channel: crate::Channel,
        /// Whether a newer version is available on `channel`.
        available: bool,
        /// The version `channel` would update to (strictly newer than current),
        /// or `None` when already up to date.
        target: Option<String>,
        /// True when the running version is a pre-release ahead of the latest
        /// stable - switching to stable would be a downgrade.
        ahead_of_stable: bool,
        /// Whether these figures are from a live fetch or a cached fallback.
        source: crate::UpdateSource,
        /// Unix epoch (seconds) when this result was obtained, for a "last
        /// checked …" display. `None` when never checked (or an older daemon that
        /// predates the field). `#[serde(default, skip_serializing_if)]` keeps the
        /// wire additive.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        checked_at_epoch: Option<u64>,
    },
    /// Reply to [`crate::Request::StageUpdate`] - the verified update artifact
    /// has been downloaded to `path`. The applier installs it.
    Staged {
        /// Absolute path to the verified, downloaded artifact on disk.
        path: String,
        /// The version that was staged (e.g. `"2.0.5"`).
        version: String,
        /// What kind of artifact it is (drives the applier's install method).
        kind: crate::StagedArtifact,
    },
    /// Reply to [`crate::Request::StartQuickTunnel`] / [`crate::Request::StopTunnel`]
    /// / [`crate::Request::TunnelStatus`] - the live tunnels plus `cloudflared`
    /// install status.
    Tunnels {
        /// One entry per live tunnel.
        tunnels: Vec<TunnelInfo>,
        /// `cloudflared` install / account status.
        cloudflared: CloudflaredStatus,
    },
    /// Reply to [`crate::Request::ListNamedTunnels`] - the account's named
    /// tunnels recorded locally, plus the per-site hostname mappings that are
    /// enabled in the consolidated tunnel.
    NamedTunnels {
        /// One entry per named tunnel.
        tunnels: Vec<NamedTunnelMeta>,
        /// The sites enabled in the named tunnel (site → hostname).
        sites: Vec<SiteHostname>,
        /// The authorized Cloudflare zone (domain) the account cert is scoped to,
        /// e.g. `"example.com"`. `None` when not logged in or unresolvable.
        /// `#[serde(default, skip_serializing_if)]` keeps the wire additive.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<String>,
    },
    /// Reply to [`crate::Request::ListGroups`] - the user-defined site groups in
    /// display order and the per-site membership map (site name → group name).
    Groups {
        /// Group display names, in display order.
        order: Vec<String>,
        /// Per-site membership: site name → group name. A site absent from the
        /// map is ungrouped ("Unallocated").
        members: BTreeMap<String, String>,
    },
    /// Reply to [`crate::Request::TrustBrowsers`] - the per-user browser NSS
    /// trust outcome.
    BrowserTrust {
        /// Number of NSS databases attempted (0 when nothing was found or
        /// `certutil` is missing).
        attempted: usize,
        /// Of those attempted, how many succeeded.
        succeeded: usize,
        /// `certutil` (`libnss3-tools`) was not installed, so nothing was
        /// changed - the client should tell the user to install it.
        certutil_missing: bool,
    },
    /// Reply to [`crate::Request::ListRoutes`] - every site's path-prefix
    /// routing rules.
    Routes {
        /// Per-site routing rules.
        rules: Vec<RouteRuleEntry>,
    },
}

/// One entry in [`Response::Sites`]: the site plus WordPress-detection
/// metadata.
///
/// This is a wire-only wrapper, not a new field on [`Site`] itself - `Site`'s
/// hand-written `Serialize`/`Deserialize` is shared between the wire and
/// `orcker.toml` persistence (`Config.linked: Vec<Site>`), and `WordPress`
/// detection is a runtime fact (it can change the moment the user runs
/// `wp core update`), not something that belongs baked into persisted config.
/// `#[serde(flatten)]` keeps the JSON shape identical to "just add fields to
/// `Site`" from the wire's perspective without touching `Site`'s own shape.
///
/// `is_wordpress` is served from an in-memory daemon cache refreshed on every
/// router rebuild (a mutation or a filesystem-watcher tick) rather than
/// detected fresh on every request - `ListSites` is polled every few seconds,
/// and re-statting every site's marker files on each poll doesn't scale with
/// site count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteEntry {
    /// The site itself - unchanged shape, still exactly what `Site`'s own
    /// serde impl produces.
    #[serde(flatten)]
    pub site: Site,
    /// Whether a `WordPress` marker (`wp-config.php`/`wp-load.php`) was found
    /// at the site's served root.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_wordpress: bool,
    /// The site's primary (canonical) domain FQDN, populated **only** when it
    /// differs from the default apex (`{name}.{tld}`). Omitted for an
    /// effectively-default site so the wire shape stays byte-identical to older
    /// clients, which synthesize `{name}.{tld}` from the TLD they already hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_domain: Option<String>,
    /// The site's full effective routable domain set as FQDNs, in router order
    /// (apex-first-then-added, so a non-apex primary is not necessarily first;
    /// identify the primary via `primary_domain`, not position). Populated
    /// **only** for an effectively-customized site (empty and omitted otherwise).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
    /// If another site claims this site's apex label, that other site's name.
    /// Omitted (`None`) when the apex is not shadowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apex_shadowed_by: Option<String>,
    /// The effective front-controller mode for this site (the daemon resolves
    /// the stored `Site::front_controller` override against the detected
    /// default, which needs the runtime `is_wordpress` fact). Always emitted so
    /// a client can render the toggle without re-deriving the default; defaults
    /// to `false` (direct execution) if an older daemon omits it.
    #[serde(default)]
    pub uses_front_controller: bool,
    /// Whether an `artisan` marker was found at the site's project root
    /// (`document_root`), i.e. it is a Laravel app - eligible to link a Reverb
    /// instance. Served from an in-memory daemon cache refreshed on router
    /// rebuild, exactly like `is_wordpress`. Additive: omitted (false) by older
    /// daemons and when the site is not Laravel.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_laravel: bool,
}

/// One whole-host reverse proxy (reply element of [`Response::Proxies`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyEntry {
    /// The proxy name (its `{name}.{tld}` host).
    pub name: String,
    /// The upstream URL (`http[s]://host:port`).
    pub target: String,
    /// Whether the proxy is served over HTTPS.
    pub secure: bool,
    /// The proxy's primary (canonical) domain FQDN, populated **only** when it
    /// differs from the default apex (`{name}.{tld}`). Omitted for an
    /// effectively-default proxy so the wire shape stays byte-identical to
    /// older clients, which synthesize `{name}.{tld}` from the TLD they already
    /// hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_domain: Option<String>,
    /// The proxy's full effective routable domain set as FQDNs, in router order
    /// (apex-first-then-added, so a non-apex primary is not necessarily first;
    /// identify the primary via `primary_domain`, not position). Populated
    /// **only** for an effectively-customized proxy (empty and omitted
    /// otherwise).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
}

/// One per-site path-prefix reverse-proxy rule (reply element of
/// [`Response::Proxies`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyRuleEntry {
    /// The site the rule attaches to (a linked site name or a parked
    /// document-root display label).
    pub site: String,
    /// The path prefix (e.g. `/app`).
    pub prefix: String,
    /// The upstream URL (`http[s]://host:port`).
    pub target: String,
}

/// One per-site path-prefix routing rule (reply element of
/// [`Response::Routes`]).
///
/// Deliberately separate from [`ProxyRuleEntry`] despite the identical field
/// shape: `orcker-ipc` is a byte-pinned contract, and coupling two features'
/// wire evolution to one struct would make either one hard to change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteRuleEntry {
    /// The site the rule attaches to (a linked site name or a parked
    /// document-root display label).
    pub site: String,
    /// The path prefix (e.g. `/api`).
    pub prefix: String,
    /// The target path, relative to the site's served root (e.g.
    /// `api/index.php`).
    pub target: String,
}
/// Machine-readable error category for [`Response::Error`].
///
/// Fail-closed on unknown variants from a newer daemon (no
/// `#[serde(other)]` catch-all) - an unknown code surfaces as
/// [`crate::IpcError::Decode`], which is the broader "version mismatch
/// signal" until a `Hello`/`Welcome` handshake lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    /// The requested site or resource does not exist.
    NotFound,
    /// A site with that name is already registered.
    AlreadyExists,
    /// The supplied path was rejected (does not exist, not a
    /// directory, outside an allowed root, etc.).
    InvalidPath,
    /// A service's configured port is already in use by another listener.
    PortInUse,
    /// A well-formed extension path was rejected because the `.so` failed its
    /// load-probe (wrong PHP version / ABI, a missing dependency, or a Zend
    /// extension registered without the Zend flag). Distinct from [`Self::InvalidPath`],
    /// which means the path itself was malformed.
    ExtensionLoadFailed,
    /// A requested port is already configured for another service instance
    /// (reserved, even if that instance is stopped). Distinct from
    /// [`Self::PortInUse`], which is a live bind conflict. Returned only on new
    /// requests (`AddService`), never on pre-existing ones, so older clients
    /// cannot receive it.
    PortReserved,
    /// The named site does not exist.
    SiteNotFound,
    /// The named site is not a Laravel app (no `artisan` marker), so it cannot
    /// host a per-site service like Reverb.
    SiteNotLaravel,
    /// The requested service type id is not known to the daemon.
    UnknownServiceType,
    /// A single-instance service type already has its one instance, or a per-site
    /// instance already exists for the chosen site.
    InstanceAlreadyExists,
    /// A LAN-only operation (e.g. `remote-setup`) was requested while LAN mode is
    /// off or its bootstrap listener isn't up. Returned only on LAN requests, so
    /// older clients never receive it.
    LanNotReady,
    /// A legacy (< 8.2) PHP version was used where it is not allowed: as the
    /// global default, or installed without the explicit `confirm_legacy`
    /// opt-in. Reachable from the pre-existing `InstallPhp` / `SetDefaultPhp`
    /// requests, so a pre-legacy client that names a legacy version can still
    /// provoke it and, lacking the variant, will surface it as an
    /// `IpcError::Decode` rather than a typed error.
    LegacyRestricted,
    /// Catch-all for daemon-side failures that don't fit a typed code.
    /// Expand this enum rather than overloading `Internal`.
    Internal,
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    // The rename-trap match arms are deliberately all `{}`; merging
    // them would collapse the per-variant check that catches Rust
    // variant renames.
    clippy::match_same_arms
)]
mod variant_name_pinning {
    use super::*;

    #[allow(dead_code)]
    fn pin_response(r: Response) {
        match r {
            Response::Pong => {}
            Response::Sites { .. } => {}
            Response::Ok => {}
            Response::Error { .. } => {}
            Response::Parked { .. } => {}
            Response::Info { .. } => {}
            Response::RemoteSetup { .. } => {}
            Response::Status { .. } => {}
            Response::EngineStatus { .. } => {}
            Response::Diagnoses { .. } => {}
            Response::DoctorFix { .. } => {}
            Response::Mails { .. } => {}
            Response::Mail { .. } => {}
            Response::Tools { .. } => {}
            Response::JobStarted { .. } => {}
            Response::JobProgress { .. } => {}
            Response::UpdateStatus { .. } => {}
            Response::Staged { .. } => {}
            Response::Tunnels { .. } => {}
            Response::NamedTunnels { .. } => {}
            Response::Groups { .. } => {}
            Response::BrowserTrust { .. } => {}
            Response::Proxies { .. } => {}
            Response::Routes { .. } => {}
        }
    }

    #[allow(dead_code)]
    fn pin_code(c: ErrorCode) {
        match c {
            ErrorCode::NotFound => {}
            ErrorCode::AlreadyExists => {}
            ErrorCode::InvalidPath => {}
            ErrorCode::PortInUse => {}
            ErrorCode::ExtensionLoadFailed => {}
            ErrorCode::PortReserved => {}
            ErrorCode::SiteNotFound => {}
            ErrorCode::SiteNotLaravel => {}
            ErrorCode::UnknownServiceType => {}
            ErrorCode::InstanceAlreadyExists => {}
            ErrorCode::LanNotReady => {}
            ErrorCode::LegacyRestricted => {}
            ErrorCode::Internal => {}
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn touch_every_variant() {
        pin_response(Response::Pong);
        pin_response(Response::Sites { sites: vec![] });
        pin_response(Response::Ok);
        pin_response(Response::Error {
            code: ErrorCode::Internal,
            message: "x".into(),
        });
        pin_response(Response::Parked {
            paths: vec!["/x".into()],
        });
        pin_response(Response::Info {
            dns_addr: "127.0.0.1:1053".parse().unwrap(),
            tld: "test".into(),
            ca_path: PathBuf::from("/x/ca.cert.pem"),
            ca_fingerprint: "ab".repeat(32),
            http_port: 8080,
            https_port: 8443,
            fallback_http: 8080,
            fallback_https: 8443,
            dns_port: 1053,
            lan_ip: None,
        });
        pin_response(Response::RemoteSetup {
            code: "abc123".into(),
            url: "http://192.168.1.42:7073/remote-setup?code=abc123".into(),
            script_sha256: "ab".repeat(32),
            expires_in_secs: 900,
        });
        pin_response(Response::Status {
            report: Box::new(crate::status::StatusReport {
                daemon_pid: 1,
                uptime_secs: 0,
                daemon_rss_bytes: None,
                tld: "test".into(),
                http: crate::status::PortStatus {
                    requested: 80,
                    bound: 8080,
                    fell_back: true,
                },
                https: crate::status::PortStatus {
                    requested: 443,
                    bound: 8443,
                    fell_back: true,
                },
                dns_addr: "127.0.0.1:1053".parse().unwrap(),
                ca: crate::status::CaStatus {
                    path: PathBuf::from("/x/ca.cert.pem"),
                    fingerprint: "ab".repeat(32),
                    trusted_system: Some(false),
                    browser_trust: None,
                },
                resolver_installed: None,
                port_redirect: None,
                foreign_web_listener: None,
                resolver_backup: None,
                sites: crate::status::SiteCounts::default(),
                load_avg: Some([100, 50, 25]),
                daemon_version: "9.9.9".into(),
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
        });
        pin_response(Response::EngineStatus {
            status: Box::new(crate::engine::DockerStatus {
                socket: crate::engine::SocketKind::Unsupported,
                reachable: false,
                engine_version: None,
                compose: crate::engine::ComposeStatus::Missing,
                problems: vec![],
            }),
        });
        pin_response(Response::Diagnoses {
            items: vec![crate::status::Diagnosis {
                code: crate::status::DiagnosisCode::AllGood,
                severity: crate::status::Severity::Ok,
                title: "x".into(),
                detail: "x".into(),
                remedy: None,
            }],
        });
        pin_response(Response::DoctorFix {
            report: crate::status::FixReport {
                performed: vec![],
                manual: vec![],
            },
        });
        pin_response(Response::Mails {
            mails: vec![crate::status::MailSummary {
                id: "000001".into(),
                from: "Example <hello@example.com>".into(),
                to: vec!["test@test.com".into()],
                subject: "Hi".into(),
                date_epoch: 1_700_000_000,
                read: false,
            }],
        });
        pin_response(Response::Mail {
            mail: Box::new(crate::status::MailDetail {
                id: "000001".into(),
                from: "Example <hello@example.com>".into(),
                to: vec!["test@test.com".into()],
                subject: "Hi".into(),
                date_epoch: 1_700_000_000,
                headers: vec![crate::status::MailHeader {
                    name: "Subject".into(),
                    value: "Hi".into(),
                }],
                html_body: Some("<p>Hi</p>".into()),
                text_body: Some("Hi".into()),
                attachments: vec![],
            }),
        });
        pin_response(Response::Tools {
            tools: vec![crate::status::ToolStatus {
                id: "node".into(),
                display_name: "Node.js".into(),
                installed: true,
                version: Some("v24.17.0".into()),
                binaries: vec!["node".into(), "npm".into(), "npx".into()],
                external: false,
                external_path: None,
            }],
        });
        pin_response(Response::JobStarted {
            job_id: "j1".into(),
        });
        pin_response(Response::JobProgress {
            state: crate::JobState::Running,
            phase: "Scaffolding".into(),
            log: vec!["line".into()],
            next_cursor: 1,
            error: None,
        });
        pin_response(Response::UpdateStatus {
            current: "2.0.2-rc.3".into(),
            latest_stable: Some("2.0.1".into()),
            latest_edge: Some("2.0.2-rc.3".into()),
            channel: crate::Channel::Stable,
            available: false,
            target: None,
            ahead_of_stable: true,
            source: crate::UpdateSource::Live,
            checked_at_epoch: Some(1_719_445_200),
        });
        pin_response(Response::Staged {
            path: "/x/Orcker.app.tar.gz".into(),
            version: "2.0.5".into(),
            kind: crate::StagedArtifact::AppTarGz,
        });
        pin_response(Response::Tunnels {
            tunnels: vec![crate::status::TunnelInfo {
                site: "app".into(),
                kind: crate::status::TunnelKind::Quick,
                state: crate::status::TunnelRunState::Running,
                url: Some("https://calm-river-1234.trycloudflare.com".into()),
                hostname: None,
            }],
            cloudflared: crate::status::CloudflaredStatus {
                installed: true,
                version: Some("2026.6.1".into()),
                source: Some(crate::status::CloudflaredSource::Managed),
                logged_in: false,
            },
        });
        pin_response(Response::NamedTunnels {
            tunnels: vec![crate::status::NamedTunnelMeta {
                name: "mysite".into(),
                uuid: "uuid-123".into(),
            }],
            sites: vec![crate::status::SiteHostname {
                site: "app".into(),
                hostname: "app.example.com".into(),
            }],
            zone: None,
        });
        pin_response(Response::Groups {
            order: vec!["Blog".into(), "Shop".into()],
            members: BTreeMap::from([("app".into(), "Blog".into())]),
        });
        pin_response(Response::Proxies {
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
                target: "http://127.0.0.1:8080".into(),
            }],
        });
        pin_response(Response::Routes {
            rules: vec![RouteRuleEntry {
                site: "portal".into(),
                prefix: "/api".into(),
                target: "api/index.php".into(),
            }],
        });
        for c in [
            ErrorCode::NotFound,
            ErrorCode::AlreadyExists,
            ErrorCode::InvalidPath,
            ErrorCode::PortInUse,
            ErrorCode::ExtensionLoadFailed,
            ErrorCode::PortReserved,
            ErrorCode::SiteNotFound,
            ErrorCode::SiteNotLaravel,
            ErrorCode::UnknownServiceType,
            ErrorCode::InstanceAlreadyExists,
            ErrorCode::LanNotReady,
            ErrorCode::LegacyRestricted,
            ErrorCode::Internal,
        ] {
            pin_code(c);
        }
    }
}
