//! Pure command→request mapping and response→output rendering.
//!
//! Both directions are I/O-free and unit-tested: `to_request` validates
//! arguments client-side (so a bad name/version is a clean usage error before
//! any connect), and `render` turns a [`Response`] into stdout/stderr text and
//! an exit code.

use std::fmt::Write as _;

use orcker_core::{PhpVersion, Site, SiteKind};
use orcker_ipc::{
    Channel, CloudflaredStatus, ComposeStatus, Diagnosis, DockerStatus, FixReport, PortStatus,
    ProjectEntry, Request, Response, Severity, SiteEntry, SocketKind, StatusReport, ToolStatus,
    TunnelInfo, TunnelRunState, UpdateSource,
};

use crate::cli::{Command, MailAction, TunnelAction};
use crate::error::ClientError;

/// Map a parsed [`Command`] to the wire [`Request`], validating site names and
/// PHP versions client-side.
#[allow(clippy::too_many_lines)]
pub fn to_request(cmd: &Command) -> Result<Request, ClientError> {
    Ok(match cmd {
        Command::Ping => Request::Ping,
        Command::Sites => Request::ListSites,
        Command::Park { path } => Request::Park { path: path.clone() },
        Command::Unlink { name } => {
            validate_name(name)?;
            Request::Unlink { name: name.clone() }
        }
        Command::Unpark { path } => Request::Unpark {
            path: path.to_string_lossy().into_owned(),
        },
        Command::Install {
            target: crate::cli::InstallTarget::Tool { id },
        } => Request::InstallTool { tool: id.clone() },
        Command::Restart {
            target: crate::cli::RestartTarget::Daemon,
        } => Request::RestartDaemon,
        Command::Uninstall {
            target: Some(crate::cli::UninstallTarget::Tool { id }),
            ..
        } => Request::UninstallTool { tool: id.clone() },
        Command::Uninstall { target: None, .. } => {
            return Err(ClientError::Usage(
                "full uninstall is handled locally, not over IPC".to_owned(),
            ));
        }
        Command::Tools => Request::ListTools,
        Command::List {
            target: crate::cli::ListTarget::Parked,
        } => Request::ListParked,
        Command::Update { edge, stable, .. } => Request::CheckUpdate {
            channel: channel_from_flags(*edge, *stable),
        },
        Command::Domain { action } => domain_request(action)?,
        Command::Proxy { action } => proxy_request(action)?,
        Command::Route { action } => route_request(action)?,
        Command::Tunnel { action } => tunnel_request(action),
        Command::Mail { action } => match action {
            MailAction::List => Request::ListMails,
            MailAction::Show { id } => Request::GetMail { id: id.clone() },
            MailAction::Clear => Request::ClearMails,
        },
        // `lan status` is normally intercepted in `run()` for a LAN-focused
        // view; this arm keeps the mapping total and shares `Status`.
        Command::Status
        | Command::Lan {
            action: crate::cli::LanAction::Status,
        } => Request::Status,
        Command::Doctor { action: None } => Request::Diagnose,
        Command::Doctor {
            action: Some(crate::cli::DoctorAction::Fix),
        } => Request::DoctorFix,
        Command::Secure { name } => {
            validate_name(name)?;
            Request::SetSecure {
                name: name.clone(),
                secure: true,
            }
        }
        Command::Unsecure { name } => {
            validate_name(name)?;
            Request::SetSecure {
                name: name.clone(),
                secure: false,
            }
        }
        Command::Root { name, path, auto } => {
            validate_name(name)?;
            Request::SetWebRoot {
                name: name.clone(),
                path: if *auto { None } else { path.clone() },
            }
        }
        Command::FrontController { name, state } => {
            validate_name(name)?;
            Request::SetFrontController {
                name: name.clone(),
                enabled: state.is_on(),
            }
        }
        Command::Elevate { .. } | Command::Unelevate { .. } => {
            return Err(ClientError::Usage(
                "elevate/unelevate are handled locally, not over IPC".to_owned(),
            ));
        }
        Command::Path { .. } => {
            return Err(ClientError::Usage(
                "path is handled locally, not over IPC".to_owned(),
            ));
        }
        Command::Mcp => {
            return Err(ClientError::Usage(
                "mcp runs its own protocol loop, not a single IPC exchange".to_owned(),
            ));
        }
        Command::Link { .. } => {
            return Err(ClientError::Usage(
                "link is handled locally, not over IPC".to_owned(),
            ));
        }
        Command::Lan {
            action: crate::cli::LanAction::Enable,
        } => Request::SetLanEnabled { enabled: true },
        Command::Lan {
            action: crate::cli::LanAction::Disable,
        } => Request::SetLanEnabled { enabled: false },
        Command::RemoteSetup => Request::MintRemoteSetupCode,
    })
}

/// Map a `orcker domain <action>` to its wire request, validating the target name
/// and domain shape client-side. The target may be a site or a whole-host proxy
/// (the daemon resolves both), so it is checked with [`validate_target_name`].
/// `List` is handled locally (it needs the TLD to render default domains), so it
/// never reaches here.
fn domain_request(action: &crate::cli::DomainAction) -> Result<Request, ClientError> {
    use crate::cli::DomainAction;
    Ok(match action {
        DomainAction::List { .. } => {
            return Err(ClientError::Usage(
                "domain list is handled locally, not over IPC".to_owned(),
            ));
        }
        DomainAction::Add { site, domain } => {
            validate_target_name(site)?;
            validate_domain(domain)?;
            Request::AddDomain {
                name: site.clone(),
                domain: domain.clone(),
            }
        }
        DomainAction::Remove { site, domain } => {
            validate_target_name(site)?;
            validate_domain(domain)?;
            Request::RemoveDomain {
                name: site.clone(),
                domain: domain.clone(),
            }
        }
        DomainAction::Primary { site, domain } => {
            validate_target_name(site)?;
            validate_domain(domain)?;
            Request::SetPrimaryDomain {
                name: site.clone(),
                domain: domain.clone(),
            }
        }
        DomainAction::Reset { site } => {
            validate_target_name(site)?;
            Request::ResetDomains { name: site.clone() }
        }
    })
}

/// Map a `orcker proxy <action>` to its wire request. Arity distinguishes a
/// whole-host proxy from a path rule (see [`crate::cli::ProxyAction`]). The
/// upstream URL is validated by the daemon (authoritative); the client only
/// checks the name and, for a rule, that the prefix is absolute. A whole-host
/// proxy name may be dotted, a path rule's site name may not, so each arm
/// validates its own name rather than sharing one up-front check.
fn proxy_request(action: &crate::cli::ProxyAction) -> Result<Request, ClientError> {
    use crate::cli::ProxyAction;
    Ok(match action {
        ProxyAction::List => Request::ListProxies,
        ProxyAction::Add {
            first,
            second,
            third,
        } => {
            if let Some(url) = third {
                validate_name(first)?;
                validate_prefix(second)?;
                Request::AddProxyRule {
                    site: first.clone(),
                    prefix: second.clone(),
                    url: url.clone(),
                }
            } else {
                validate_target_name(first)?;
                Request::AddProxy {
                    name: first.clone(),
                    url: second.clone(),
                }
            }
        }
        ProxyAction::Remove { target, prefix } => {
            if let Some(prefix) = prefix {
                validate_name(target)?;
                validate_prefix(prefix)?;
                Request::RemoveProxyRule {
                    site: target.clone(),
                    prefix: prefix.clone(),
                }
            } else {
                validate_target_name(target)?;
                Request::RemoveProxy {
                    name: target.clone(),
                }
            }
        }
    })
}

/// Map a `orcker route <action>` to its wire request. The target is validated by
/// the daemon (authoritative, via `orcker_core::RouteRule`); the client only
/// checks the site name and that the prefix is absolute. `List` carries no
/// filter on the wire - the same full response is filtered client-side, so
/// there is only one IPC surface.
fn route_request(action: &crate::cli::RouteAction) -> Result<Request, ClientError> {
    use crate::cli::RouteAction;
    Ok(match action {
        RouteAction::List { .. } => Request::ListRoutes,
        RouteAction::Add {
            site,
            prefix,
            target,
        } => {
            validate_name(site)?;
            validate_prefix(prefix)?;
            Request::AddRouteRule {
                site: site.clone(),
                prefix: prefix.clone(),
                target: target.clone(),
            }
        }
        RouteAction::Remove { site, prefix } => {
            validate_name(site)?;
            validate_prefix(prefix)?;
            Request::RemoveRouteRule {
                site: site.clone(),
                prefix: prefix.clone(),
            }
        }
    })
}

/// Client-side check that a proxy rule prefix is an absolute path, for a clean
/// exit-2 error before connecting. The daemon is authoritative.
fn validate_prefix(prefix: &str) -> Result<(), ClientError> {
    if !prefix.starts_with('/') {
        return Err(ClientError::Usage(format!(
            "invalid path prefix {prefix:?}: must begin with '/'"
        )));
    }
    Ok(())
}

/// Light client-side shape check for a domain FQDN, for a clean exit-2 error
/// before connecting. The daemon is authoritative (it strips and validates
/// against the configured TLD); this only catches obvious typos: ASCII,
/// `[a-z0-9.*-]`, at least two labels, non-empty labels, and `*` only as the
/// leftmost label.
fn validate_domain(domain: &str) -> Result<(), ClientError> {
    let bad = |msg: &str| ClientError::Usage(format!("invalid domain {domain:?}: {msg}"));
    if domain.is_empty() {
        return Err(bad("must not be empty"));
    }
    let lowered = domain.to_ascii_lowercase();
    let trimmed = lowered.strip_suffix('.').unwrap_or(&lowered);
    let labels: Vec<&str> = trimmed.split('.').collect();
    if labels.len() < 2 {
        return Err(bad("must be a full domain including the TLD"));
    }
    for (i, label) in labels.iter().enumerate() {
        if label.is_empty() {
            return Err(bad("contains an empty label"));
        }
        if *label == "*" {
            if i != 0 {
                return Err(bad("'*' is only allowed as the leftmost label"));
            }
            continue;
        }
        if !label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(bad("labels may only contain [a-z0-9-] (or a leading '*')"));
        }
    }
    Ok(())
}

