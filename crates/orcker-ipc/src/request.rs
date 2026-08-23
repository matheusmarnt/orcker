//! Client → daemon request envelope.
//!
//! Internally tagged on `type`, `snake_case`. Treat this enum as a
//! published contract - add variants and fields additively, never
//! rename, and let `tests/wire_stability.rs` pin the byte-exact wire
//! shape.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// IMPORTANT: per-field serde renames are forbidden in this crate. Add
// new variants/fields additively; let rename_all handle casing. See
// README and the verification script's grep gate.
/// A request sent from a client (CLI or GUI) to the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Request {
    /// Liveness check.
    Ping,
    /// Enumerate every parked or linked site.
    ListSites,
    /// Register a parked directory. `path` is opaque to `orcker-ipc`;
    /// the daemon canonicalises before storing. Windows paths arrive
    /// with backslashes - that is fine.
    Park {
        /// The directory to park.
        path: PathBuf,
    },
    /// Link a site by name to a directory.
    Link {
        /// The site name (a single DNS label).
        name: String,
        /// The directory to link.
        path: PathBuf,
    },
    /// Remove a linked or parked site by name.
    Unlink {
        /// The site name to remove.
        name: String,
    },
    /// Enumerate the registered parked directory roots (including empty ones,
    /// which produce no sites and so never appear in [`Self::ListSites`]).
    ListParked,
    /// Un-park a directory root: remove it from the parked set and re-scan.
    /// Its parked sites disappear; linked sites are untouched.
    Unpark {
        /// The parked root to remove. Deliberately a `String`, not a
        /// `PathBuf`: the daemon stores parked roots as canonical
        /// `String`s (`config.parked.paths` is a `BTreeSet<String>`), and
        /// clients echo a value straight from [`super::Response::Parked`].
        /// Keeping it a `String` makes the removal an exact identity match -
        /// a `PathBuf` round-trip risks lossy normalisation. The daemon does
        /// **not** canonicalise it (so a folder deleted from disk is still
        /// removable).
        path: String,
    },
    /// Toggle whether a site is served over HTTPS.
    SetSecure {
        /// The site name.
        name: String,
        /// The desired HTTPS state.
        secure: bool,
    },
    /// Set or clear a site's served web root (the subdirectory served as the
    /// document root, e.g. `public` for Laravel).
    SetWebRoot {
        /// The site name.
        name: String,
        /// The served path. The daemon resolves it against the site's
        /// `document_root` (relative or absolute), validates containment, and
        /// stores the relative remainder. `None` resets the site to
        /// auto-detection.
        path: Option<String>,
    },
    /// Add a routable domain to a site (an exact host or a single-label
    /// wildcard, e.g. `api.foo.test` or `*.foo.test`). The `domain` is the full
    /// FQDN under the configured TLD; the daemon strips the TLD and validates.
    AddDomain {
        /// The site name to add the domain to.
        name: String,
        /// The full domain FQDN (under the configured TLD).
        domain: String,
    },
    /// Remove a routable domain from a site. Removing a site's last exact
    /// (non-wildcard) domain is refused.
    RemoveDomain {
        /// The site name to remove the domain from.
        name: String,
        /// The full domain FQDN to remove.
        domain: String,
    },
    /// Set a site's primary (canonical) domain, the address shown/opened and used
    /// for URL sync. Must be an exact domain; auto-added to the site's set if not
    /// already present.
    SetPrimaryDomain {
        /// The site name.
        name: String,
        /// The full domain FQDN to make primary.
        domain: String,
    },
    /// Reset a site's domains to the default (apex only), clearing any added,
    /// suppressed, or primary customisation.
    ResetDomains {
        /// The site name.
        name: String,
    },
    /// Register a whole-host reverse proxy (`name.test` → `url`).
    AddProxy {
        /// The proxy name (a single DNS label).
        name: String,
        /// The upstream URL, e.g. `http://localhost:8080` (validated by the daemon).
        url: String,
    },
    /// Remove a whole-host reverse proxy by name.
    RemoveProxy {
        /// The proxy name to remove.
        name: String,
    },
    /// Add a path-prefix reverse-proxy rule to an existing site
    /// (`site.test/prefix` → `url`), leaving all other paths served by PHP.
    AddProxyRule {
        /// The site the rule attaches to.
        site: String,
        /// The path prefix, e.g. `/app` (must begin with `/`).
        prefix: String,
        /// The upstream URL (validated by the daemon).
        url: String,
    },
    /// Remove a path-prefix reverse-proxy rule from a site.
    RemoveProxyRule {
        /// The site the rule is on.
        site: String,
        /// The path prefix to remove.
        prefix: String,
    },
    /// Enumerate whole-host proxies and per-site path-prefix rules.
    ListProxies,
    /// Fetch read-only daemon runtime facts (DNS address, TLD, CA path +
    /// fingerprint). Used by `orcker elevate` to drive the privileged helper.
    DaemonInfo,
    /// Fetch a read-only [`crate::StatusReport`] of daemon/proxy/DNS/PHP health.
    Status,
    /// Run the doctor checks and return the resulting diagnoses.
    Diagnose,
    /// Run the doctor checks, attempt the safe auto-fixes, and report what
    /// happened plus what still needs manual action.
    DoctorFix,
    /// Restart the daemon's own process in place (re-exec). The daemon replies
    /// `Ok` *before* tearing down; the connection then closes as it restarts.
    /// Unix-only.
    RestartDaemon,
    /// List captured emails (metadata only), newest first.
    ListMails,
    /// Fetch one captured email's full decoded content by id.
    GetMail {
        /// The email id (from [`super::Response::Mails`]).
        id: String,
    },
    /// Delete every captured email.
    ClearMails,
    /// Delete a specific set of captured emails by id (e.g. all the mail shown
    /// for one application). Unknown ids are ignored.
    DeleteMails {
        /// The email ids to delete.
        ids: Vec<String>,
    },
    /// Mark a set of captured emails as read. Unknown ids are ignored.
    MarkMailsRead {
        /// The email ids to mark read.
        ids: Vec<String>,
    },
    /// Set the mail-capture SMTP port. Takes effect on the next daemon
    /// start/restart (no implicit hot rebind), like [`Self::SetServicePort`].
    SetMailPort {
        /// The new loopback port (must be non-zero).
        port: u16,
    },
    /// Set the rootless HTTP/HTTPS fallback ports (the pair the daemon drops to
    /// when 80/443 can't bind without elevation). Both must be `>= 1024` and
    /// differ. Refused while a privileged-port redirect is active (it is pinned
    /// to the current ports). Takes effect on the next daemon restart.
    SetFallbackPorts {
        /// New rootless HTTP port (`>= 1024`).
        http: u16,
        /// New rootless HTTPS port (`>= 1024`).
        https: u16,
    },
    /// Set the embedded DNS responder port (`dns_port`). Must be non-zero. Takes
    /// effect on the next daemon restart (no implicit hot rebind). Changing it may
    /// require re-running the OS-resolver install so it points at the new port.
    SetDnsPort {
        /// The new loopback DNS port (must be non-zero).
        port: u16,
    },
    /// Enable or disable the mail-capture server. Takes effect on the next
    /// daemon start/restart.
    SetMailEnabled {
        /// The desired enabled state.
        enabled: bool,
    },
    /// List the installable dev tools (Composer, Node, Bun) with install status.
    ListTools,
    /// Download + install a dev tool's latest release into orcker's data dir and
    /// expose its commands on `PATH`. Idempotent (reinstalls/updates to latest).
    InstallTool {
        /// Tool id (`"composer"`, `"node"`, `"bun"`).
        tool: String,
    },
    /// Remove a dev tool's files and its `PATH` shims.
    UninstallTool {
        /// Tool id.
        tool: String,
    },
    /// Install a dev tool, streaming its output as a background job. Returns
    /// [`super::Response::JobStarted`] immediately; progress (and the install's
    /// stdout/stderr) is polled via [`Self::JobStatus`]. The streaming sibling of
    /// [`Self::InstallTool`].
    InstallToolStreamed {
        /// Tool id (`"composer"`, `"node"`, `"bun"`, `"laravel"`).
        tool: String,
    },
    /// Poll a running job's progress. `cursor` is the number of log lines the
    /// client has already seen; the daemon returns only newer lines plus the
    /// next cursor. Returns [`super::Response::JobProgress`].
    JobStatus {
        /// The job to poll.
        job_id: crate::JobId,
        /// How many log lines the client already holds.
        cursor: u64,
    },
    /// Request cancellation of a running job (kills its process tree). Returns
    /// [`super::Response::Ok`].
    JobCancel {
        /// The job to cancel.
        job_id: crate::JobId,
    },
    /// Check for an available Orcker self-update. Returns
    /// [`super::Response::UpdateStatus`] reporting the latest stable and edge
    /// versions, the active channel preference, and whether an update is
    /// available. Tolerant of network failure (the daemon serves its cache).
    CheckUpdate {
        /// Override the configured channel for this check only. `None` uses the
        /// persisted `update_channel`.
        channel: Option<crate::Channel>,
    },
    /// Return the **last persisted** self-update result without any network
    /// access - used to pre-fill the UI on load. Returns
    /// [`super::Response::UpdateStatus`] with `source = Cached` and
    /// `checked_at_epoch` set (or, if never checked, the running version with
    /// `checked_at_epoch = None`).
    CachedUpdateStatus,
    /// Persist the self-update channel preference. Returns
    /// [`super::Response::Ok`].
    SetUpdateChannel {
        /// The channel to make the new default.
        channel: crate::Channel,
    },
    /// Download + cryptographically verify the latest update artifact for this
    /// platform on `channel` (the configured channel when `None`). Blocking: the
    /// daemon returns [`super::Response::Staged`] with the on-disk path of the
    /// verified artifact (or [`super::Response::Error`]). The privileged
    /// install/swap is then performed by the applier, not the daemon.
    StageUpdate {
        /// Override the configured channel for this stage only.
        channel: Option<crate::Channel>,
    },
    /// Download + install the `cloudflared` binary as a streamed background job
    /// (the Cloudflare Tunnel integration's prerequisite). Replies
    /// [`super::Response::JobStarted`] immediately; progress is polled via
    /// [`Self::JobStatus`]. The streaming-only sibling of the dev-tool installers.
    InstallCloudflaredStreamed,
    /// Start a Quick Tunnel for a site, publishing it at a random
    /// `*.trycloudflare.com` URL. Replies [`super::Response::Tunnels`] with the
    /// live tunnel (including its URL once captured). Requires `cloudflared` to be
    /// installed.
    StartQuickTunnel {
        /// The site name to share.
        site: String,
    },
    /// Stop and tear down the tunnel for a site. No-op if none is running.
    StopTunnel {
        /// The site whose tunnel to stop.
        site: String,
    },
    /// Fetch the live tunnel state plus `cloudflared` install status. Returns
    /// [`super::Response::Tunnels`].
    TunnelStatus,
    /// Run the interactive Cloudflare account login (`cloudflared tunnel login`)
    /// as a streamed background job. The job log carries the one-time auth URL
    /// line for the GUI to open. Replies [`super::Response::JobStarted`]. Named
    /// Tunnels (Phase 2).
    CloudflaredLogin,
    /// Create a named tunnel on the logged-in account, recording its UUID.
    /// Replies [`super::Response::Ok`] (or `Error`). Requires a prior login.
    CreateNamedTunnel {
        /// The tunnel name to create.
        name: String,
    },
    /// List the named tunnels recorded locally. Returns
    /// [`super::Response::NamedTunnels`].
    ListNamedTunnels,
    /// Route a DNS hostname to a named tunnel (`cloudflared tunnel route dns`),
    /// creating the proxied CNAME on the user's Cloudflare zone. Account- and
    /// DNS-mutating; replies [`super::Response::Ok`].
    RouteTunnelDns {
        /// The tunnel name (or UUID) to route to.
        tunnel: String,
        /// The public hostname to create.
        hostname: String,
    },
    /// Set or clear a site's persisted public hostname (the named-tunnel
    /// mapping). Setting a hostname enables the site in the named tunnel;
    /// `None` removes (disables) it. Replies [`super::Response::Ok`].
    SetSiteTunnel {
        /// The site name.
        site: String,
        /// The public hostname, or `None` to remove the mapping.
        hostname: Option<String>,
    },
    /// (Re)start the single consolidated Named Tunnel serving every enabled site
    /// (one process, one config with one ingress rule per site). Returns
    /// [`super::Response::Tunnels`].
    StartNamedTunnel,
    /// Stop the consolidated Named Tunnel. Returns [`super::Response::Tunnels`].
    StopNamedTunnel,
    /// Delete a named tunnel from the Cloudflare account and forget it locally
    /// (stops the process, removes its credentials, and clears the persisted
    /// tunnel/site mappings). Account-mutating; replies [`super::Response::Ok`].
    DeleteNamedTunnel {
        /// The tunnel name to delete.
        name: String,
    },
    /// List the user-defined site groups (ordered) and per-site membership.
    /// Returns [`super::Response::Groups`]. Groups are a GUI organisational
    /// overlay and do not affect routing.
    ListGroups,
    /// Create a new site group, appended last in display order. Replies
    /// [`super::Response::Ok`]. Rejected if the name is empty, a duplicate
    /// (case-insensitive), or the reserved `Unallocated`.
    CreateGroup {
        /// The group display name to create.
        name: String,
    },
    /// Delete a site group. Its member sites fall back to the synthetic
    /// "Unallocated" bucket (their membership entries are dropped). Replies
    /// [`super::Response::Ok`].
    DeleteGroup {
        /// The group name to delete.
        name: String,
    },
    /// Replace the group display order. `order` must be an exact permutation of
    /// the existing group names. Replies [`super::Response::Ok`].
    SetGroupOrder {
        /// The full set of group names in the desired display order.
        order: Vec<String>,
    },
    /// Set or clear a site's group membership (a site belongs to at most one
    /// group). `Some(group)` must name an existing group; `None` moves the site
    /// to "Unallocated". Replies [`super::Response::Ok`].
    SetSiteGroup {
        /// The site name.
        site: String,
        /// The group to assign, or `None` to unassign.
        group: Option<String>,
    },
    /// Rename a site group, preserving its display position and moving every
    /// member with it. Replies [`super::Response::Ok`]. Rejected if `to` is
    /// empty, the reserved `Unallocated`, or a case-insensitive duplicate of a
    /// different group, or if `from` names no group.
    RenameGroup {
        /// The current group name.
        from: String,
        /// The new group name.
        to: String,
    },
    /// Enable or disable the proxy's symlink-escape protection (the global
    /// `symlink_protection` setting). When disabled, the proxy serves assets
    /// and resolves scripts reached via a symlink that resolves outside a
    /// site's document root. Takes effect immediately (no daemon restart) and
    /// is persisted to config.
    SetSymlinkProtection {
        /// `true` = protection on (block escapes); `false` = allow escapes.
        enabled: bool,
    },
    /// Override a site's front-controller mode. When enabled, every request
    /// funnels through the site-root `index.php`; when disabled, a named `.php`
    /// under the served root is executed directly. Persisted per site and
    /// applied on the next request. See
    /// [`orcker_core::Site::uses_front_controller`].
    SetFrontController {
        /// The site name.
        name: String,
        /// `true` = front-controller mode; `false` = direct script execution.
        enabled: bool,
    },
    /// Enable or disable the MCP server gate (whether `orcker mcp` serves tools
    /// to local AI agents). Persisted to config and reported back through
    /// [`crate::StatusReport::mcp_enabled`]; the daemon itself runs no MCP
    /// server, so this only gates `orcker mcp` sessions.
    SetMcpEnabled {
        /// `true` = agents may call Orcker's MCP tools; `false` = gated off.
        enabled: bool,
    },
    /// Enable or disable LAN exposure (serving `.test` sites to other devices on
    /// the network). Persisted to config; the actual re-bind happens on the
    /// daemon restart the CLI triggers next, so this is persist-only here.
    /// Reported back via [`crate::StatusReport::lan_enabled`].
    SetLanEnabled {
        /// `true` = expose to the LAN; `false` = loopback-only.
        enabled: bool,
    },
    /// Mint a one-time, expiring code for the remote-device bootstrap and return
    /// the setup URL + the CA fingerprint (for out-of-band verification). Only
    /// valid while LAN mode is up; otherwise the daemon returns an error.
    MintRemoteSetupCode,
    /// Install (or remove) the Orcker CA into the per-user **browser** NSS stores
    /// (`~/.pki/nssdb`, Firefox profiles, Snap/Flatpak). Runs unprivileged as
    /// the daemon's user; the CA PEM is read from disk daemon-side, so it is
    /// **not** carried on the wire. Reported back via [`crate::Response::BrowserTrust`].
    TrustBrowsers {
        /// `true` removes the CA from the NSS stores; `false` installs it.
        uninstall: bool,
    },
    /// Add a path-prefix **routing** rule to a site: URIs under `prefix` that
    /// match no real file are handled by `target`, a path relative to the site's
    /// served root. Unlike [`Self::AddProxyRule`], which forwards to an HTTP
    /// upstream, this resolves to a file inside the site's own tree - a nested
    /// front controller (`api/index.php`) or an SPA document (`index.html`).
    AddRouteRule {
        /// The site the rule attaches to.
        site: String,
        /// The path prefix, e.g. `/api` (must begin with `/`).
        prefix: String,
        /// The target path relative to the served root (validated by the
        /// daemon; never absolute and never containing `..`).
        target: String,
    },
    /// Remove a path-prefix routing rule from a site.
    RemoveRouteRule {
        /// The site the rule is on.
        site: String,
        /// The path prefix to remove.
        prefix: String,
    },
    /// Enumerate every site's path-prefix routing rules.
    ListRoutes,
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::match_same_arms
)]
mod variant_name_pinning {
    use super::*;
    use std::path::PathBuf;