/// Map a `orcker tunnel <action>` to its wire request. `Install` and `Login` are
/// streamed jobs the CLI intercepts before this point; mapping them here keeps
/// the match total.
fn tunnel_request(action: &TunnelAction) -> Request {
    match action {
        TunnelAction::Install => Request::InstallCloudflaredStreamed,
        TunnelAction::Share { site } => Request::StartQuickTunnel { site: site.clone() },
        TunnelAction::Stop { site } => Request::StopTunnel { site: site.clone() },
        TunnelAction::Status => Request::TunnelStatus,
        TunnelAction::Login => Request::CloudflaredLogin,
        TunnelAction::Create { name } => Request::CreateNamedTunnel { name: name.clone() },
        TunnelAction::Delete { name } => Request::DeleteNamedTunnel { name: name.clone() },
        TunnelAction::List => Request::ListNamedTunnels,
        TunnelAction::Route { tunnel, hostname } => Request::RouteTunnelDns {
            tunnel: tunnel.clone(),
            hostname: hostname.clone(),
        },
        TunnelAction::SetHost {
            site,
            hostname,
            clear,
        } => Request::SetSiteTunnel {
            site: site.clone(),
            hostname: if *clear { None } else { hostname.clone() },
        },
        TunnelAction::Publish => Request::StartNamedTunnel,
        TunnelAction::Unpublish => Request::StopNamedTunnel,
    }
}

/// A site's effective domain set + primary FQDN, derived from a [`SiteEntry`] and
/// the configured `tld`. For an effectively-default site the daemon omits the
/// domain fields, so the primary/domains are synthesized as `{name}.{tld}`.
#[must_use]
pub fn site_domains(entry: &SiteEntry, tld: &str) -> (String, Vec<String>) {
    let default = format!("{}.{tld}", entry.site.name());
    let primary = entry
        .primary_domain
        .clone()
        .unwrap_or_else(|| default.clone());
    let domains = if entry.domains.is_empty() {
        vec![default]
    } else {
        entry.domains.clone()
    };
    (primary, domains)
}

/// Render `orcker domain list [site]`. `tld` comes from a `DaemonInfo` round-trip
/// (needed to show default `{name}.{tld}` domains). With `filter`, shows only
/// that site (exit 1 if absent).
#[must_use]
pub fn render_domains(
    sites: &[SiteEntry],
    tld: &str,
    filter: Option<&str>,
    json: bool,
) -> Rendered {
    let selected: Vec<&SiteEntry> = match filter {
        Some(f) => {
            let f = f.to_ascii_lowercase();
            sites.iter().filter(|e| e.site.name() == f).collect()
        }
        None => sites.iter().collect(),
    };

    if let Some(f) = filter {
        if selected.is_empty() {
            return Rendered::err(format!(
                "no site named {f:?} (a proxy's domains are listed by `orcker proxy list`)"
            ));
        }
    }

    if json {
        let items: Vec<_> = selected
            .iter()
            .map(|e| {
                let (primary, domains) = site_domains(e, tld);
                serde_json::json!({
                    "name": e.site.name(),
                    "primary": primary,
                    "domains": domains,
                    "apex_shadowed_by": e.apex_shadowed_by,
                })
            })
            .collect();
        let body = serde_json::to_string(&serde_json::json!({ "domains": items }))
            .unwrap_or_else(|_| "{\"domains\":[]}".to_owned());
        return Rendered::ok(body);
    }

    if selected.is_empty() {
        return Rendered::ok("No sites yet.".to_owned());
    }
    let mut out = String::new();
    for e in selected {
        let (primary, domains) = site_domains(e, tld);
        let list = domains
            .iter()
            .map(|d| {
                if *d == primary {
                    format!("{d} (primary)")
                } else {
                    d.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ = write!(out, "{}: {list}", e.site.name());
        if let Some(by) = &e.apex_shadowed_by {
            let _ = write!(out, "  [apex shadowed by {by}]");
        }
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
    Rendered::ok(out)
}

/// Validate a site name client-side by constructing a throwaway `Site` (the
/// document root is irrelevant - only the name is checked).
pub(crate) fn validate_name(name: &str) -> Result<(), ClientError> {
    Site::linked(name, "/", PhpVersion::new(8, 3))
        .map(|_| ())
        .map_err(|e| ClientError::Usage(format!("invalid site name {name:?}: {e}")))
}

/// Validate a name that may denote either a site or a whole-host proxy, for the
/// commands the daemon resolves against both namespaces.
///
/// Delegates to `orcker_core::validate_proxy_name`, which accepts every valid site
/// name plus the dotted names only a proxy can hold; the wrapper just rewords
/// the failure as a usage error.
pub(crate) fn validate_target_name(name: &str) -> Result<(), ClientError> {
    orcker_core::validate_proxy_name(name)
        .map(|_| ())
        .map_err(|e| ClientError::Usage(format!("invalid site or proxy name {name:?}: {e}")))
}

/// The result of rendering a response: text to print and a process exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// Text for stdout (may be empty).
    pub stdout: String,
    /// Text for stderr (may be empty).
    pub stderr: String,
    /// Process exit code.
    pub code: u8,
}

impl Rendered {
    fn ok(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            code: 0,
        }
    }

    fn err(stderr: String) -> Self {
        Self {
            stdout: String::new(),
            stderr,
            code: 1,
        }
    }
}

/// Render a daemon [`Response`] to stdout/stderr text + an exit code. With
/// `json`, prints the response as pretty JSON instead of a human table.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render(resp: &Response, json: bool) -> Rendered {
    let code = doctor_exit_code(resp);
    if json {
        let body = serde_json::to_string_pretty(resp)
            .unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {e}\"}}"));
        return Rendered {
            stdout: body,
            stderr: String::new(),
            code,
        };
    }
    match resp {
        Response::Pong => Rendered::ok("pong".to_owned()),
        Response::Ok => Rendered::ok("ok".to_owned()),
        Response::Sites { sites } => Rendered::ok(format_sites(sites, &[])),
        Response::Projects { projects } => Rendered::ok(format_projects(projects)),
        Response::Parked { paths } => Rendered::ok(format_parked(paths)),
        Response::Project {
            project,
            created,
            wrote_descriptor,
        } => Rendered::ok(format_linked_project(project, *created, *wrote_descriptor)),
        Response::Error { code: c, message } => Rendered::err(format!("error ({c:?}): {message}")),
        Response::RemoteSetup {
            code: _,
            url,
            script_sha256,
            expires_in_secs,
        } => Rendered::ok(format_remote_setup(url, script_sha256, *expires_in_secs)),
        Response::Status { report } => Rendered {
            stdout: format_status(report),
            stderr: String::new(),
            code,
        },
        Response::Diagnoses { items } => Rendered {
            stdout: format_doctor(items),
            stderr: String::new(),
            code,
        },
        Response::DoctorFix { report } => Rendered {
            stdout: format_fix(report),
            stderr: String::new(),
            code,
        },
        Response::Mails { mails } => Rendered::ok(format_mails(mails)),
        Response::Mail { mail } => Rendered::ok(format_mail(mail)),
        Response::Tools { tools } => Rendered::ok(format_tools(tools)),
        Response::Tunnels {
            tunnels,
            cloudflared,
        } => Rendered::ok(format_tunnels(tunnels, cloudflared)),
        Response::NamedTunnels {
            tunnels,
            sites,
            zone,
        } => Rendered::ok(format_named_tunnels(tunnels, sites, zone.as_deref())),
        Response::UpdateStatus {
            current,
            latest_stable,
            latest_edge,
            channel,
            available,
            target,
            ahead_of_stable,
            source,
            checked_at_epoch: _,
        } => Rendered::ok(format_update_status(
            current,
            latest_stable.as_deref(),
            latest_edge.as_deref(),
            *channel,
            *available,
            target.as_deref(),
            *ahead_of_stable,
            *source,
        )),
        Response::Proxies { proxies, rules } => Rendered::ok(format_proxies(proxies, rules)),
        Response::Routes { rules } => Rendered::ok(format_routes(rules, None)),
        _ => Rendered::err("unexpected response from daemon".to_owned()),
    }
}

/// Render `orcker route list [site]`. With `filter`, shows only that site's rules.
/// The filter is applied here rather than on the wire: `ListRoutes` returns
/// every site's rules and the CLI narrows them, so there is one IPC surface.
#[must_use]
pub fn render_routes(
    rules: &[orcker_ipc::RouteRuleEntry],
    filter: Option<&str>,
    json: bool,
) -> Rendered {
    if json {
        let selected: Vec<&orcker_ipc::RouteRuleEntry> = select_routes(rules, filter);
        let body = serde_json::to_string_pretty(&selected)
            .unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {e}\"}}"));
        return Rendered::ok(body);
    }
    Rendered::ok(format_routes(rules, filter))
}

fn select_routes<'a>(
    rules: &'a [orcker_ipc::RouteRuleEntry],
    filter: Option<&str>,
) -> Vec<&'a orcker_ipc::RouteRuleEntry> {
    match filter {
        Some(f) => {
            let f = f.to_ascii_lowercase();
            rules.iter().filter(|r| r.site == f).collect()
        }
        None => rules.iter().collect(),
    }
}

/// Render `orcker proxy list`: whole-host proxies then per-site path rules. A
/// proxy the daemon reports as customized gains an indented `domains:` line,
/// mirroring `orcker domain list`; an effectively-default proxy carries no domain
/// fields and renders as a single line.
fn format_proxies(
    proxies: &[orcker_ipc::ProxyEntry],
    rules: &[orcker_ipc::ProxyRuleEntry],
) -> String {
    use std::fmt::Write as _;
    if proxies.is_empty() && rules.is_empty() {
        return "no proxies configured".to_owned();
    }
    let mut out = String::new();
    if !proxies.is_empty() {
        out.push_str("Whole-host proxies:\n");
        for p in proxies {
            let scheme = if p.secure { "https" } else { "http" };
            let _ = writeln!(out, "  {} ({scheme}) -> {}", p.name, p.target);
            if !p.domains.is_empty() {
                let _ = writeln!(out, "    domains: {}", format_proxy_domains(p));
            }
        }
    }
    if !rules.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("Path rules:\n");
        for r in rules {
            let _ = writeln!(out, "  {} {} -> {}", r.site, r.prefix, r.target);
        }
    }
    out.trim_end().to_owned()
}

/// Comma-separated effective domain list for a proxy, with the primary marked -
/// the same shape [`render_domains`] uses for a site.
fn format_proxy_domains(proxy: &orcker_ipc::ProxyEntry) -> String {
    proxy
        .domains
        .iter()
        .map(|d| {
            if proxy.primary_domain.as_ref() == Some(d) {
                format!("{d} (primary)")
            } else {
                d.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render `orcker route list`: one row per rule, `site prefix -> target`.
fn format_routes(rules: &[orcker_ipc::RouteRuleEntry], filter: Option<&str>) -> String {
    use std::fmt::Write as _;
    let selected = select_routes(rules, filter);
    if selected.is_empty() {
        return match filter {
            Some(site) => format!("no routing rules configured for {site}"),
            None => "no routing rules configured".to_owned(),
        };
    }
    let mut out = String::from("Routing rules:\n");
    for r in selected {
        let _ = writeln!(out, "  {} {} -> {}", r.site, r.prefix, r.target);
    }
    out.trim_end().to_owned()
}

/// Process exit code for a response: `1` for an error or any `Fail`-severity
/// doctor finding, otherwise `0`. Pure; used by both the JSON and human paths.
#[must_use]
pub fn doctor_exit_code(resp: &Response) -> u8 {
    match resp {
        Response::Error { .. } => 1,
        Response::Diagnoses { items } => {
            u8::from(items.iter().any(|d| d.severity == Severity::Fail))
        }
        Response::DoctorFix { report } => {
            u8::from(report.manual.iter().any(|d| d.severity == Severity::Fail))
        }
        _ => 0,
    }
}

/// Renders the `orcker sites` table. The optional WORDPRESS and DOMAIN columns are
/// added only when at least one listed site needs them, so the common case's
/// table stays unchanged; full per-site domain lists live in `orcker domain list`.
fn format_sites(sites: &[SiteEntry], projects: &[ProjectEntry]) -> String {
    if sites.is_empty() && projects.is_empty() {
        return "no sites".to_owned();
    }
    if sites.is_empty() {
        return format_projects(projects);
    }
    let show_wordpress = sites.iter().any(|entry| entry.is_wordpress);
    let show_domain = sites
        .iter()
        .any(|e| e.primary_domain.is_some() || e.apex_shadowed_by.is_some());
    let mut out = String::from("NAME\tKIND\tPHP\tSECURE\tSERVED\tDOCROOT\tFRONT-CTRL");
    if show_domain {
        out.push_str("\tDOMAIN");
    }
    if show_wordpress {
        out.push_str("\tWORDPRESS");
    }
    for entry in sites {
        let s = &entry.site;
        let kind = match s.kind() {
            SiteKind::Parked => "parked",
            SiteKind::Linked => "linked",
        };
        let served = if s.web_subpath().as_os_str().is_empty() {
            "/".to_owned()
        } else {
            s.web_subpath().display().to_string()
        };
        let front = if entry.uses_front_controller {
            "index.php"
        } else {
            "direct"
        };
        let _ = write!(
            out,
            "\n{}\t{}\t{}\t{}\t{}\t{}\t{}",
            s.name(),
            kind,
            s.php(),
            s.secure(),
            served,
            s.document_root().display(),
            front
        );
        if show_domain {
            let domain = match (&entry.primary_domain, &entry.apex_shadowed_by) {
                (_, Some(by)) => format!("apex shadowed by {by}"),
                (Some(p), None) => p.clone(),
                (None, None) => "-".to_owned(),
            };
            let _ = write!(out, "\t{domain}");
        }
        if show_wordpress {
            let wp = if entry.is_wordpress { "yes" } else { "-" };
            let _ = write!(out, "\t{wp}");
        }
    }
    if !projects.is_empty() {
        out.push_str("\n\n");
        out.push_str(&format_projects(projects));
    }
    out
}

/// Renders `orcker sites`: the on-disk site table followed by the container
/// project table. In `--json` the two replies are merged into one object so a
/// script sees a single document.
#[must_use]
pub fn render_sites(sites: &[SiteEntry], projects: &[ProjectEntry], json: bool) -> Rendered {
    if json {
        let body = serde_json::to_string_pretty(&serde_json::json!({
            "sites": sites,
            "projects": projects,
        }))
        .unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {e}\"}}"));
        return Rendered {
            stdout: body,
            stderr: String::new(),
            code: 0,
        };
    }
    Rendered::ok(format_sites(sites, projects))
}

/// The reply to `orcker link`: the site's URL plus what the command actually
/// changed. A relink states that nothing changed (R5 idempotence).
fn format_linked_project(project: &ProjectEntry, created: bool, wrote_descriptor: bool) -> String {
    let scheme = if project.secure { "https" } else { "http" };
    let host = project.primary_domain.as_deref().unwrap_or(&project.name);
    let mut out = if created {
        format!(
            "linked {} -> {scheme}://{host} (upstream 127.0.0.1:{})",
            project.name, project.port
        )
    } else {
        format!(
            "{} is already linked on port {}; nothing changed",
            project.name, project.port
        )
    };
    if wrote_descriptor {
        let _ = write!(out, "\ncreated {}/orcker.yml", project.root.display());
    }
    out
}

/// The container-project table: one row per linked project with its loopback
/// port and the values its own `orcker.yml` declares (`-` when the file is
/// missing or unreadable).
fn format_projects(projects: &[ProjectEntry]) -> String {
    let mut out = String::from("PROJECT\tDOMAIN\tPORT\tSECURE\tPHP\tDB\tPRESET\tROOT");
    for p in projects {
        let _ = write!(
            out,
            "\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            p.name,
            p.primary_domain.as_deref().unwrap_or("-"),
            p.port,
            p.secure,
            p.php.map_or_else(|| "-".to_owned(), |v| v.to_string()),
            p.db.as_deref().unwrap_or("-"),
            p.preset.as_deref().unwrap_or("-"),
            p.root.display()
        );
    }
    out
}

fn format_parked(paths: &[String]) -> String {
    if paths.is_empty() {
        return "no parked folders".to_owned();
    }
    paths.join("\n")
}

/// The remote-device bootstrap instructions (hash-anchored flow). The SHA-256
/// printed here is the trust anchor - it travels by copy-paste, NOT over the
/// wire. The device fetches one self-contained installer (which embeds the CA)
/// over plain HTTP, the pasted command verifies its SHA-256 equals this value,
/// and only then runs it. Content integrity comes from the hash, so the plain
/// transport is safe.
///
/// `url` is the plain-HTTP installer URL (`http://<ip>:<port>/remote-setup?code=…`).
fn format_remote_setup(url: &str, script_sha256: &str, expires_in_secs: u64) -> String {
    let mins = expires_in_secs / 60;
    format!(
        "Run this on the OTHER device (needs sudo, curl, and openssl):\n\
         \n\
         \x20 curl -fsS --retry 3 -o orcker-setup.sh '{url}' && [ \"$(openssl dgst -sha256 -r orcker-setup.sh | cut -d' ' -f1)\" = \"{script_sha256}\" ] && sudo bash orcker-setup.sh\n\
         \n\
         How it works: it downloads Orcker's self-contained installer over HTTP, checks\n\
         its SHA-256 matches the one ON THIS SCREEN (the trust anchor), then runs it -\n\
         installing the embedded CA (system store plus Firefox/Chromium/Brave on Linux)\n\
         and pointing .test at this host. If the hash does not match, the '&&' chain\n\
         stops before sudo runs. The code expires in {mins} minutes and is single-use.\n\
         The hash is what makes this safe - do not skip it. To undo on the device\n\
         later:  sudo bash orcker-setup.sh uninstall"
    )
}

/// Flatten tab/CR/LF in a value so a folded or multi-line mail header can't
/// break the tab-separated `orcker mail list` table (the `--json` path needs no
/// such treatment - serde already escapes control bytes).
fn flatten_cell(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ")
}

/// Render `orcker mail list` - a tab-separated table of captured emails.
fn format_mails(mails: &[orcker_ipc::MailSummary]) -> String {
    if mails.is_empty() {
        return "no captured emails".to_owned();
    }
    let mut out = String::from("ID\tFROM\tSUBJECT");
    for m in mails {
        let subject = if m.subject.is_empty() {
            "(no subject)".to_owned()
        } else {
            flatten_cell(&m.subject)
        };
        let _ = write!(out, "\n{}\t{}\t{}", m.id, flatten_cell(&m.from), subject);
    }
    out
}

/// Render `orcker mail show <id>` - headers followed by the text body (falling
/// back to a note when only an HTML body is present).
fn format_mail(mail: &orcker_ipc::MailDetail) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "From:    {}", mail.from);
    let _ = writeln!(out, "To:      {}", mail.to.join(", "));
    let _ = writeln!(out, "Subject: {}", mail.subject);
    out.push('\n');
    match (&mail.text_body, &mail.html_body) {
        (Some(text), _) => out.push_str(text),
        (None, Some(_)) => out.push_str("(HTML-only message - open it in the GUI viewer)"),
        (None, None) => out.push_str("(empty message)"),
    }
    out
}

/// Render `orcker tools` as a tab-separated table (tool, status, commands, location).
fn format_tools(tools: &[ToolStatus]) -> String {
    if tools.is_empty() {
        return "no tools".to_owned();
    }
    let mut out = String::from("TOOL\tSTATUS\tCOMMANDS\tLOCATION");
    for t in tools {
        let status = if t.installed {
            t.version.as_deref().unwrap_or("installed")
        } else if t.external {
            "external"
        } else {
            "not installed"
        };
        let location = t.external_path.as_deref().unwrap_or("-");
        let _ = write!(
            out,
            "\n{}\t{}\t{}\t{}",
            t.id,
            status,
            t.binaries.join(","),
            location
        );
    }
    out
}

fn format_tunnels(tunnels: &[TunnelInfo], cloudflared: &CloudflaredStatus) -> String {
    let cf = if cloudflared.installed {
        cloudflared.version.as_deref().map_or_else(
            || "cloudflared: installed".to_owned(),
            |v| format!("cloudflared: {v}"),
        )
    } else {
        "cloudflared: not installed (run `orcker tunnel install`)".to_owned()
    };
    if tunnels.is_empty() {
        return format!("{cf}\nno active tunnels");
    }
    let mut out = format!("{cf}\n\nSITE\tSTATE\tURL");
    for t in tunnels {
        let state = match t.state {
            TunnelRunState::Running => "running",
            TunnelRunState::Failed => "failed",
            _ => "unknown",
        };
        let target = t.url.as_deref().or(t.hostname.as_deref()).unwrap_or("-");
        let _ = write!(out, "\n{}\t{}\t{}", t.site, state, target);
    }
    out
}

fn format_named_tunnels(
    tunnels: &[orcker_ipc::NamedTunnelMeta],
    sites: &[orcker_ipc::SiteHostname],
    zone: Option<&str>,
) -> String {
    let mut out = if tunnels.is_empty() {
        "no named tunnels".to_owned()
    } else {
        let mut s = String::from("NAME\tUUID");
        for t in tunnels {
            let _ = write!(s, "\n{}\t{}", t.name, t.uuid);
        }
        s
    };
    if let Some(zone) = zone {
        let _ = write!(out, "\n\nauthorized domain: {zone}");
    }
    if !sites.is_empty() {
        out.push_str("\n\nEXPOSED SITE\tHOSTNAME");
        for s in sites {
            let _ = write!(out, "\n{}\t{}", s.site, s.hostname);
        }
    }
    out
}