    // Inline (not in tests/) so the #[non_exhaustive] enum matches
    // exhaustively: a renamed Rust variant fails this match at compile time.
    #[allow(dead_code, clippy::too_many_lines)]
    fn pin(r: Request) {
        match r {
            Request::Ping => {}
            Request::ListSites => {}
            Request::Park { .. } => {}
            Request::Link { .. } => {}
            Request::Unlink { .. } => {}
            Request::ListParked => {}
            Request::Unpark { .. } => {}
            Request::SetSecure { .. } => {}
            Request::SetWebRoot { .. } => {}
            Request::AddDomain { .. } => {}
            Request::RemoveDomain { .. } => {}
            Request::SetPrimaryDomain { .. } => {}
            Request::ResetDomains { .. } => {}
            Request::DaemonInfo => {}
            Request::Status => {}
            Request::Diagnose => {}
            Request::DoctorFix => {}
            Request::RestartDaemon => {}
            Request::ListMails => {}
            Request::GetMail { .. } => {}
            Request::ClearMails => {}
            Request::DeleteMails { .. } => {}
            Request::MarkMailsRead { .. } => {}
            Request::SetMailPort { .. } => {}
            Request::SetFallbackPorts { .. } => {}
            Request::SetDnsPort { .. } => {}
            Request::SetMailEnabled { .. } => {}
            Request::ListTools => {}
            Request::InstallTool { .. } => {}
            Request::UninstallTool { .. } => {}
            Request::InstallToolStreamed { .. } => {}
            Request::JobStatus { .. } => {}
            Request::JobCancel { .. } => {}
            Request::CheckUpdate { .. } => {}
            Request::CachedUpdateStatus => {}
            Request::SetUpdateChannel { .. } => {}
            Request::StageUpdate { .. } => {}
            Request::InstallCloudflaredStreamed => {}
            Request::StartQuickTunnel { .. } => {}
            Request::StopTunnel { .. } => {}
            Request::TunnelStatus => {}
            Request::CloudflaredLogin => {}
            Request::CreateNamedTunnel { .. } => {}
            Request::ListNamedTunnels => {}
            Request::RouteTunnelDns { .. } => {}
            Request::SetSiteTunnel { .. } => {}
            Request::StartNamedTunnel => {}
            Request::StopNamedTunnel => {}
            Request::DeleteNamedTunnel { .. } => {}
            Request::ListGroups => {}
            Request::CreateGroup { .. } => {}
            Request::DeleteGroup { .. } => {}
            Request::SetGroupOrder { .. } => {}
            Request::SetSiteGroup { .. } => {}
            Request::RenameGroup { .. } => {}
            Request::SetSymlinkProtection { .. } => {}
            Request::SetFrontController { .. } => {}
            Request::AddProxy { .. } => {}
            Request::RemoveProxy { .. } => {}
            Request::AddProxyRule { .. } => {}
            Request::RemoveProxyRule { .. } => {}
            Request::ListProxies => {}
            Request::SetMcpEnabled { .. } => {}
            Request::SetLanEnabled { .. } => {}
            Request::MintRemoteSetupCode => {}
            Request::TrustBrowsers { .. } => {}
            Request::AddRouteRule { .. } => {}
            Request::RemoveRouteRule { .. } => {}
            Request::ListRoutes => {}
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn touch_every_variant() {
        pin(Request::Ping);
        pin(Request::ListSites);
        pin(Request::Park {
            path: PathBuf::from("/x"),
        });
        pin(Request::Link {
            name: "x".into(),
            path: PathBuf::from("/x"),
        });
        pin(Request::Unlink { name: "x".into() });
        pin(Request::ListParked);
        pin(Request::Unpark { path: "/x".into() });
        pin(Request::SetSecure {
            name: "x".into(),
            secure: true,
        });
        pin(Request::SetWebRoot {
            name: "x".into(),
            path: Some("public".into()),
        });
        pin(Request::AddDomain {
            name: "foo".into(),
            domain: "api.foo.test".into(),
        });
        pin(Request::RemoveDomain {
            name: "foo".into(),
            domain: "api.foo.test".into(),
        });
        pin(Request::SetPrimaryDomain {
            name: "foo".into(),
            domain: "corp.test".into(),
        });
        pin(Request::ResetDomains { name: "foo".into() });
        pin(Request::DaemonInfo);
        pin(Request::Status);
        pin(Request::Diagnose);
        pin(Request::DoctorFix);
        pin(Request::RestartDaemon);
        pin(Request::ListMails);
        pin(Request::GetMail {
            id: "000001".into(),
        });
        pin(Request::ClearMails);
        pin(Request::DeleteMails {
            ids: vec!["000001".into()],
        });
        pin(Request::MarkMailsRead {
            ids: vec!["000001".into()],
        });
        pin(Request::SetMailPort { port: 2525 });
        pin(Request::SetFallbackPorts {
            http: 8080,
            https: 8443,
        });
        pin(Request::SetMailEnabled { enabled: true });
        pin(Request::ListTools);
        pin(Request::InstallTool {
            tool: "node".into(),
        });
        pin(Request::UninstallTool {
            tool: "node".into(),
        });
        pin(Request::InstallToolStreamed {
            tool: "laravel".into(),
        });
        pin(Request::JobStatus {
            job_id: "j1".into(),
            cursor: 0,
        });
        pin(Request::JobCancel {
            job_id: "j1".into(),
        });
        pin(Request::CheckUpdate {
            channel: Some(crate::Channel::Edge),
        });
        pin(Request::CachedUpdateStatus);
        pin(Request::SetUpdateChannel {
            channel: crate::Channel::Stable,
        });
        pin(Request::StageUpdate { channel: None });
        pin(Request::InstallCloudflaredStreamed);
        pin(Request::StartQuickTunnel { site: "app".into() });
        pin(Request::StopTunnel { site: "app".into() });
        pin(Request::TunnelStatus);
        pin(Request::CloudflaredLogin);
        pin(Request::CreateNamedTunnel {
            name: "mysite".into(),
        });
        pin(Request::ListNamedTunnels);
        pin(Request::RouteTunnelDns {
            tunnel: "mysite".into(),
            hostname: "app.example.com".into(),
        });
        pin(Request::SetSiteTunnel {
            site: "app".into(),
            hostname: Some("app.example.com".into()),
        });
        pin(Request::StartNamedTunnel);
        pin(Request::StopNamedTunnel);
        pin(Request::DeleteNamedTunnel {
            name: "mysite".into(),
        });
        pin(Request::ListGroups);
        pin(Request::CreateGroup {
            name: "Blog".into(),
        });
        pin(Request::DeleteGroup {
            name: "Blog".into(),
        });
        pin(Request::SetGroupOrder {
            order: vec!["Blog".into(), "Shop".into()],
        });
        pin(Request::SetSiteGroup {
            site: "app".into(),
            group: Some("Blog".into()),
        });
        pin(Request::RenameGroup {
            from: "Blog".into(),
            to: "Journal".into(),
        });
        pin(Request::SetSymlinkProtection { enabled: true });
        pin(Request::SetFrontController {
            name: "blog".to_owned(),
            enabled: true,
        });
        pin(Request::AddProxy {
            name: "reverb".to_owned(),
            url: "http://localhost:8080".to_owned(),
        });
        pin(Request::RemoveProxy {
            name: "reverb".to_owned(),
        });
        pin(Request::AddProxyRule {
            site: "app".to_owned(),
            prefix: "/app".to_owned(),
            url: "http://127.0.0.1:8080".to_owned(),
        });
        pin(Request::RemoveProxyRule {
            site: "app".to_owned(),
            prefix: "/app".to_owned(),
        });
        pin(Request::ListProxies);
        pin(Request::AddRouteRule {
            site: "portal".to_owned(),
            prefix: "/api".to_owned(),
            target: "api/index.php".to_owned(),
        });
        pin(Request::RemoveRouteRule {
            site: "portal".to_owned(),
            prefix: "/api".to_owned(),
        });
        pin(Request::ListRoutes);
        pin(Request::SetMcpEnabled { enabled: true });
        pin(Request::SetLanEnabled { enabled: true });
        pin(Request::MintRemoteSetupCode);
    }
}