/// Render `orcker status`: the daemon report plus the Docker section.
///
/// `orcker status` is the one command that needs two responses, so it does not
/// go through [`render`]. Both output paths read the same pair of values: the
/// human block appends a `docker`/`compose` section, and `--json` nests the
/// `Status` response under `report` beside a `docker` object.
///
/// The exit code stays the daemon's: a stopped engine is reported, not an
/// error (R8).
#[must_use]
pub fn render_status(report: &StatusReport, docker: Option<&DockerStatus>, json: bool) -> Rendered {
    if json {
        let body = serde_json::json!({
            "type": "status",
            "report": report,
            "docker": docker.map(docker_json),
        });
        let text = serde_json::to_string_pretty(&body)
            .unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {e}\"}}"));
        return Rendered::ok(text);
    }
    let docker_block = docker.map_or_else(
        || "docker    unknown (the daemon did not answer the docker probe)\n".to_owned(),
        format_docker,
    );
    Rendered::ok(format!("{}{docker_block}", format_status(report)))
}

/// Split the `EngineStatus` exchange outcome into the section to render and a
/// note for stderr.
///
/// Pure so the branch is testable without a daemon. The section is simply
/// absent on any non-answer - `orcker status` reports, it does not fail (R8) -
/// but the *reason* is never guessed: this fork ships the CLI and the daemon
/// together, so a transport failure or an error response is far likelier than
/// version skew, and inventing "your daemon is old" would send the user after
/// the wrong problem. The real cause goes to stderr instead.
pub fn docker_section(
    outcome: Result<Response, ClientError>,
) -> (Option<DockerStatus>, Option<String>) {
    match outcome {
        Ok(Response::EngineStatus { status }) => (Some(*status), None),
        Ok(Response::Error { code, message }) => (
            None,
            Some(format!("docker status unavailable ({code:?}): {message}")),
        ),
        Ok(other) => (
            None,
            Some(format!(
                "docker status unavailable: unexpected response {other:?}"
            )),
        ),
        Err(e) => (None, Some(format!("docker status unavailable: {e}"))),
    }
}

/// The `docker` object used by `orcker status --json`.
///
/// Hoists the compose version out of [`ComposeStatus`] so a consumer reads
/// `docker.compose_version` without branching on the state tag, and keeps
/// `compose` for the full verdict.
fn docker_json(d: &DockerStatus) -> serde_json::Value {
    serde_json::json!({
        "socket": d.socket,
        "reachable": d.reachable,
        "engine_version": d.engine_version,
        "compose_version": compose_version(&d.compose),
        "compose": d.compose,
        "problems": d.problems,
    })
}

/// The version inside a [`ComposeStatus`], present whenever the plugin is
/// installed at all - including when it is too old to use.
fn compose_version(compose: &ComposeStatus) -> Option<&str> {
    match compose {
        ComposeStatus::Found { version } => Some(version),
        ComposeStatus::TooOld { found, .. } => Some(found),
        _ => None,
    }
}

/// The endpoint as it reads in the human block.
fn socket_label(socket: &SocketKind) -> &str {
    match socket {
        SocketKind::Unix { path } => path,
        SocketKind::Tcp { endpoint } => endpoint,
        _ => "no supported endpoint",
    }
}

/// Render a [`DockerStatus`] as human-readable lines, in the same
/// `label     value` column shape as the rest of `orcker status`.
fn format_docker(d: &DockerStatus) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let endpoint = socket_label(&d.socket);
    match (d.reachable, d.engine_version.as_deref()) {
        (true, Some(v)) => {
            let _ = writeln!(s, "docker    {v} ({endpoint})");
        }
        (true, None) => {
            let _ = writeln!(s, "docker    running, version unknown ({endpoint})");
        }
        (false, _) => {
            let _ = writeln!(s, "docker    not running ({endpoint})");
        }
    }
    match &d.compose {
        ComposeStatus::Found { version } => {
            let _ = writeln!(s, "compose   {version}");
        }
        ComposeStatus::TooOld { found, min } => {
            let _ = writeln!(s, "compose   {found} (older than the supported {min})");
        }
        _ => {
            let _ = writeln!(s, "compose   not installed");
        }
    }
    for p in &d.problems {
        let _ = writeln!(s, "          ! {}", p.message);
        let _ = writeln!(s, "            -> {}", p.hint);
    }
    s
}

/// Render a [`StatusReport`] as a human-readable block.
fn format_status(r: &StatusReport) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let rss = r
        .daemon_rss_bytes
        .map(|b| format!(", rss {}", fmt_bytes(b)))
        .unwrap_or_default();
    let _ = writeln!(
        s,
        "daemon    running (pid {}, up {}{})",
        r.daemon_pid,
        fmt_duration(r.uptime_secs),
        rss
    );
    let version = if r.daemon_version.is_empty() {
        "unknown"
    } else {
        &r.daemon_version
    };
    let _ = writeln!(s, "version   {version}");
    let _ = writeln!(s, "tld       .{}", r.tld);
    let redirected = r.port_redirect == Some(true);
    if let Some(u) = r.web_unbound {
        let _ = writeln!(
            s,
            "http      not serving - couldn't bind {} (run `orcker doctor`)",
            u.http
        );
        let _ = writeln!(s, "https     not serving - couldn't bind {}", u.https);
    } else {
        let _ = writeln!(s, "http      {}", fmt_port(r.http, redirected));
        let _ = writeln!(s, "https     {}", fmt_port(r.https, redirected));
    }
    if r.foreign_web_listener == Some(true) {
        let _ = writeln!(
            s,
            "ports     conflict: another process is using 80/443 (run `orcker doctor`)"
        );
    }
    if let Some(line) = redirect_stale_line(r) {
        let _ = writeln!(s, "{line}");
    }
    if let Some(port) = r.dns_unbound {
        let _ = writeln!(
            s,
            "dns       not resolving - couldn't bind port {port} (run `orcker doctor`)"
        );
    } else {
        let _ = writeln!(s, "dns       {}", r.dns_addr);
    }
    let _ = writeln!(
        s,
        "ca        trusted: {}  ({})",
        fmt_tristate(r.ca.trusted_system),
        r.ca.path.display()
    );
    let _ = writeln!(
        s,
        "resolver  installed: {}",
        fmt_tristate(r.resolver_installed)
    );
    if r.resolver_installed == Some(false) && r.web_unbound.is_none() {
        let port = if r.http.bound == 80 {
            String::new()
        } else {
            format!(":{}", r.http.bound)
        };
        let _ = writeln!(
            s,
            "          → not installed: reach sites at http://localhost{port}/~<name>.{}",
            r.tld
        );
    }
    if let Some([one, five, fifteen]) = r.load_avg {
        let _ = writeln!(
            s,
            "load      {} {} {}",
            fmt_centi(one),
            fmt_centi(five),
            fmt_centi(fifteen)
        );
    }
    let _ = writeln!(
        s,
        "sites     {} parked, {} linked, {} secured",
        r.sites.parked, r.sites.linked, r.sites.secured
    );

    s
}

/// Render the doctor findings as ✓/⚠/✗ lines with remedies.
fn format_doctor(items: &[Diagnosis]) -> String {
    use std::fmt::Write;
    if items.is_empty() {
        return "no findings".to_owned();
    }
    let mut s = String::new();
    for (i, d) in items.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        let _ = write!(s, "{} {}", severity_mark(d.severity), d.title);
        if !d.detail.is_empty() {
            let _ = write!(s, "\n    {}", d.detail);
        }
        if let Some(remedy) = &d.remedy {
            let _ = write!(s, "\n    → {remedy}");
        }
    }
    s
}

/// Render a [`FixReport`]: what was fixed, then what still needs attention.
fn format_fix(report: &FixReport) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    if report.performed.is_empty() {
        s.push_str("no automatic fixes were applicable");
    } else {
        s.push_str("applied fixes:");
        for f in &report.performed {
            let mark = if f.ok { "✓" } else { "✗" };
            let _ = write!(s, "\n  {mark} {}", f.message);
        }
    }
    if !report.manual.is_empty() {
        s.push_str("\n\nstill needs attention:");
        for d in &report.manual {
            let _ = write!(s, "\n  {} {}", severity_mark(d.severity), d.title);
            if let Some(remedy) = &d.remedy {
                let _ = write!(s, "\n      → {remedy}");
            }
        }
    }
    s
}

fn severity_mark(sev: Severity) -> &'static str {
    match sev {
        Severity::Ok => "✓",
        Severity::Warn => "⚠",
        Severity::Fail => "✗",
        _ => "•",
    }
}

fn fmt_port(p: PortStatus, redirected: bool) -> String {
    if p.fell_back {
        let tag = if redirected { "redirected" } else { "fallback" };
        format!("{} → {} ({tag})", p.requested, p.bound)
    } else {
        p.bound.to_string()
    }
}

fn fmt_tristate(b: Option<bool>) -> &'static str {
    match b {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

/// A `ports` status line when an installed macOS pf redirect targets a port the
/// daemon is no longer serving. The host (loopback) case takes precedence and
/// names both elevate commands; a LAN-only mismatch (host fine) names the LAN
/// step. `None` when nothing is stale. Mirrors the doctor's stale findings.
fn redirect_stale_line(r: &StatusReport) -> Option<String> {
    let mismatched =
        |t: orcker_ipc::PortRedirectTargets| t.http != r.http.bound || t.https != r.https.bound;
    if r.port_redirect_targets.is_some_and(mismatched) {
        return Some(
            "ports     stale redirect: pf targets a dead port (run `sudo orcker elevate ports`, \
             then `sudo orcker elevate lan` if LAN is on; see `orcker doctor`)"
                .to_owned(),
        );
    }
    if r.lan_enabled && r.lan_redirect_targets.is_some_and(mismatched) {
        return Some(
            "ports     stale LAN redirect: other devices reach a dead port (run \
             `sudo orcker elevate lan`; see `orcker doctor`)"
                .to_owned(),
        );
    }
    None
}

/// Render integer hundredths (e.g. `152`) as a decimal (`1.52`).
fn fmt_centi(c: u32) -> String {
    format!("{}.{:02}", c / 100, c % 100)
}

/// Human-readable uptime, coarse-grained.
fn fmt_duration(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    if h > 0 {
        format!("{h}h{m}m")
    } else if m > 0 {
        format!("{m}m{s}s")
    } else {
        format!("{s}s")
    }
}

/// Human-readable byte size (integer math; no float-cast lints).
fn fmt_bytes(b: u64) -> String {
    if b < 1024 {
        return format!("{b} B");
    }
    let kib = b / 1024;
    if kib < 1024 {
        return format!("{kib} KiB");
    }
    let mib_whole = kib / 1024;
    let mib_tenths = (kib % 1024) * 10 / 1024;
    format!("{mib_whole}.{mib_tenths} MiB")
}

/// The channel override implied by the `--edge` / `--stable` flags, or `None`
/// when neither is set and the saved preference applies.
pub fn channel_from_flags(edge: bool, stable: bool) -> Option<Channel> {
    if edge {
        Some(Channel::Edge)
    } else if stable {
        Some(Channel::Stable)
    } else {
        None
    }
}

/// Lowercase display name for a wire channel.
fn channel_str(c: Channel) -> &'static str {
    match c {
        Channel::Edge => "edge",
        _ => "stable",
    }
}

/// Render the `orcker update` report: current version, both channel latests, the
/// active channel, the availability status, and whether the figures are live or
/// cached. Both channel latests are always shown (per the feature spec).
#[allow(clippy::too_many_arguments)]
fn format_update_status(
    current: &str,
    latest_stable: Option<&str>,
    latest_edge: Option<&str>,
    channel: Channel,
    available: bool,
    target: Option<&str>,
    ahead_of_stable: bool,
    source: UpdateSource,
) -> String {
    let unknown = "unknown";
    let mut out = String::new();
    let _ = writeln!(out, "Current:       {current}");
    let _ = writeln!(out, "Latest stable: {}", latest_stable.unwrap_or(unknown));
    let _ = writeln!(out, "Latest edge:   {}", latest_edge.unwrap_or(unknown));
    let _ = writeln!(out, "Channel:       {}", channel_str(channel));
    let status = match (available, target) {
        (true, Some(t)) => format!("update available: {t}"),
        _ if ahead_of_stable => "up to date (on a pre-release ahead of stable)".to_owned(),
        _ => "up to date".to_owned(),
    };
    let _ = writeln!(out, "Status:        {status}");
    let src = match source {
        UpdateSource::Cached => "cached (offline - last known values)",
        _ => "live",
    };
    let _ = write!(out, "Source:        {src}");
    out
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    use orcker_ipc::ErrorCode;
    use std::path::PathBuf;

    #[test]
    fn maps_lan_and_remote_setup_commands() {
        use crate::cli::LanAction;
        assert_eq!(
            to_request(&Command::Lan {
                action: LanAction::Enable
            })
            .unwrap(),
            Request::SetLanEnabled { enabled: true }
        );
        assert_eq!(
            to_request(&Command::Lan {
                action: LanAction::Disable
            })
            .unwrap(),
            Request::SetLanEnabled { enabled: false }
        );
        assert_eq!(
            to_request(&Command::Lan {
                action: LanAction::Status
            })
            .unwrap(),
            Request::Status
        );
        assert_eq!(
            to_request(&Command::RemoteSetup).unwrap(),
            Request::MintRemoteSetupCode
        );
    }

    #[test]
    fn remote_setup_render_pins_the_script_hash_in_one_line() {
        let out = format_remote_setup(
            "http://192.168.1.42:7073/remote-setup?code=abc",
            &"ab".repeat(32),
            900,
        );
        assert!(out.contains(&"ab".repeat(32)), "script hash present: {out}");
        assert!(
            out.contains("openssl dgst -sha256 -r orcker-setup.sh"),
            "hashes the downloaded script: {out}"
        );
        assert!(
            out.contains("http://192.168.1.42:7073/remote-setup?code=abc"),
            "installer fetched over the plain-HTTP url: {out}"
        );
        assert!(
            !out.contains("https://") && !out.contains("--cacert"),
            "no HTTPS / CA-file hop remains: {out}"
        );
        let cmd = out
            .lines()
            .find(|l| l.trim_start().starts_with("curl "))
            .expect("has a curl command line");
        assert_eq!(
            cmd.matches("&&").count(),
            2,
            "the whole flow is one chained line (fetch && verify && run): {cmd}"
        );
        assert!(
            cmd.contains("sudo bash orcker-setup.sh"),
            "runs the verified script: {cmd}"
        );
    }

    #[test]
    fn renders_update_status_with_all_rows() {
        let resp = Response::UpdateStatus {
            current: "2.0.0".into(),
            latest_stable: Some("2.0.5".into()),
            latest_edge: Some("2.1.0-rc.1".into()),
            channel: Channel::Stable,
            available: true,
            target: Some("2.0.5".into()),
            ahead_of_stable: false,
            source: UpdateSource::Live,
            checked_at_epoch: None,
        };
        let out = render(&resp, false).stdout;
        assert!(out.contains("Current:       2.0.0"), "{out}");
        assert!(out.contains("Latest stable: 2.0.5"), "{out}");
        assert!(out.contains("Latest edge:   2.1.0-rc.1"), "{out}");
        assert!(out.contains("Channel:       stable"), "{out}");
        assert!(
            out.contains("Status:        update available: 2.0.5"),
            "{out}"
        );
        assert!(out.contains("Source:        live"), "{out}");
        assert_eq!(render(&resp, false).code, 0);
    }

    #[test]
    fn renders_update_status_cached_and_ahead_of_stable() {
        let resp = Response::UpdateStatus {
            current: "2.1.0-rc.3".into(),
            latest_stable: Some("2.0.5".into()),
            latest_edge: Some("2.1.0-rc.3".into()),
            channel: Channel::Stable,
            available: false,
            target: None,
            ahead_of_stable: true,
            source: UpdateSource::Cached,
            checked_at_epoch: None,
        };
        let out = render(&resp, false).stdout;
        assert!(out.contains("ahead of stable"), "{out}");
        assert!(out.contains("Source:        cached"), "{out}");
    }

    #[test]
    fn renders_parked_folders() {
        let empty = render(&Response::Parked { paths: vec![] }, false);
        assert_eq!(empty.stdout, "no parked folders");
        assert_eq!(empty.code, 0);

        let listed = render(
            &Response::Parked {
                paths: vec!["/srv/a".into(), "/srv/b".into()],
            },
            false,
        );
        assert!(listed.stdout.contains("/srv/a"));
        assert!(listed.stdout.contains("/srv/b"));
        assert_eq!(listed.code, 0);
    }

    #[test]
    fn maps_status_and_doctor_commands() {
        assert_eq!(to_request(&Command::Status).unwrap(), Request::Status);
        assert_eq!(
            to_request(&Command::Doctor { action: None }).unwrap(),
            Request::Diagnose
        );
        assert_eq!(
            to_request(&Command::Doctor {
                action: Some(crate::cli::DoctorAction::Fix)
            })
            .unwrap(),
            Request::DoctorFix
        );
    }

    #[test]
    fn domain_add_remove_primary_reset_map_to_requests() {
        use crate::cli::DomainAction;
        assert_eq!(
            to_request(&Command::Domain {
                action: DomainAction::Add {
                    site: "foo".into(),
                    domain: "api.foo.test".into(),
                },
            })
            .unwrap(),
            Request::AddDomain {
                name: "foo".into(),
                domain: "api.foo.test".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Domain {
                action: DomainAction::Primary {
                    site: "foo".into(),
                    domain: "corp.test".into(),
                },
            })
            .unwrap(),
            Request::SetPrimaryDomain {
                name: "foo".into(),
                domain: "corp.test".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Domain {
                action: DomainAction::Reset { site: "foo".into() },
            })
            .unwrap(),
            Request::ResetDomains { name: "foo".into() }
        );
    }

    #[test]
    fn domain_and_proxy_commands_accept_a_dotted_proxy_name() {
        use crate::cli::{DomainAction, ProxyAction};
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Add {
                    first: "api.account".into(),
                    second: "http://127.0.0.1:9011".into(),
                    third: None,
                },
            })
            .unwrap(),
            Request::AddProxy {
                name: "api.account".into(),
                url: "http://127.0.0.1:9011".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Remove {
                    target: "api.account".into(),
                    prefix: None,
                },
            })
            .unwrap(),
            Request::RemoveProxy {
                name: "api.account".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Domain {
                action: DomainAction::Add {
                    site: "account-dev".into(),
                    domain: "custom-domain.test".into(),
                },
            })
            .unwrap(),
            Request::AddDomain {
                name: "account-dev".into(),
                domain: "custom-domain.test".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Domain {
                action: DomainAction::Reset {
                    site: "api.account".into(),
                },
            })
            .unwrap(),
            Request::ResetDomains {
                name: "api.account".into(),
            }
        );
    }

    #[test]
    fn malformed_names_and_dotted_rule_targets_are_usage_errors() {
        use crate::cli::ProxyAction;
        let err = to_request(&Command::Proxy {
            action: ProxyAction::Add {
                first: "api..account".into(),
                second: "http://127.0.0.1:9011".into(),
                third: None,
            },
        })
        .unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)), "got: {err:?}");
        assert_eq!(
            err.to_string(),
            "invalid site or proxy name \"api..account\": proxy name \"api..account\" is invalid: \
             domain must not contain an empty label"
        );
        assert!(to_request(&Command::Proxy {
            action: ProxyAction::Add {
                first: "api.account".into(),
                second: "/app".into(),
                third: Some("http://127.0.0.1:8080".into()),
            },
        })
        .is_err());
        assert!(to_request(&Command::Proxy {
            action: ProxyAction::Remove {
                target: "api.account".into(),
                prefix: Some("/app".into()),
            },
        })
        .is_err());
    }

    #[test]
    fn domain_list_is_handled_locally() {
        use crate::cli::DomainAction;
        assert!(matches!(
            to_request(&Command::Domain {
                action: DomainAction::List { site: None },
            }),
            Err(ClientError::Usage(_))
        ));
    }

    #[test]
    fn validate_domain_accepts_and_rejects() {
        assert!(validate_domain("api.foo.test").is_ok());
        assert!(validate_domain("*.foo.test").is_ok());
        assert!(validate_domain("foo").is_err()); // needs a TLD
        assert!(validate_domain("foo.*.test").is_err()); // misplaced wildcard
        assert!(validate_domain("a_b.test").is_err()); // bad char
        assert!(validate_domain("foo..test").is_err()); // empty label
    }

    #[test]
    fn render_domains_unknown_site_filter_errors() {
        let r = render_domains(&[], "test", Some("ghost"), false);
        assert_eq!(r.code, 1);
        assert_eq!(
            r.stderr,
            "no site named \"ghost\" (a proxy's domains are listed by `orcker proxy list`)"
        );
    }

    #[test]
    fn fmt_port_distinguishes_fallback_from_redirect() {
        let fell_back = PortStatus {
            requested: 80,
            bound: 8080,
            fell_back: true,
        };
        assert_eq!(fmt_port(fell_back, false), "80 → 8080 (fallback)");
        assert_eq!(fmt_port(fell_back, true), "80 → 8080 (redirected)");
        let bound = PortStatus {
            requested: 80,
            bound: 80,
            fell_back: false,
        };
        assert_eq!(fmt_port(bound, true), "80");
    }

    #[test]
    fn renders_doctor_and_sets_exit_code_on_fail() {
        let warn_only = Response::Diagnoses {
            items: vec![Diagnosis {
                code: orcker_ipc::DiagnosisCode::CaNotTrusted,
                severity: Severity::Warn,
                title: "Local CA not trusted".into(),
                detail: "d".into(),
                remedy: Some("sudo orcker elevate trust".into()),
            }],
        };
        let r = render(&warn_only, false);
        assert_eq!(r.code, 0, "warn-only must not fail the exit code");
        assert!(r.stdout.contains("⚠ Local CA not trusted"));
        assert!(r.stdout.contains("→ sudo orcker elevate trust"));

        let with_fail = Response::Diagnoses {
            items: vec![Diagnosis {
                code: orcker_ipc::DiagnosisCode::CaNotTrusted,
                severity: Severity::Fail,
                title: "CA not trusted".into(),
                detail: "d".into(),
                remedy: None,
            }],
        };
        assert_eq!(render(&with_fail, false).code, 1);
        assert_eq!(render(&with_fail, true).code, 1);
    }

    #[test]
    fn renders_doctor_fix_report() {
        let resp = Response::DoctorFix {
            report: FixReport {
                performed: vec![orcker_ipc::FixResult {
                    code: orcker_ipc::DiagnosisCode::ResolverNotInstalled,
                    ok: true,
                    message: "installed the resolver".into(),
                }],
                manual: vec![Diagnosis {
                    code: orcker_ipc::DiagnosisCode::ResolverNotInstalled,
                    severity: Severity::Warn,
                    title: "Resolver not installed".into(),
                    detail: "d".into(),
                    remedy: Some("sudo orcker elevate resolver".into()),
                }],
            },
        };
        let r = render(&resp, false);
        assert_eq!(r.code, 0);
        assert!(r.stdout.contains("✓ installed the resolver"));
        assert!(r.stdout.contains("still needs attention"));
        assert!(r.stdout.contains("sudo orcker elevate resolver"));
    }

    #[test]
    fn fmt_bytes_is_human_readable() {
        assert_eq!(fmt_bytes(512), "512 B");
        assert_eq!(fmt_bytes(2048), "2 KiB");
        assert_eq!(fmt_bytes(3_200_000), "3.0 MiB");
    }

    #[test]
    fn json_rendering_is_valid_and_codes_match() {
        let ok = render(&Response::Ok, true);
        assert!(serde_json::from_str::<serde_json::Value>(&ok.stdout).is_ok());
        assert_eq!(ok.code, 0);

        let err = render(
            &Response::Error {
                code: ErrorCode::Internal,
                message: "boom".into(),
            },
            true,
        );
        let v: serde_json::Value = serde_json::from_str(&err.stdout).unwrap();
        assert_eq!(v["type"], "error");
        assert_eq!(err.code, 1);
    }

    #[test]
    fn maps_route_command() {
        use crate::cli::RouteAction;
        assert_eq!(
            to_request(&Command::Route {
                action: RouteAction::Add {
                    site: "portal".into(),
                    prefix: "/api".into(),
                    target: "api/index.php".into(),
                },
            })
            .unwrap(),
            Request::AddRouteRule {
                site: "portal".into(),
                prefix: "/api".into(),
                target: "api/index.php".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Route {
                action: RouteAction::Remove {
                    site: "portal".into(),
                    prefix: "/api".into(),
                },
            })
            .unwrap(),
            Request::RemoveRouteRule {
                site: "portal".into(),
                prefix: "/api".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Route {
                action: RouteAction::List { site: None },
            })
            .unwrap(),
            Request::ListRoutes
        );
        assert_eq!(
            to_request(&Command::Route {
                action: RouteAction::List {
                    site: Some("portal".into())
                },
            })
            .unwrap(),
            Request::ListRoutes,
            "the site filter is applied client-side, not on the wire"
        );
    }

    #[test]
    fn route_command_rejects_a_relative_prefix() {
        use crate::cli::RouteAction;
        assert!(to_request(&Command::Route {
            action: RouteAction::Add {
                site: "portal".into(),
                prefix: "api".into(),
                target: "api/index.php".into(),
            },
        })
        .is_err());
    }

    #[test]
    fn renders_route_list_with_and_without_a_filter() {
        let rules = vec![
            orcker_ipc::RouteRuleEntry {
                site: "portal".into(),
                prefix: "/api".into(),
                target: "api/index.php".into(),
            },
            orcker_ipc::RouteRuleEntry {
                site: "spa".into(),
                prefix: "/".into(),
                target: "index.html".into(),
            },
        ];

        let all = render_routes(&rules, None, false);
        assert_eq!(all.code, 0);
        assert!(all.stdout.contains("portal /api -> api/index.php"));
        assert!(all.stdout.contains("spa / -> index.html"));

        let one = render_routes(&rules, Some("Portal"), false);
        assert!(one.stdout.contains("portal /api -> api/index.php"));
        assert!(
            !one.stdout.contains("spa"),
            "the filter must be case-insensitive and exclusive"
        );

        let none = render_routes(&rules, Some("ghost"), false);
        assert_eq!(none.stdout, "no routing rules configured for ghost");

        let empty = render_routes(&[], None, false);
        assert_eq!(empty.stdout, "no routing rules configured");
    }

    #[test]
    fn maps_proxy_command() {
        use crate::cli::ProxyAction;
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Add {
                    first: "reverb".into(),
                    second: "http://localhost:8080".into(),
                    third: None,
                },
            })
            .unwrap(),
            Request::AddProxy {
                name: "reverb".into(),
                url: "http://localhost:8080".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Add {
                    first: "myapp".into(),
                    second: "/app".into(),
                    third: Some("http://127.0.0.1:8080".into()),
                },
            })
            .unwrap(),
            Request::AddProxyRule {
                site: "myapp".into(),
                prefix: "/app".into(),
                url: "http://127.0.0.1:8080".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Remove {
                    target: "reverb".into(),
                    prefix: None,
                },
            })
            .unwrap(),
            Request::RemoveProxy {
                name: "reverb".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::Remove {
                    target: "myapp".into(),
                    prefix: Some("/app".into()),
                },
            })
            .unwrap(),
            Request::RemoveProxyRule {
                site: "myapp".into(),
                prefix: "/app".into(),
            }
        );
        assert_eq!(
            to_request(&Command::Proxy {
                action: ProxyAction::List,
            })
            .unwrap(),
            Request::ListProxies
        );
        assert!(to_request(&Command::Proxy {
            action: ProxyAction::Add {
                first: "myapp".into(),
                second: "app".into(),
                third: Some("http://127.0.0.1:8080".into()),
            },
        })
        .is_err());
    }

    #[test]
    fn format_proxies_lists_domains_only_for_a_customized_proxy() {
        let plain = orcker_ipc::ProxyEntry {
            name: "reverb".into(),
            target: "http://127.0.0.1:8080".into(),
            secure: false,
            primary_domain: None,
            domains: vec![],
        };
        assert_eq!(
            format_proxies(std::slice::from_ref(&plain), &[]),
            "Whole-host proxies:\n  reverb (http) -> http://127.0.0.1:8080"
        );

        let customized = orcker_ipc::ProxyEntry {
            name: "account-dev".into(),
            target: "http://127.0.0.1:48087".into(),
            secure: true,
            primary_domain: Some("custom-domain.test".into()),
            domains: vec![
                "account-dev.test".into(),
                "custom-domain.test".into(),
                "*.account-dev.test".into(),
            ],
        };
        assert_eq!(
            format_proxies(&[plain, customized], &[]),
            "Whole-host proxies:\n  reverb (http) -> http://127.0.0.1:8080\n  \
             account-dev (https) -> http://127.0.0.1:48087\n    domains: account-dev.test, \
             custom-domain.test (primary), *.account-dev.test"
        );
    }

    /// A customized proxy whose primary is still its apex reports
    /// `primary_domain: None` (the daemon omits a primary equal to the apex), and
    /// the renderer has no TLD to rebuild that apex from, so no domain carries the
    /// marker. Pinned so the behaviour and `docs/reference/cli/proxies.md` agree.
    #[test]
    fn format_proxies_marks_nothing_when_the_primary_is_the_apex() {
        let apex_primary = orcker_ipc::ProxyEntry {
            name: "account-dev".into(),
            target: "http://127.0.0.1:48087".into(),
            secure: false,
            primary_domain: None,
            domains: vec!["account-dev.test".into(), "custom-domain.test".into()],
        };
        assert_eq!(
            format_proxies(std::slice::from_ref(&apex_primary), &[]),
            "Whole-host proxies:\n  account-dev (http) -> http://127.0.0.1:48087\n    \
             domains: account-dev.test, custom-domain.test"
        );
    }

    #[test]
    fn maps_every_mail_action() {
        use crate::cli::MailAction;
        assert_eq!(
            to_request(&Command::Mail {
                action: MailAction::List
            })
            .unwrap(),
            Request::ListMails
        );
        assert_eq!(
            to_request(&Command::Mail {
                action: MailAction::Show { id: "abc".into() }
            })
            .unwrap(),
            Request::GetMail { id: "abc".into() }
        );
        assert_eq!(
            to_request(&Command::Mail {
                action: MailAction::Clear
            })
            .unwrap(),
            Request::ClearMails
        );
    }

    #[test]
    fn root_rejects_bad_name() {
        match to_request(&Command::Root {
            name: "bad name".into(),
            path: None,
            auto: true,
        }) {
            Err(ClientError::Usage(_)) => {}
            other => panic!("expected Usage error, got {other:?}"),
        }
    }

    /// Human rendering of the responses a client sees most: `Pong`, `Ok`, an
    /// empty and a populated `Sites` listing, `Tools`, and an `Error` (which
    /// goes to stderr with exit code 1). Restored verbatim - every type it
    /// touches survives unchanged, including `SiteEntry`'s
    /// `uses_front_controller`, rendered as the FRONT-CTRL column.
    #[test]
    fn renders_human_responses_and_exit_codes() {
        assert_eq!(render(&Response::Pong, false).stdout, "pong");
        assert_eq!(render(&Response::Pong, false).code, 0);
        assert_eq!(render(&Response::Ok, false).code, 0);

        let empty = render(&Response::Sites { sites: vec![] }, false);
        assert_eq!(empty.stdout, "no sites");
        assert_eq!(empty.code, 0);

        let tools = render(
            &Response::Tools {
                tools: vec![
                    ToolStatus {
                        id: "node".into(),
                        display_name: "Node.js".into(),
                        installed: true,
                        version: Some("v24.17.0".into()),
                        binaries: vec!["node".into(), "npm".into(), "npx".into()],
                        external: false,
                        external_path: None,
                    },
                    ToolStatus {
                        id: "bun".into(),
                        display_name: "Bun".into(),
                        installed: false,
                        version: None,
                        binaries: vec!["bun".into(), "bunx".into()],
                        external: true,
                        external_path: Some("/opt/homebrew/bin/bun".into()),
                    },
                ],
            },
            false,
        );
        assert!(tools.stdout.contains("node"));
        assert!(tools.stdout.contains("v24.17.0"));
        assert!(tools.stdout.contains("npm"));
        assert!(tools.stdout.contains("external"));
        assert!(tools.stdout.contains("/opt/homebrew/bin/bun"));
        assert_eq!(tools.code, 0);

        let site = Site::linked("foo", "/srv/foo", PhpVersion::new(8, 3)).unwrap();
        let listed = render(
            &Response::Sites {
                sites: vec![SiteEntry {
                    site,
                    is_wordpress: false,
                    primary_domain: None,
                    domains: vec![],
                    apex_shadowed_by: None,
                    uses_front_controller: true,
                    is_laravel: false,
                }],
            },
            false,
        );
        assert!(listed.stdout.contains("foo"));
        assert!(listed.stdout.contains("linked"));
        assert!(listed.stdout.contains("8.3"));
        assert!(
            !listed.stdout.contains("WORDPRESS"),
            "no WORDPRESS column when nothing listed is WordPress"
        );
        assert!(
            listed.stdout.contains("FRONT-CTRL"),
            "front-controller column header"
        );
        assert!(
            listed.stdout.contains("index.php"),
            "uses_front_controller=true renders as index.php"
        );
        assert_eq!(listed.code, 0);

        let blog = Site::parked("blog", "/srv/blog", PhpVersion::new(8, 3)).unwrap();
        let with_wp = render(
            &Response::Sites {
                sites: vec![SiteEntry {
                    site: blog,
                    is_wordpress: true,
                    primary_domain: None,
                    domains: vec![],
                    apex_shadowed_by: None,
                    uses_front_controller: false,
                    is_laravel: false,
                }],
            },
            false,
        );
        assert!(with_wp.stdout.contains("WORDPRESS"));
        assert!(with_wp.stdout.contains("yes"));
        assert!(
            with_wp.stdout.contains("direct"),
            "uses_front_controller=false renders as direct"
        );

        let err = render(
            &Response::Error {
                code: ErrorCode::NotFound,
                message: "nope".into(),
            },
            false,
        );
        assert!(err.stdout.is_empty());
        assert!(err.stderr.contains("nope"));
        assert_eq!(err.code, 1);
    }

    /// The core command -> `Request` mapping. Every arm kept here had **no**
    /// `to_request` coverage at HEAD: `to_request(&Command::` appears zero times
    /// for Ping, Sites, Park, Unlink, Unpark, Tools, Secure, Unsecure,
    /// `FrontController`, Install, Uninstall, Restart and List.
    ///
    /// Scrubbed from the deleted original: its PHP arms (`Use`, `Set`/`Unset`
    /// php, `Install`/`Restart`/`Uninstall`/`List` php, `Update` php) went with
    /// the native runtime, and its two `Update` -> `CheckUpdate` arms are
    /// already pinned by `bare_update_stable_flag_overrides_channel` and
    /// `channel_from_flags_table`.
    #[test]
    #[allow(clippy::too_many_lines)]
    fn maps_each_command_to_its_request() {
        assert_eq!(to_request(&Command::Ping).unwrap(), Request::Ping);
        assert_eq!(to_request(&Command::Sites).unwrap(), Request::ListSites);
        assert_eq!(
            to_request(&Command::Park {
                path: PathBuf::from("/srv/sites")
            })
            .unwrap(),
            Request::Park {
                path: PathBuf::from("/srv/sites")
            }
        );
        assert_eq!(
            to_request(&Command::Unlink { name: "foo".into() }).unwrap(),
            Request::Unlink { name: "foo".into() }
        );
        assert_eq!(
            to_request(&Command::Unpark {
                path: PathBuf::from("/srv/sites")
            })
            .unwrap(),
            Request::Unpark {
                path: "/srv/sites".into()
            }
        );
        assert_eq!(
            to_request(&Command::List {
                target: crate::cli::ListTarget::Parked
            })
            .unwrap(),
            Request::ListParked
        );
        assert_eq!(to_request(&Command::Tools).unwrap(), Request::ListTools);
        assert_eq!(
            to_request(&Command::Install {
                target: crate::cli::InstallTarget::Tool { id: "node".into() }
            })
            .unwrap(),
            Request::InstallTool {
                tool: "node".into()
            }
        );
        assert_eq!(
            to_request(&Command::Uninstall {
                target: Some(crate::cli::UninstallTarget::Tool { id: "bun".into() }),
                yes: false
            })
            .unwrap(),
            Request::UninstallTool { tool: "bun".into() }
        );
        assert_eq!(
            to_request(&Command::Restart {
                target: crate::cli::RestartTarget::Daemon
            })
            .unwrap(),
            Request::RestartDaemon
        );
        assert_eq!(
            to_request(&Command::Secure { name: "foo".into() }).unwrap(),
            Request::SetSecure {
                name: "foo".into(),
                secure: true
            }
        );
        assert_eq!(
            to_request(&Command::Unsecure { name: "foo".into() }).unwrap(),
            Request::SetSecure {
                name: "foo".into(),
                secure: false
            }
        );
        assert_eq!(
            to_request(&Command::FrontController {
                name: "foo".into(),
                state: crate::cli::OnOff::On,
            })
            .unwrap(),
            Request::SetFrontController {
                name: "foo".into(),
                enabled: true
            }
        );
        assert_eq!(
            to_request(&Command::FrontController {
                name: "foo".into(),
                state: crate::cli::OnOff::Off,
            })
            .unwrap(),
            Request::SetFrontController {
                name: "foo".into(),
                enabled: false
            }
        );
        assert_eq!(
            to_request(&Command::Root {
                name: "foo".into(),
                path: Some("public".into()),
                auto: false,
            })
            .unwrap(),
            Request::SetWebRoot {
                name: "foo".into(),
                path: Some("public".into()),
            }
        );
        assert_eq!(
            to_request(&Command::Root {
                name: "foo".into(),
                path: Some("public".into()),
                auto: true,
            })
            .unwrap(),
            Request::SetWebRoot {
                name: "foo".into(),
                path: None,
            }
        );
        assert_eq!(
            to_request(&Command::Root {
                name: "foo".into(),
                path: None,
                auto: false,
            })
            .unwrap(),
            Request::SetWebRoot {
                name: "foo".into(),
                path: None,
            }
        );
    }

    /// Name validation happens client-side, before a socket is opened.
    /// Scrubbed from `rejects_bad_version_and_name_before_connect`: its version
    /// and php-setting arms drove `Command::Use` and `SetTarget::Php`, both
    /// deleted. `Root` is covered by `root_rejects_bad_name` and proxy names by
    /// `malformed_names_and_dotted_rule_targets_are_usage_errors`; `Unlink` and
    /// `Secure` had nothing.
    #[test]
    fn rejects_a_bad_site_name_before_connect() {
        match to_request(&Command::Unlink {
            name: "bad/name".into(),
        }) {
            Err(ClientError::Usage(_)) => {}
            other => panic!("expected Usage error, got {other:?}"),
        }
        match to_request(&Command::Secure {
            name: "bad name".into(),
        }) {
            Err(ClientError::Usage(_)) => {}
            other => panic!("expected Usage error, got {other:?}"),
        }
    }

    #[test]
    fn bare_update_stable_flag_overrides_channel() {
        assert_eq!(
            to_request(&Command::Update {
                yes: false,
                edge: false,
                stable: true,
                force: false,
            })
            .unwrap(),
            Request::CheckUpdate {
                channel: Some(Channel::Stable)
            }
        );
    }

    #[test]
    fn local_only_commands_are_usage_errors() {
        for cmd in [
            Command::Uninstall {
                target: None,
                yes: false,
            },
            Command::Elevate { target: None },
            Command::Unelevate { target: None },
            Command::Elevate {
                target: Some(crate::cli::ElevateTarget::Trust),
            },
            Command::Unelevate {
                target: Some(crate::cli::ElevateTarget::Resolver),
            },
            Command::Path {
                action: crate::cli::PathAction::Install,
            },
        ] {
            match to_request(&cmd) {
                Err(ClientError::Usage(_)) => {}
                other => panic!("expected Usage error for {cmd:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn channel_from_flags_table() {
        assert_eq!(channel_from_flags(true, false), Some(Channel::Edge));
        assert_eq!(channel_from_flags(false, true), Some(Channel::Stable));
        assert_eq!(channel_from_flags(false, false), None);
        assert_eq!(channel_from_flags(true, true), Some(Channel::Edge));
    }

    #[test]
    fn renders_mail_list() {
        let empty = render(&Response::Mails { mails: vec![] }, false);
        assert_eq!(empty.stdout, "no captured emails");

        let listed = render(
            &Response::Mails {
                mails: vec![
                    orcker_ipc::MailSummary {
                        id: "id1".into(),
                        from: "a@example.com".into(),
                        to: vec!["b@example.com".into()],
                        subject: "hello\tthere\nworld".into(),
                        date_epoch: 0,
                        read: false,
                    },
                    orcker_ipc::MailSummary {
                        id: "id2".into(),
                        from: "c@example.com".into(),
                        to: vec![],
                        subject: String::new(),
                        date_epoch: 0,
                        read: false,
                    },
                ],
            },
            false,
        );
        assert!(listed.stdout.contains("ID\tFROM\tSUBJECT"));
        assert!(listed
            .stdout
            .contains("id1\ta@example.com\thello there world"));
        assert!(listed.stdout.contains("id2\tc@example.com\t(no subject)"));
    }

    #[test]
    fn renders_mail_detail_body_variants() {
        let base = orcker_ipc::MailDetail {
            id: "id1".into(),
            from: "a@example.com".into(),
            to: vec!["b@example.com".into(), "c@example.com".into()],
            subject: "Hi".into(),
            date_epoch: 0,
            headers: vec![],
            html_body: None,
            text_body: Some("plain body".into()),
            attachments: vec![],
        };
        let text = render(
            &Response::Mail {
                mail: Box::new(base.clone()),
            },
            false,
        );
        assert!(text.stdout.contains("From:    a@example.com"));
        assert!(text
            .stdout
            .contains("To:      b@example.com, c@example.com"));
        assert!(text.stdout.contains("Subject: Hi"));
        assert!(text.stdout.contains("plain body"));

        let mut html_only = base.clone();
        html_only.text_body = None;
        html_only.html_body = Some("<p>hi</p>".into());
        assert!(render(
            &Response::Mail {
                mail: Box::new(html_only)
            },
            false
        )
        .stdout
        .contains("HTML-only message"));

        let mut empty = base;
        empty.text_body = None;
        empty.html_body = None;
        assert!(render(
            &Response::Mail {
                mail: Box::new(empty)
            },
            false
        )
        .stdout
        .contains("(empty message)"));
    }

    fn sample_report() -> orcker_ipc::StatusReport {
        orcker_ipc::StatusReport {
            daemon_pid: 4242,
            uptime_secs: 65,
            daemon_rss_bytes: Some(12_000_000),
            tld: "test".into(),
            http: PortStatus {
                requested: 80,
                bound: 8080,
                fell_back: true,
            },
            https: PortStatus {
                requested: 443,
                bound: 443,
                fell_back: false,
            },
            dns_addr: "127.0.0.1:1053".parse().unwrap(),
            ca: orcker_ipc::CaStatus {
                path: "/x/ca.cert.pem".into(),
                fingerprint: "ab".repeat(32),
                trusted_system: Some(false),
                browser_trust: None,
            },
            resolver_installed: None,
            port_redirect: None,
            foreign_web_listener: None,
            resolver_backup: None,
            sites: orcker_ipc::SiteCounts {
                parked: 2,
                linked: 1,
                secured: 1,
            },
            load_avg: Some([152, 48, 5]),
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
    fn renders_status_human_block() {
        let out = render(
            &Response::Status {
                report: Box::new(sample_report()),
            },
            false,
        );
        assert_eq!(out.code, 0);
        assert!(out.stdout.contains("pid 4242"));
        assert!(out.stdout.contains("version   2.0.1"));
        assert!(out.stdout.contains("80 → 8080 (fallback)"));
        assert!(out.stdout.contains("trusted: no"));
        assert!(out.stdout.contains("installed: unknown"));
        assert!(out.stdout.contains("1.52 0.48 0.05"));
    }

    #[test]
    fn status_shows_stale_redirect_lines() {
        let mut r = sample_report();
        r.http.bound = 8080;
        r.https.bound = 8443;

        r.port_redirect_targets = Some(orcker_ipc::PortRedirectTargets {
            http: 9090,
            https: 9443,
        });
        let out = format_status(&r);
        assert!(out.contains("stale redirect"), "{out}");
        assert!(out.contains("elevate ports"), "{out}");

        r.port_redirect_targets = Some(orcker_ipc::PortRedirectTargets {
            http: 8080,
            https: 8443,
        });
        r.lan_enabled = true;
        r.lan_redirect_targets = Some(orcker_ipc::PortRedirectTargets {
            http: 80,
            https: 443,
        });
        let out = format_status(&r);
        assert!(out.contains("stale LAN redirect"), "{out}");

        r.lan_redirect_targets = Some(orcker_ipc::PortRedirectTargets {
            http: 8080,
            https: 8443,
        });
        let out = format_status(&r);
        assert!(!out.contains("stale"), "{out}");
    }

    #[test]
    fn status_degraded_web_ports_shows_not_serving() {
        let mut r = sample_report();
        r.http = PortStatus {
            requested: 80,
            bound: 0,
            fell_back: true,
        };
        r.https = PortStatus {
            requested: 443,
            bound: 0,
            fell_back: true,
        };
        r.web_unbound = Some(orcker_ipc::UnboundWeb {
            http: 8080,
            https: 8443,
        });
        let out = format_status(&r);
        assert!(out.contains("not serving - couldn't bind 8080"), "{out}");
        assert!(out.contains("not serving - couldn't bind 8443"), "{out}");
        assert!(!out.contains("→ 0"), "{out}");
    }

    #[test]
    fn status_degraded_dns_shows_not_resolving() {
        let mut r = sample_report();
        r.dns_unbound = Some(1053);
        let out = format_status(&r);
        assert!(
            out.contains("not resolving - couldn't bind port 1053"),
            "{out}"
        );
        assert!(!out.contains("dns       127.0.0.1:1053"), "{out}");
    }

    #[test]
    fn status_shows_unknown_for_empty_daemon_version() {
        let mut report = sample_report();
        report.daemon_version = String::new();
        let out = render(
            &Response::Status {
                report: Box::new(report),
            },
            false,
        );
        assert!(
            out.stdout.contains("version   unknown"),
            "got: {}",
            out.stdout
        );
    }

    #[test]
    fn render_domains_marks_primary_and_shadow() {
        let e = SiteEntry {
            site: Site::linked("blog", "/srv/blog", PhpVersion::new(8, 3)).unwrap(),
            is_wordpress: false,
            primary_domain: Some("corp.test".into()),
            domains: vec!["corp.test".into(), "*.blog.test".into()],
            apex_shadowed_by: Some("shop".into()),
            uses_front_controller: false,
            is_laravel: false,
        };
        let r = render_domains(&[e], "test", None, false);
        assert!(r.stdout.contains("corp.test (primary)"));
        assert!(r.stdout.contains("*.blog.test"));
        assert!(r.stdout.contains("apex shadowed by shop"));
    }

    #[test]
    fn render_domains_synthesizes_default_domain() {
        let e = SiteEntry {
            site: Site::linked("foo", "/srv/foo", PhpVersion::new(8, 3)).unwrap(),
            is_wordpress: false,
            primary_domain: None,
            domains: vec![],
            apex_shadowed_by: None,
            uses_front_controller: false,
            is_laravel: false,
        };
        let r = render_domains(&[e], "test", None, false);
        assert!(r.stdout.contains("foo.test (primary)"));
    }

    fn sample_docker(reachable: bool) -> DockerStatus {
        DockerStatus {
            socket: SocketKind::Unix {
                path: "/var/run/docker.sock".into(),
            },
            reachable,
            engine_version: reachable.then(|| "27.3.1".to_owned()),
            compose: ComposeStatus::Found {
                version: "2.29.7".into(),
            },
            problems: vec![],
        }
    }

    #[test]
    fn status_human_output_carries_the_docker_section() {
        let out = render_status(&sample_report(), Some(&sample_docker(true)), false);
        assert_eq!(out.code, 0);
        assert!(out.stdout.contains("daemon    running"), "{}", out.stdout);
        assert!(out.stdout.contains("docker"), "{}", out.stdout);
        assert!(out.stdout.contains("27.3.1"), "{}", out.stdout);
        assert!(out.stdout.contains("2.29.7"), "{}", out.stdout);
    }

    #[test]
    fn status_json_output_carries_the_docker_section() {
        let out = render_status(&sample_report(), Some(&sample_docker(true)), true);
        assert_eq!(out.code, 0);
        let v: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
        assert_eq!(v["type"], "status");
        assert_eq!(v["report"]["daemon_pid"], 4242);
        assert_eq!(v["docker"]["engine_version"], "27.3.1");
        assert_eq!(v["docker"]["compose_version"], "2.29.7");
        assert!(v["docker"]["problems"].is_array());
    }

    /// A stopped engine is reported, with its hint, and still exits 0.
    #[test]
    fn status_with_the_engine_down_reports_and_exits_zero() {
        let docker = DockerStatus {
            socket: SocketKind::Unix {
                path: "/var/run/docker.sock".into(),
            },
            reachable: false,
            engine_version: None,
            compose: ComposeStatus::Missing,
            problems: vec![
                orcker_ipc::EngineProblem {
                    code: orcker_ipc::EngineProblemCode::EngineUnreachable,
                    message: "docker engine unreachable on /var/run/docker.sock".into(),
                    hint: "start Docker, then re-run `orcker status`".into(),
                },
                orcker_ipc::EngineProblem {
                    code: orcker_ipc::EngineProblemCode::ComposeMissing,
                    message: "the docker compose plugin is not installed".into(),
                    hint: "install the docker-compose-plugin package".into(),
                },
            ],
        };

        let human = render_status(&sample_report(), Some(&docker), false);
        assert_eq!(human.code, 0, "status reports, it does not fail");
        assert!(
            human.stdout.contains("docker engine unreachable"),
            "{}",
            human.stdout
        );
        assert!(
            human.stdout.contains("start Docker, then re-run"),
            "the hint must reach the user: {}",
            human.stdout
        );

        let json = render_status(&sample_report(), Some(&docker), true);
        assert_eq!(json.code, 0);
        let v: serde_json::Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(v["docker"]["engine_version"], serde_json::Value::Null);
        assert_eq!(v["docker"]["compose_version"], serde_json::Value::Null);
        assert_eq!(v["docker"]["problems"].as_array().unwrap().len(), 2);
        assert_eq!(
            v["docker"]["problems"][0]["hint"],
            "start Docker, then re-run `orcker status`"
        );
    }

    /// A daemon too old to answer `EngineStatus` must not blank the whole
    /// command: the report still renders, the docker section says unknown.
    #[test]
    fn status_without_a_docker_section_still_renders() {
        let human = render_status(&sample_report(), None, false);
        assert_eq!(human.code, 0);
        assert!(
            human.stdout.contains("daemon    running"),
            "{}",
            human.stdout
        );
        assert!(human.stdout.contains("unknown"), "{}", human.stdout);
        assert!(
            !human.stdout.contains("predates"),
            "the CLI cannot know why the section is missing, so it must not \
             blame version skew: {}",
            human.stdout
        );

        let json = render_status(&sample_report(), None, true);
        let v: serde_json::Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(v["docker"], serde_json::Value::Null);
    }

    /// Each way the `EngineStatus` exchange can fail drops the section *and*
    /// surfaces its real cause, rather than collapsing into one invented reason.
    #[test]
    fn a_failed_engine_status_exchange_names_its_own_cause() {
        let (section, note) = docker_section(Ok(Response::EngineStatus {
            status: Box::new(sample_docker(true)),
        }));
        assert_eq!(section, Some(sample_docker(true)));
        assert_eq!(note, None, "a good answer says nothing on stderr");

        let (section, note) = docker_section(Ok(Response::Error {
            code: orcker_ipc::ErrorCode::Internal,
            message: "engine probe failed".into(),
        }));
        assert!(section.is_none());
        let note = note.expect("an error response must explain itself");
        assert!(note.contains("engine probe failed"), "{note}");
        assert!(note.contains("Internal"), "{note}");

        let (section, note) = docker_section(Ok(Response::Ok));
        assert!(section.is_none());
        assert!(note.unwrap().contains("unexpected response"));

        let (section, note) = docker_section(Err(ClientError::DaemonUnreachable(
            "socket vanished".into(),
        )));
        assert!(section.is_none());
        let note = note.expect("a transport failure must explain itself");
        assert!(note.contains("socket vanished"), "{note}");
    }

    /// `compose_version` is present but flagged when the plugin is too old.
    #[test]
    fn status_json_reports_a_too_old_compose_with_its_minimum() {
        let mut docker = sample_docker(true);
        docker.compose = ComposeStatus::TooOld {
            found: "2.10.2".into(),
            min: "2.20.0".into(),
        };
        let out = render_status(&sample_report(), Some(&docker), true);
        let v: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
        assert_eq!(v["docker"]["compose_version"], "2.10.2");
        assert_eq!(v["docker"]["compose"]["state"], "too_old");
        assert_eq!(v["docker"]["compose"]["min"], "2.20.0");
    }
}
