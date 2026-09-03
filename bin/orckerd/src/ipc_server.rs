//! IPC accept loop + per-request dispatch.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::tokio::Stream as IpcStream;
use interprocess::local_socket::traits::tokio::Listener as _;
use interprocess::local_socket::traits::tokio::Stream as _;
use tokio::sync::watch;

use orcker_ipc::{
    read_message, write_message, ErrorCode, FrameDecoder, IpcError, Request, Response,
    DEFAULT_MAX_FRAME,
};

use crate::error::DaemonError;
use crate::state::DaemonState;
use crate::{link, mutate, startup};

/// Run the IPC accept loop until `shutdown_rx` resolves.
pub async fn run(
    listener: Listener,
    state: Arc<DaemonState>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            biased;
            _ = shutdown_rx.changed() => break,
            accepted = listener.accept() => {
                match accepted {
                    Ok(stream) => {
                        let state = state.clone();
                        tokio::spawn(handle_client(stream, state));
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "ipc accept failed");
                    }
                }
            }
        }
    }
}

async fn handle_client(stream: IpcStream, state: Arc<DaemonState>) {
    let (reader, writer) = stream.split();
    let mut reader = reader;
    let mut writer = writer;
    let mut decoder = FrameDecoder::new();
    loop {
        let req = match read_message::<_, Request>(&mut reader, &mut decoder).await {
            Ok(Some(r)) => r,
            Ok(None) => return,
            Err(e) => {
                if !matches!(e, IpcError::UnexpectedEof { .. }) {
                    tracing::debug!(error = %e, "ipc decode error");
                }
                return;
            }
        };
        let resp = match req {
            Request::InstallToolStreamed { tool } => {
                install_tool_streamed(tool, state.clone()).await
            }
            Request::InstallCloudflaredStreamed => {
                crate::tunnel::install_cloudflared_streamed(state.clone()).await
            }
            Request::CloudflaredLogin => crate::tunnel::named::login_streamed(state.clone()).await,
            Request::JobStatus { job_id, cursor } => state.jobs.poll(&job_id, cursor).await,
            Request::JobCancel { job_id } => state.jobs.cancel(&job_id).await,
            other => dispatch(other, &state).await,
        };
        if let Err(e) = write_message(&mut writer, &resp, DEFAULT_MAX_FRAME).await {
            tracing::debug!(error = %e, "ipc write error");
            return;
        }
        if state
            .restart_requested
            .load(std::sync::atomic::Ordering::Acquire)
        {
            use tokio::io::AsyncWriteExt as _;
            let _ = writer.flush().await;
            let _ = state.shutdown_tx.send_replace(true);
            return;
        }
    }
}

/// Builds the payload of a [`Response::Projects`] reply.
///
/// Each project's `orcker.yml` is read from its root on every call: that file
/// is the project's own source of truth (FR-024), so the daemon does not mirror
/// it into `orcker.toml`. A missing or invalid descriptor leaves the optional
/// fields empty rather than failing the whole listing. Cache it the way
/// `wordpress_sites` is cached if this listing's polling ever costs.
fn project_entries(
    config: &orcker_config::Config,
    router: &orcker_core::SiteRouter,
) -> Vec<orcker_ipc::ProjectEntry> {
    let mut entries: Vec<_> = config
        .projects
        .iter()
        .map(|p| {
            let yml = std::fs::read_to_string(p.root().join(orcker_config::orcker_yml::FILE_NAME))
                .ok()
                .and_then(|raw| orcker_config::OrckerYml::parse(&raw).ok());
            orcker_ipc::ProjectEntry {
                name: p.name().to_owned(),
                root: p.root().to_path_buf(),
                port: p.port(),
                secure: p.secure(),
                primary_domain: Some(project_domain(router, p.name(), config.tld.as_str())),
                schema_version: yml.as_ref().map(|y| y.schema_version),
                php: yml.as_ref().map(|y| y.php),
                db: yml.as_ref().map(|y| y.db.clone()),
                preset: yml.map(|y| y.preset),
            }
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// A project's primary FQDN, taken from the live router so a custom domain or a
/// non-default TLD is reflected, and falling back to the apex when the router
/// does not hold the project (a name collision dropped it).
fn project_domain(router: &orcker_core::SiteRouter, name: &str, tld: &str) -> String {
    site_entry_domains(router, name, tld)
        .0
        .unwrap_or_else(|| format!("{name}.{tld}"))
}

#[allow(clippy::too_many_lines)]
async fn dispatch(req: Request, state: &DaemonState) -> Response {
    match req {
        Request::Ping => Response::Pong,
        Request::ListSites => {
            // `is_wordpress` is a cheap lookup into `state.wordpress_sites`,
            // refreshed on every router rebuild (a mutation or an
            // fs-watcher tick) rather than detected fresh here - this
            // handler is polled every few seconds and must not re-stat every
            // site's marker files on each poll. See `wordpress_detect`.
            let router = state.router.read().await;
            let tld = router.config().tld().to_owned();
            let wordpress_sites = state.wordpress_sites.read().await;
            let laravel_sites = state.laravel_sites.read().await;
            let entries = router
                .iter()
                .map(|site| {
                    let name = site.name();
                    let is_wordpress = wordpress_sites.get(name).copied().unwrap_or(false);
                    let uses_front_controller = site.uses_front_controller(is_wordpress);
                    let (primary_domain, domains) = site_entry_domains(&router, name, &tld);
                    let apex_shadowed_by = router.apex_shadowed_by(name).map(str::to_owned);
                    let is_laravel = laravel_sites.get(name).copied().unwrap_or(false);
                    orcker_ipc::SiteEntry {
                        site: site.clone(),
                        is_wordpress,
                        primary_domain,
                        domains,
                        apex_shadowed_by,
                        uses_front_controller,
                        is_laravel,
                    }
                })
                .collect();
            Response::Sites { sites: entries }
        }
        Request::ListParked => Response::Parked {
            paths: state
                .config
                .lock()
                .await
                .parked
                .paths
                .iter()
                .cloned()
                .collect(),
        },
        Request::DaemonInfo => {
            let cfg = state.config.lock().await;
            Response::Info {
                dns_addr: state.dns_addr,
                tld: cfg.tld.as_str().to_owned(),
                ca_path: state.ca_path.clone(),
                ca_fingerprint: state.ca_fingerprint.to_hex(),
                http_port: state.http.bound,
                https_port: state.https.bound,
                fallback_http: cfg.ports.fallback_http,
                fallback_https: cfg.ports.fallback_https,
                dns_port: cfg.dns_port,
                lan_ip: state.lan_ip,
            }
        }
        Request::Park { .. }
        | Request::Link { .. }
        | Request::Unlink { .. }
        | Request::Unpark { .. }
        | Request::SetSecure { .. }
        | Request::SetWebRoot { .. }
        | Request::SetFrontController { .. }
        | Request::AddDomain { .. }
        | Request::RemoveDomain { .. }
        | Request::SetPrimaryDomain { .. }
        | Request::ResetDomains { .. }
        | Request::RemoveProxy { .. }
        | Request::RemoveProxyRule { .. }
        | Request::AddRouteRule { .. }
        | Request::RemoveRouteRule { .. } => handle_mutation(req, state).await,
        Request::AddProxy { ref url, .. } | Request::AddProxyRule { ref url, .. } => {
            if is_self_forward(url, &[state.http.bound, state.https.bound]) {
                self_forward_refusal()
            } else {
                handle_mutation(req, state).await
            }
        }
        Request::ListProjects => {
            let router = state.router.read().await;
            Response::Projects {
                projects: project_entries(&*state.config.lock().await, &router),
            }
        }
        Request::LinkProject { .. } => handle_link_project(req, state).await,
        Request::ListProxies => list_proxies(state).await,
        Request::ListRoutes => list_routes(state).await,
        Request::ListGroups => {
            let cfg = state.config.lock().await;
            Response::Groups {
                order: cfg.groups.order.clone(),
                members: cfg.groups.members.clone(),
            }
        }
        Request::CreateGroup { .. }
        | Request::DeleteGroup { .. }
        | Request::SetGroupOrder { .. }
        | Request::SetSiteGroup { .. }
        | Request::RenameGroup { .. } => handle_group_mutation(req, state).await,
        Request::Status => Response::Status {
            report: Box::new(build_status_report(state).await),
        },
        Request::EngineStatus => Response::EngineStatus {
            status: Box::new(state.engine_status.get().await),
        },
        Request::Diagnose => Response::Diagnoses {
            items: orcker_doctor::diagnose(
                &build_status_report(state).await,
                path_needs_setup(state),
            ),
        },
        Request::DoctorFix => run_doctor_fix(state).await,
        #[cfg(unix)]
        Request::RestartDaemon => {
            state
                .restart_requested
                .store(true, std::sync::atomic::Ordering::Release);
            Response::Ok
        }
        #[cfg(not(unix))]
        Request::RestartDaemon => Response::Error {
            code: ErrorCode::Internal,
            message: "daemon restart is not supported on this platform".into(),
        },
        Request::ListMails => Response::Mails {
            mails: state.mail_store.list().await,
        },
        Request::GetMail { id } => match state.mail_store.get(&id).await {
            Ok(Some(mail)) => Response::Mail {
                mail: Box::new(mail),
            },
            Ok(None) => Response::Error {
                code: ErrorCode::NotFound,
                message: format!("no captured mail with id {id}"),
            },
            Err(e) => internal(format!("mail read failed: {e}")),
        },
        Request::ClearMails => match state.mail_store.clear().await {
            Ok(()) => Response::Ok,
            Err(e) => internal(format!("mail clear failed: {e}")),
        },
        Request::DeleteMails { ids } => match state.mail_store.delete_many(&ids).await {
            Ok(()) => Response::Ok,
            Err(e) => internal(format!("mail delete failed: {e}")),
        },
        Request::MarkMailsRead { ids } => match state.mail_store.mark_read(&ids).await {
            Ok(()) => Response::Ok,
            Err(e) => internal(format!("mail mark-read failed: {e}")),
        },
        Request::SetMailPort { port } => set_mail_port(port, state).await,
        Request::SetFallbackPorts { http, https } => set_fallback_ports(http, https, state).await,
        Request::SetDnsPort { port } => set_dns_port(port, state).await,
        Request::SetMailEnabled { enabled } => set_mail_enabled(enabled, state).await,
        Request::SetSymlinkProtection { enabled } => set_symlink_protection(enabled, state).await,
        Request::SetMcpEnabled { enabled } => set_mcp_enabled(enabled, state).await,
        Request::SetLanEnabled { enabled } => set_lan_enabled(enabled, state).await,
        Request::MintRemoteSetupCode => mint_remote_setup_code(state).await,
        Request::TrustBrowsers { uninstall } => trust_browsers(uninstall, state).await,
        Request::ListTools => Response::Tools {
            tools: list_tools_with_external(state).await,
        },
        Request::InstallTool { tool } => install_tool(&tool, state).await,
        Request::UninstallTool { tool } => uninstall_tool(&tool, state).await,
        Request::CheckUpdate { channel } => {
            let dl = crate::download::ReqwestDownloader::new();
            crate::self_update::check_update(channel, state, &dl, orcker_update::UPDATE_PUBLIC_KEY)
                .await
        }
        Request::CachedUpdateStatus => crate::self_update::cached_update_status(state).await,
        Request::SetUpdateChannel { channel } => {
            crate::self_update::set_update_channel(channel, state).await
        }
        Request::StageUpdate { channel } => {
            let dl = crate::download::ReqwestDownloader::new();
            crate::self_update::stage_update(channel, state, &dl, orcker_update::UPDATE_PUBLIC_KEY)
                .await
        }
        Request::StartQuickTunnel { site } => crate::tunnel::start_quick_tunnel(&site, state).await,
        Request::StopTunnel { site } => crate::tunnel::stop_tunnel(&site, state).await,
        Request::TunnelStatus => crate::tunnel::tunnel_status(state).await,
        Request::CreateNamedTunnel { name } => crate::tunnel::named::create(&name, state).await,
        Request::ListNamedTunnels => crate::tunnel::named::list(state).await,
        Request::RouteTunnelDns { tunnel, hostname } => {
            crate::tunnel::named::route_dns(&tunnel, &hostname, state).await
        }
        Request::SetSiteTunnel { site, hostname } => {
            crate::tunnel::named::set_site_hostname(&site, hostname.as_deref(), state).await
        }
        Request::StartNamedTunnel => crate::tunnel::named::start(state).await,
        Request::StopNamedTunnel => crate::tunnel::named::stop(state).await,
        Request::DeleteNamedTunnel { name } => crate::tunnel::named::delete(&name, state).await,
        _ => Response::Error {
            code: ErrorCode::Internal,
            message: "unsupported request".into(),
        },
    }
}

/// Compute a site's `SiteEntry` domain fields. Returns `(primary_domain,
/// domains)`, both **omitted** (`None`/empty) for an effectively-default site
/// (apex only, primary = apex) so the wire shape stays byte-identical to older
/// clients. For a customized site, `domains` is the full effective set as FQDNs
/// in router order (apex-first-then-added, so a non-apex primary is *not*
/// necessarily first) and `primary_domain` is set only when the primary differs
/// from the default apex. Clients identify the primary by matching
/// `primary_domain`, not by position.
fn site_entry_domains(
    router: &orcker_core::SiteRouter,
    name: &str,
    tld: &str,
) -> (Option<String>, Vec<String>) {
    let apex = orcker_core::Domain::apex(name);
    let effective = router.effective_domains(name).unwrap_or(&[]);
    let primary = router.primary_domain(name);

    let is_default =
        effective.len() == 1 && effective.first() == Some(&apex) && primary == Some(&apex);
    if is_default {
        return (None, Vec::new());
    }

    let domains = effective.iter().map(|d| d.to_fqdn(tld)).collect();
    let primary_domain = match primary {
        Some(p) if *p != apex => Some(p.to_fqdn(tld)),
        _ => None,
    };
    (primary_domain, domains)
}

/// Whether a dev tool is installed but Orcker's `{data}/bin` isn't on the user's
/// PATH yet (no managed block in any known shell rc) - drives the doctor's
/// [`orcker_ipc::DiagnosisCode::BinDirNotOnPath`] warning. `Some(false)` when no
/// tool is installed or PATH is already wired; `None` when undeterminable
/// (non-Unix, or `$HOME` unset). Computed on demand from the `Diagnose` handler,
/// not on the per-poll status path. The cover/pcov shims alone don't count - the
/// gate is an actual installed dev tool.
fn path_needs_setup(state: &DaemonState) -> Option<bool> {
    #[cfg(not(unix))]
    {
        let _ = state;
        None
    }
    #[cfg(unix)]
    {
        use orcker_platform::pure::shell_profile::{self, rc_relpaths, HostOs, Shell};

        let any_tool = crate::tools::list_status(&state.dirs)
            .iter()
            .any(|t| t.installed);
        if !any_tool {
            return Some(false);
        }
        let home = std::env::var_os("HOME")
            .filter(|h| !h.is_empty())
            .map(std::path::PathBuf::from)?;
        let os = if cfg!(target_os = "macos") {
            HostOs::MacOs
        } else {
            HostOs::Linux
        };
        let present = [Shell::Zsh, Shell::Bash, Shell::Fish, Shell::Posix]
            .into_iter()
            .flat_map(|s| rc_relpaths(s, os))
            .any(|rel| {
                std::fs::read_to_string(home.join(rel))
                    .is_ok_and(|c| shell_profile::contains_block(&c))
            });
        Some(!present)
    }
}

/// Assemble a read-only [`orcker_ipc::StatusReport`].
///
/// Lock discipline: each guard is acquired, drained into owned data, and dropped
/// before the next acquisition - never two at once, never a guard held across an
/// `.await` that touches another lock. Mirrors the hazard documented in
/// `handle_mutation`.
/// Resident-set size for each of `pids`, gathered in a single `spawn_blocking`.
///
/// `SystemMetrics::rss_bytes` shells out to `ps` on macOS (fork+exec+wait) -
/// genuinely blocking I/O, unlike every other field of a `StatusReport`. Doing
/// it once off-executor, rather than synchronously per pid inline, keeps a
/// tokio worker thread from being tied up once per installed PHP version plus
/// once for the daemon itself on every `Request::Status`/`Request::Diagnose`
/// (the GUI polls this every ~6s), which under load could starve the whole
/// worker pool. Missing pids are simply absent from the returned map.
async fn collect_rss_by_pid(
    metrics: orcker_platform::ActiveSystemMetrics,
    pids: Vec<u32>,
) -> std::collections::HashMap<u32, u64> {
    use orcker_platform::SystemMetrics;
    tokio::task::spawn_blocking(move || {
        let mut out = std::collections::HashMap::new();
        for pid in pids {
            if let Some(rss) = metrics.rss_bytes(pid) {
                out.insert(pid, rss);
            }
        }
        out
    })
    .await
    .unwrap_or_default()
}

/// Convert domain collisions (live sites plus persisted `[domains]` deltas) into
/// per-losing-site shadow records for the status report, de-duplicated on
/// `(site, winner)` so a site that loses several domains to one winner appears
/// once. The common entry is a shadowed apex; a hand-edited config can also
/// collide two sites on an explicit domain.
fn domain_shadows(
    cfg: &orcker_config::Config,
    sites: Vec<orcker_core::Site>,
) -> Vec<orcker_ipc::DomainShadow> {
    let mut out: Vec<orcker_ipc::DomainShadow> = Vec::new();
    for collision in crate::site_domains::collisions(cfg, sites) {
        for loser in collision.losers {
            let entry = orcker_ipc::DomainShadow {
                site: loser,
                shadowed_by: collision.winner.clone(),
            };
            if !out.contains(&entry) {
                out.push(entry);
            }
        }
    }
    out
}

/// Builds the full [`StatusReport`]. The config lock is held across the router
/// snapshot (config-then-router, the same order `handle_mutation` takes) so
/// `domain_shadows` sees a consistent (config, router) pair rather than one from
/// either side of a concurrent mutation.
#[allow(clippy::too_many_lines)]
async fn build_status_report(state: &DaemonState) -> orcker_ipc::StatusReport {
    use orcker_platform::SystemMetrics;

    let (
        sites,
        tld,
        mail_enabled,
        mail_port,
        symlink_protection,
        mcp_enabled,
        lan_enabled,
        shadows,
    ) = {
        let cfg = state.config.lock().await;
        let router = state.router.read().await;
        let mut counts = orcker_ipc::SiteCounts::default();
        for s in router.iter() {
            match s.kind() {
                orcker_core::SiteKind::Parked => counts.parked += 1,
                orcker_core::SiteKind::Linked => counts.linked += 1,
            }
            if s.secure() {
                counts.secured += 1;
            }
        }
        let site_snapshot: Vec<orcker_core::Site> = router.iter().cloned().collect();
        let shadows = domain_shadows(&cfg, site_snapshot);
        (
            counts,
            cfg.tld.as_str().to_owned(),
            cfg.mail.enabled,
            cfg.mail.port,
            cfg.symlink_protection,
            cfg.mcp_enabled,
            cfg.lan_enabled,
            shadows,
        )
    };

    // LAN effective signals: discover the IP (gated on LAN being on) and read
    // whether the bootstrap listener actually bound. `None` when LAN is off.
    let (lan_ip, lan_setup_bound) = if lan_enabled {
        let bound = state
            .lan_setup_bound
            .load(std::sync::atomic::Ordering::Relaxed);
        (state.lan_ip, Some(bound))
    } else {
        (None, None)
    };

    let metrics = orcker_platform::ActiveSystemMetrics::new();

    let daemon_pid = std::process::id();
    let rss_by_pid = collect_rss_by_pid(metrics, vec![daemon_pid]).await;

    let fp = state.ca_fingerprint;
    let ca_path = state.ca_path.clone();
    let trusted_system = tokio::task::spawn_blocking(move || {
        use orcker_platform::TrustStore;
        orcker_platform::ActiveTrustStore::new()
            .is_trusted(&ca_path, &fp)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let browser_fp = state.ca_fingerprint;
    let browser_trust = tokio::task::spawn_blocking(move || {
        use orcker_platform::TrustStore;
        orcker_platform::ActiveTrustStore::new()
            .browser_ca_trust(&browser_fp)
            .ok()
            .map(map_browser_trust)
    })
    .await
    .ok()
    .flatten();

    let tld_probe = tld.clone();
    let dns_addr = state.dns_addr;
    let resolver_installed = tokio::task::spawn_blocking(move || {
        use orcker_platform::ResolverInstaller;
        orcker_platform::ActiveResolverInstaller::new()
            .is_installed(&tld_probe, dns_addr)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let (port_redirect, foreign_web_listener, port_redirect_targets, lan_redirect_targets) =
        tokio::task::spawn_blocking(|| {
            use orcker_platform::PortRedirector;
            let r = orcker_platform::ActivePortRedirector::new();
            (
                r.is_active(),
                r.foreign_web_listener(),
                r.redirect_targets().map(redirect_targets_to_wire),
                r.lan_redirect_targets().map(redirect_targets_to_wire),
            )
        })
        .await
        .unwrap_or((None, None, None, None));

    let backup_tld = tld.clone();
    let resolver_backup = tokio::task::spawn_blocking(move || latest_resolver_backup(&backup_tld))
        .await
        .ok()
        .flatten();

    let load_avg = metrics
        .load_average()
        .map(|[a, b, c]| [load_to_centi(a), load_to_centi(b), load_to_centi(c)]);

    let (mail_count, mail_unread) = state.mail_store.counts().await;
    let shared_sites = crate::tunnel::shared_site_count(state).await;

    orcker_ipc::StatusReport {
        daemon_pid: std::process::id(),
        uptime_secs: state.started_at.elapsed().as_secs(),
        daemon_rss_bytes: rss_by_pid.get(&daemon_pid).copied(),
        tld,
        http: state.http,
        https: state.https,
        dns_addr: state.dns_addr,
        ca: orcker_ipc::CaStatus {
            path: state.ca_path.clone(),
            fingerprint: state.ca_fingerprint.to_hex(),
            trusted_system,
            browser_trust,
        },
        resolver_installed,
        port_redirect,
        foreign_web_listener,
        port_redirect_targets,
        lan_redirect_targets,
        resolver_backup,
        sites,
        load_avg,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        mail: Some(orcker_ipc::MailStatus {
            enabled: mail_enabled,
            port: mail_port,
            listening: state.mail.listening,
            count: mail_count,
            unread: mail_unread,
        }),
        web_unbound: state.web_unbound,
        dns_unbound: state.dns_unbound,
        boot_id: Some(state.boot_id),
        shared_sites,
        symlink_protection,
        shadows,
        mcp_enabled,
        lan_enabled,
        lan_ip,
        lan_setup_bound,
    }
}

/// Map the platform layer's parsed anchor targets `(http, https)` to the wire
/// type.
fn redirect_targets_to_wire((http, https): (u16, u16)) -> orcker_ipc::PortRedirectTargets {
    orcker_ipc::PortRedirectTargets { http, https }
}

/// The path of the most recent replaced-resolver backup for `tld`, if one was
/// saved within the last 7 days. macOS-only - the helper writes these when it
/// overwrites a pre-existing `/etc/resolver/<tld>`. The age bound keeps the
/// `doctor` finding a transient migration notice rather than permanent noise.
#[cfg(target_os = "macos")]
fn latest_resolver_backup(tld: &str) -> Option<String> {
    use orcker_platform::pure::resolver_file;
    const MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

    let dir = resolver_file::macos_backup_dir();
    let names: Vec<String> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    let latest = resolver_file::latest_backup(&names, tld)?;
    let secs = resolver_file::parse_backup_secs(latest, tld)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    (now.saturating_sub(secs) <= MAX_AGE_SECS)
        .then(|| dir.join(latest).to_string_lossy().into_owned())
}

#[cfg(not(target_os = "macos"))]
#[allow(clippy::missing_const_for_fn)]
fn latest_resolver_backup(_tld: &str) -> Option<String> {
    None
}

/// Convert a (non-negative) load-average figure to integer hundredths, clamped
/// into `u32`. The `as` cast is sign- and range-safe given the explicit clamp.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn load_to_centi(x: f64) -> u32 {
    let v = (x * 100.0).round();
    if v <= 0.0 {
        0
    } else if v >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        v as u32
    }
}

/// `doctor fix` - run the safe auto-fixes, then re-diagnose for the remainder.
async fn run_doctor_fix(state: &DaemonState) -> Response {
    let report = build_status_report(state).await;
    let performed: Vec<orcker_ipc::FixResult> = Vec::new();

    for action in orcker_doctor::plan_auto_fixes(&report) {
        tracing::warn!(?action, "unhandled doctor auto-fix action");
    }

    let after = build_status_report(state).await;
    let manual = orcker_doctor::diagnose(&after, path_needs_setup(state))
        .into_iter()
        .filter(|d| {
            matches!(
                d.severity,
                orcker_ipc::Severity::Warn | orcker_ipc::Severity::Fail
            )
        })
        .collect();

    Response::DoctorFix {
        report: orcker_ipc::FixReport { performed, manual },
    }
}

/// Absolute path to the `orcker` CLI binary, assumed a sibling of the running
/// `orckerd` (mirrors `orcker`'s own `elevate::sibling_binaries`). This is the target
/// the cover shims (`phpcover`/`php<ver>cover`) symlink to.
fn orcker_sibling() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("orcker"))
}

/// Map the platform browser-trust probe to the wire enum.
fn map_browser_trust(t: orcker_platform::BrowserCaTrust) -> orcker_ipc::BrowserTrust {
    match t {
        orcker_platform::BrowserCaTrust::Trusted => orcker_ipc::BrowserTrust::Trusted,
        orcker_platform::BrowserCaTrust::Untrusted => orcker_ipc::BrowserTrust::Untrusted,
        orcker_platform::BrowserCaTrust::ToolMissing => orcker_ipc::BrowserTrust::ToolMissing,
    }
}

/// Install (or remove) the Orcker CA into the per-user **browser** NSS stores.
/// Runs unprivileged as the daemon's user; the CA PEM is read from the daemon's
/// on-disk `ca_path`, so no cert material crosses the IPC boundary.
async fn trust_browsers(uninstall: bool, state: &DaemonState) -> Response {
    let ca_path = state.ca_path.clone();
    let joined = tokio::task::spawn_blocking(move || {
        use orcker_platform::TrustStore;
        let ts = orcker_platform::ActiveTrustStore::new();
        if uninstall {
            ts.uninstall_firefox_nss()
        } else {
            ts.install_firefox_nss(&ca_path)
        }
    })
    .await;

    match joined {
        Ok(Ok(o)) => Response::BrowserTrust {
            attempted: o.profiles_attempted,
            succeeded: o.profiles_succeeded,
            certutil_missing: o.certutil_missing,
        },
        Ok(Err(e)) => Response::Error {
            code: orcker_ipc::ErrorCode::Internal,
            message: format!("browser NSS trust failed: {e}"),
        },
        Err(e) => Response::Error {
            code: orcker_ipc::ErrorCode::Internal,
            message: format!("browser NSS trust task panicked: {e}"),
        },
    }
}

/// Reconcile the dev-tool shims (`node`/`npm`/`npx`/`bun`/`bunx`) under the
/// shared `shim_reconcile` mutex.
/// Best-effort: failures are logged. Used at startup and after install/uninstall.
pub(crate) async fn reconcile_tool_shims_now(state: &DaemonState) {
    let Some(_orcker_bin) = orcker_sibling() else {
        tracing::warn!("cannot locate the `orcker` binary; skipping tool-shim reconcile");
        return;
    };
    let _guard = state.shim_reconcile.lock().await;
    if let Err(e) = crate::tools::reconcile_tool_shims(&state.dirs) {
        tracing::warn!(error = %e, "tool-shim reconcile failed");
    }
}

/// Build the tool list and tag any *not* Orcker-managed tool that's available on
/// the user's PATH as `external` (Tooling shows an "External" badge but still
/// offers **Install**, to add orcker's own copy alongside it). Tools that
/// [don't accept an external copy](crate::tools::Tool::accepts_external) are
/// never tagged - orcker needs its own build, so Tooling shows them as simply not
/// installed. Skips the (login-shell) PATH resolution entirely when no remaining
/// tool could be tagged.
async fn list_tools_with_external(state: &DaemonState) -> Vec<orcker_ipc::ToolStatus> {
    let mut tools = crate::tools::list_status(&state.dirs);
    let taggable = |t: &orcker_ipc::ToolStatus| {
        !t.installed
            && crate::tools::Tool::parse(&t.id).is_some_and(crate::tools::Tool::accepts_external)
    };
    if !tools.iter().any(taggable) {
        return tools;
    }
    let Some(dirs) = crate::tools::external::resolve_user_path().await else {
        return tools;
    };
    let data_bin = crate::tools::bin_dir(&state.dirs);
    let data_root = &state.dirs.data;
    for t in &mut tools {
        if !taggable(t) {
            continue;
        }
        if let Some(tool) = crate::tools::Tool::parse(&t.id) {
            let found = crate::tools::external::external_tool(&dirs, tool, &data_bin, data_root);
            t.external = found.is_some();
            t.external_path = found.map(|p| p.display().to_string());
        }
    }
    tools
}

/// `install tool <id>` - download + verify the latest release, then (re)build its
/// `{data}/bin` shims. Runs the slow download with no lock held.
async fn install_tool(tool: &str, state: &DaemonState) -> Response {
    let Some(t) = crate::tools::Tool::parse(tool) else {
        return Response::Error {
            code: ErrorCode::NotFound,
            message: format!("unknown tool {tool:?}"),
        };
    };
    let dl = crate::download::ReqwestDownloader::new();
    let _mutate = state.tool_mutate.lock().await;
    match crate::tools::install(t, &state.dirs, &dl, None).await {
        Ok(()) => {
            reconcile_tool_shims_now(state).await;
            Response::Ok
        }
        Err(e) => Response::Error {
            code: tool_error_code(&e),
            message: e.to_string(),
        },
    }
}

/// `InstallToolStreamed` - install a tool as a background job, streaming its
/// output (Composer's, for the Laravel installer) into the job log. Returns
/// `JobStarted` immediately; the client polls `JobStatus`.
pub(crate) async fn install_tool_streamed(tool: String, state: Arc<DaemonState>) -> Response {
    let Some(t) = crate::tools::Tool::parse(&tool) else {
        return Response::Error {
            code: ErrorCode::NotFound,
            message: format!("unknown tool {tool:?}"),
        };
    };
    let (job_id, mut cancel) = state.jobs.create().await;
    let id = job_id.clone();
    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let drain = {
            let state = state.clone();
            let id = id.clone();
            tokio::spawn(async move {
                while let Some(line) = rx.recv().await {
                    state.jobs.push_log(&id, line).await;
                }
            })
        };

        state
            .jobs
            .set_phase(&id, format!("Installing {}", t.display_name()))
            .await;
        let dl = crate::download::ReqwestDownloader::new();
        let guard = state.tool_mutate.lock().await;
        let result = tokio::select! {
            r = crate::tools::install(t, &state.dirs, &dl, Some(&tx)) => Some(r),
            _ = cancel.changed() => None,
        };
        drop(guard);
        drop(tx);
        let _ = drain.await;

        match result {
            Some(Ok(())) => {
                reconcile_tool_shims_now(&state).await;
                state
                    .jobs
                    .finish(&id, orcker_ipc::JobState::Succeeded, None)
                    .await;
            }
            Some(Err(e)) => {
                state
                    .jobs
                    .finish(&id, orcker_ipc::JobState::Failed, Some(e.to_string()))
                    .await;
            }
            None => {
                state
                    .jobs
                    .finish(&id, orcker_ipc::JobState::Cancelled, None)
                    .await;
            }
        }
    });
    Response::JobStarted { job_id }
}

/// `uninstall tool <id>` - remove the tool's files, then prune its shims.
async fn uninstall_tool(tool: &str, state: &DaemonState) -> Response {
    let Some(t) = crate::tools::Tool::parse(tool) else {
        return Response::Error {
            code: ErrorCode::NotFound,
            message: format!("unknown tool {tool:?}"),
        };
    };
    let _mutate = state.tool_mutate.lock().await;
    match crate::tools::uninstall(&state.dirs, t) {
        Ok(()) => {
            reconcile_tool_shims_now(state).await;
            Response::Ok
        }
        Err(e) => Response::Error {
            code: tool_error_code(&e),
            message: e.to_string(),
        },
    }
}

/// Map a [`crate::tools::ToolError`] to an IPC error code (mirrors `php_error_code`).
fn tool_error_code(e: &crate::tools::ToolError) -> ErrorCode {
    use crate::tools::ToolError;
    match e {
        ToolError::Unknown(_) => ErrorCode::NotFound,
        ToolError::UnsupportedHost(_) => ErrorCode::InvalidPath,
        _ => ErrorCode::Internal,
    }
}

/// Set the mail-capture SMTP port. Persisted to config; takes effect on the next
/// daemon start/restart (no hot rebind), matching `SetServicePort`. Modelled on
/// `set_php_settings` (clone → set → validate → save → commit under the config
/// mutex) so an invalid value (e.g. a zero port) is rejected by the config
/// validator rather than overloading an unrelated `ErrorCode`.
async fn set_mail_port(port: u16, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.mail.port = port;
    if let Err(e) = new.validate() {
        return internal(format!("config validation failed: {e}"));
    }
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    tracing::info!(port, "set mail port (effective on next restart)");
    Response::Ok
}

/// Set the rootless HTTP/HTTPS fallback ports. Persisted to config; takes effect
/// on the next daemon restart (the client triggers it). Refused while a
/// privileged-port redirect is active - it is pinned to the current ports, so
/// changing them would break elevation until the user re-elevates.
async fn set_fallback_ports(http: u16, https: u16, state: &DaemonState) -> Response {
    let redirect_active = tokio::task::spawn_blocking(|| {
        use orcker_platform::PortRedirector;
        orcker_platform::ActivePortRedirector::new().is_active()
    })
    .await
    .unwrap_or(None);
    if redirect_active == Some(true) {
        return internal(
            "ports are elevated - remove the privileged-port redirect first (un-elevate ports), \
             change the ports, then re-elevate"
                .to_owned(),
        );
    }

    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.ports.fallback_http = http;
    new.ports.fallback_https = https;
    if let Err(e) = new.validate() {
        return internal(format!("config validation failed: {e}"));
    }
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    tracing::info!(
        http,
        https,
        "set fallback ports (effective on next restart)"
    );
    Response::Ok
}

/// Set the embedded DNS responder port (`dns_port`). Persisted to config; takes
/// effect on the next daemon restart (the client triggers it). A zero port is
/// rejected explicitly here - unlike the web/mail/dumps ports, `dns_port == 0` is
/// a *valid* "ephemeral" value for in-process tests, so `validate()` permits it;
/// for a user-facing change a zero port (which the OS resolver could never target)
/// is meaningless.
async fn set_dns_port(port: u16, state: &DaemonState) -> Response {
    if port == 0 {
        return Response::Error {
            code: orcker_ipc::ErrorCode::Internal,
            message: "DNS port must be non-zero".to_owned(),
        };
    }
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.dns_port = port;
    if let Err(e) = new.validate() {
        return internal(format!("config validation failed: {e}"));
    }
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    tracing::info!(port, "set DNS port (effective on next restart)");
    Response::Ok
}

/// Enable or disable mail capture. Persisted to config; takes effect on the next
/// daemon start/restart.
async fn set_mail_enabled(enabled: bool, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.mail.enabled = enabled;
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    tracing::info!(enabled, "set mail enabled (effective on next restart)");
    Response::Ok
}

/// Enable or disable the proxy's symlink-escape protection. Persisted to config
/// and mirrored into the shared `symlink_protection` atomic, so the proxy picks
/// it up on the next request without a daemon restart.
async fn set_symlink_protection(enabled: bool, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.symlink_protection = enabled;
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    state
        .symlink_protection
        .store(enabled, std::sync::atomic::Ordering::Relaxed);
    tracing::info!(enabled, "set symlink protection");
    Response::Ok
}

/// Enable or disable the MCP server gate. Persist-only: the daemon runs no MCP
/// server, so there is nothing live to update. Each `orcker mcp` session reads the
/// flag back from [`Request::Status`], so enabling reaches running agent
/// sessions on their next tool call.
async fn set_mcp_enabled(enabled: bool, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.mcp_enabled = enabled;
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    tracing::info!(enabled, "set mcp enabled");
    Response::Ok
}

/// Persist the `lan_enabled` flag. Persist-only: the bootstrap listener only
/// re-binds on the daemon restart the CLI triggers next (a listen socket's bind
/// address is fixed at bind time), so there is no live atomic to flip here.
///
/// Disabling also revokes any pending one-time remote-setup code, since the
/// listener lingers until that restart and an already-minted code would
/// otherwise stay redeemable. The revocation happens while the `config` lock is
/// held so it and [`mint_remote_setup_code`]'s check-and-publish cannot
/// interleave - otherwise a mint that had already passed its `lan_enabled` check
/// could store a live code *after* disablement completed.
async fn set_lan_enabled(enabled: bool, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();
    new.lan_enabled = enabled;
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }
    *cfg_guard = new;
    if !enabled {
        *state.remote_setup_code.lock().await = None;
    }
    drop(cfg_guard);
    tracing::info!(enabled, "set lan enabled");
    Response::Ok
}

/// TTL for a minted remote-setup code.
const REMOTE_SETUP_CODE_TTL: std::time::Duration = std::time::Duration::from_secs(900);

/// Mint a one-time bootstrap code. Guarded on LAN actually being up (config on
/// *and* the bootstrap listener bound), so a code is never handed out when
/// nothing would serve it.
///
/// The `config` lock is held across the whole check-and-publish so a concurrent
/// [`set_lan_enabled`]`(false)` cannot slip between the `lan_enabled` check and
/// storing the code (which would leave a live code after disablement). The URL
/// reuses the startup-discovered `lan_ip` so the printed host always matches what
/// is actually served, and the returned `script_sha256` is the hash recorded when
/// the endpoint bound - its absence means the endpoint isn't serving, so minting
/// fails closed rather than hand out a useless code.
async fn mint_remote_setup_code(state: &DaemonState) -> Response {
    let cfg_guard = state.config.lock().await;
    if !cfg_guard.lan_enabled {
        return lan_not_ready("LAN mode is off - run `orcker lan enable` first".to_owned());
    }
    let lan_setup_port = cfg_guard.lan_setup_port;
    if !state
        .lan_setup_bound
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return lan_not_ready(
            "the remote-setup listener isn't bound (restart the daemon after enabling LAN)"
                .to_owned(),
        );
    }
    let Some(lan_ip) = state.lan_ip else {
        return lan_not_ready("the LAN IP isn't known (restart the daemon)".to_owned());
    };

    let Some(script_sha256) = state.lan_setup_script_sha256.lock().await.clone() else {
        return lan_not_ready(
            "the installer hash isn't ready (restart the daemon after enabling LAN)".to_owned(),
        );
    };

    let mut bytes = [0u8; 16];
    {
        use rand::RngCore as _;
        rand::thread_rng().fill_bytes(&mut bytes);
    }
    let code = hex::encode(bytes);

    *state.remote_setup_code.lock().await = Some(crate::state::RemoteSetupCode {
        value: code.clone(),
        expires_at: std::time::Instant::now() + REMOTE_SETUP_CODE_TTL,
        used: false,
    });
    drop(cfg_guard);

    let url = format!("http://{lan_ip}:{lan_setup_port}/remote-setup?code={code}");
    Response::RemoteSetup {
        code,
        url,
        script_sha256,
        expires_in_secs: REMOTE_SETUP_CODE_TTL.as_secs(),
    }
}

/// Apply a mutation: canonicalise paths, run the pure delta, validate, persist,
/// and swap the live router - **build-then-validate-then-commit** so a failed
/// mutation leaves disk and the live router untouched. A `Link`'s web-root
/// detection scan runs here, before `state.config` is locked, so a slow or
/// network-mounted project directory can't stall other mutating requests
/// that share the lock.
pub(crate) async fn handle_mutation(req: Request, state: &DaemonState) -> Response {
    let canonical = match &req {
        Request::Park { path } | Request::Link { path, .. } => match canonicalize_dir(path) {
            Ok(p) => Some(p),
            Err(resp) => return resp,
        },
        _ => None,
    };
    let link_web_subpath = match &req {
        Request::Link { .. } => canonical.as_deref().map(detect_web_subpath),
        _ => None,
    };

    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();

    let live_default = new.php.default;
    let applied = match &req {
        Request::SetWebRoot { name, path } => {
            match resolve_web_root_mutation(
                &mut new,
                &*state.router.read().await,
                name,
                path.as_deref(),
            ) {
                Ok(a) => a,
                Err(resp) => return resp,
            }
        }
        _ => match mutate::apply(
            &mut new,
            &*state.router.read().await,
            &req,
            canonical,
            live_default,
        ) {
            Ok(a) => a,
            Err(e) => {
                return Response::Error {
                    code: mutate::error_code(&e),
                    message: e.to_string(),
                }
            }
        },
    };

    if let (Request::Link { name, .. }, Some(subpath)) = (&req, &link_web_subpath) {
        let name_lc = name.to_ascii_lowercase();
        if let Some(site) = new.linked.iter_mut().find(|s| s.name() == name_lc) {
            site.set_web_subpath(subpath);
        }
    }

    if let Some(failure) = commit_config(new, &mut cfg_guard, state, &applied.summary).await {
        return failure;
    }
    drop(cfg_guard);
    Response::Ok
}

/// Handles [`Request::LinkProject`]: register a directory as a container
/// project on a persistent loopback port (FR-021, FR-013).
///
/// The I/O lives here (canonicalising the path, reading or creating
/// `orcker.yml`, probing ports); the decision is [`link::plan_link`], which is
/// pure and takes the probe as a trait. A relink of an already-registered
/// directory touches neither the config nor the descriptor.
async fn handle_link_project(req: Request, state: &DaemonState) -> Response {
    let Request::LinkProject { path, name, port } = req else {
        return internal("handle_link_project called with the wrong request".to_owned());
    };

    let root = match canonicalize_dir(&path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let descriptor_path = root.join(orcker_config::orcker_yml::FILE_NAME);
    let existing = match std::fs::read_to_string(&descriptor_path) {
        Ok(raw) => match orcker_config::OrckerYml::parse(&raw) {
            Ok(yml) => Some(yml),
            Err(e) => {
                return Response::Error {
                    code: ErrorCode::InvalidPath,
                    message: format!("{}: {e}", descriptor_path.display()),
                }
            }
        },
        Err(_) => None,
    };

    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();

    let plan = match link::plan_link(
        &new,
        &root,
        name.as_deref(),
        port,
        existing.as_ref(),
        &link::TcpPortProbe,
    ) {
        Ok(plan) => plan,
        Err(e) => {
            return Response::Error {
                code: link_error_code(&e),
                message: e.to_string(),
            }
        }
    };

    let (project, descriptor, wrote_descriptor) = match plan {
        link::LinkPlan::AlreadyLinked { project } => {
            let domain = project_domain(
                &*state.router.read().await,
                project.name(),
                new.tld.as_str(),
            );
            return Response::Project {
                project: Box::new(project_entry(&project, existing.as_ref(), domain)),
                created: false,
                wrote_descriptor: false,
            };
        }
        link::LinkPlan::Link { project, write_yml } => {
            if let Some(refusal) =
                project_loop_refusal(&project, &[state.http.bound, state.https.bound])
            {
                return refusal;
            }
            match write_yml {
                Some(yml) => {
                    if let Err(e) = std::fs::write(&descriptor_path, yml.render()) {
                        return internal(format!(
                            "could not write {}: {e}",
                            descriptor_path.display()
                        ));
                    }
                    (project, Some(yml), true)
                }
                None => (project, existing, false),
            }
        }
    };

    let tld = new.tld.as_str().to_owned();
    let summary = format!(
        "linked project {} on port {}",
        project.name(),
        project.port()
    );
    new.projects.push(project.clone());
    if let Some(failure) = commit_config(new, &mut cfg_guard, state, &summary).await {
        return failure;
    }
    drop(cfg_guard);

    let domain = project_domain(&*state.router.read().await, project.name(), &tld);
    Response::Project {
        project: Box::new(project_entry(&project, descriptor.as_ref(), domain)),
        created: true,
        wrote_descriptor,
    }
}

/// Maps a [`link::LinkError`] onto the wire error code clients match on.
fn link_error_code(e: &link::LinkError) -> ErrorCode {
    match e {
        link::LinkError::NameTaken { .. } => ErrorCode::AlreadyExists,
        link::LinkError::PortTaken { .. } => ErrorCode::PortReserved,
        link::LinkError::Core(orcker_core::CoreError::PortRangeExhausted { .. }) => {
            ErrorCode::PortRangeExhausted
        }
        link::LinkError::PortOutOfRange { .. }
        | link::LinkError::NoName { .. }
        | link::LinkError::Config(_)
        | link::LinkError::Core(_) => ErrorCode::InvalidPath,
    }
}

/// One project's wire entry, with the descriptor fields filled in from `yml`.
fn project_entry(
    project: &orcker_core::ContainerProject,
    yml: Option<&orcker_config::OrckerYml>,
    domain: String,
) -> orcker_ipc::ProjectEntry {
    orcker_ipc::ProjectEntry {
        name: project.name().to_owned(),
        root: project.root().to_path_buf(),
        port: project.port(),
        secure: project.secure(),
        primary_domain: Some(domain),
        schema_version: yml.map(|y| y.schema_version),
        php: yml.map(|y| y.php),
        db: yml.map(|y| y.db.clone()),
        preset: yml.map(|y| y.preset.clone()),
    }
}

/// Validates `new`, rebuilds the router from it, saves it, and swaps both into
/// the live state. Returns the error [`Response`] on failure and `None` on
/// success, leaving the caller to send its own success reply.
///
/// Shared by [`handle_mutation`] and [`handle_link_project`] so the commit
/// sequence (validate, rebuild, save, swap, notify the watcher) exists once.
async fn commit_config(
    new: orcker_config::Config,
    cfg_guard: &mut tokio::sync::MutexGuard<'_, orcker_config::Config>,
    state: &DaemonState,
    summary: &str,
) -> Option<Response> {
    if let Err(e) = new.validate() {
        return Some(internal(format!("config validation failed: {e}")));
    }

    let (candidate, candidate_wordpress, candidate_laravel) =
        match startup::build_router(&new, &state.dirs, &state.detect_cache) {
            Ok(r) => r,
            Err(DaemonError::Core(orcker_core::CoreError::DuplicateSite { name })) => {
                return Some(Response::Error {
                    code: ErrorCode::AlreadyExists,
                    message: format!("duplicate site: {name}"),
                })
            }
            Err(e) => return Some(internal(format!("router rebuild failed: {e}"))),
        };

    if let Err(e) = new.save(&state.config_path) {
        return Some(internal(format!("config save failed: {e}")));
    }

    **cfg_guard = new;
    *state.router.write().await = candidate;
    *state.wordpress_sites.write().await = candidate_wordpress;
    *state.laravel_sites.write().await = candidate_laravel;

    state.watch_dirty.notify_one();

    tracing::info!(summary, "applied mutation");
    None
}

/// Whether `url` is a loopback target on one of Orcker's **actively bound** proxy
/// ports - a request to such a proxy would forward straight back into Orcker,
/// re-resolve, and loop. Checked here rather than in the pure config layer
/// because the bound port is a runtime fact. A malformed URL returns `false` so
/// the mutation handler surfaces the precise parse error instead.
fn is_self_forward(url: &str, bound_ports: &[u16]) -> bool {
    let Ok(target) = orcker_core::UpstreamTarget::from_url_str(url) else {
        return false;
    };
    targets_bound_listener(&target, bound_ports)
}

/// The loop test itself, over an already-typed target. A linked project is
/// routed as a proxy onto its own loopback port, so it has to answer this
/// question too, and it holds a [`orcker_core::ContainerProject`] rather than a
/// URL string - formatting one just to re-parse it would be a second
/// implementation of the same rule.
fn targets_bound_listener(target: &orcker_core::UpstreamTarget, bound_ports: &[u16]) -> bool {
    let loopback = target.host() == "localhost"
        || target
            .host()
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
    loopback && bound_ports.contains(&target.port())
}

/// The refusal a project about to be linked earns, or `None` when it routes
/// somewhere other than Orcker itself. A project that cannot even be projected
/// into a proxy would be dropped by the router later, so it is refused here
/// instead of being registered into a config that silently does nothing.
fn project_loop_refusal(
    project: &orcker_core::ContainerProject,
    bound_ports: &[u16],
) -> Option<Response> {
    match project.proxy_site() {
        Ok(proxy) => targets_bound_listener(proxy.target(), bound_ports).then(self_forward_refusal),
        Err(e) => Some(Response::Error {
            code: ErrorCode::InvalidPath,
            message: e.to_string(),
        }),
    }
}

/// The refusal both `AddProxy` and `LinkProject` return for a target that points
/// back at Orcker's own listener.
fn self_forward_refusal() -> Response {
    Response::Error {
        code: ErrorCode::InvalidPath,
        message: "proxy target points at orcker's own listening port (routing loop)".to_owned(),
    }
}

/// Reply to [`Request::ListProxies`]: whole-host proxies plus every per-site
/// path-prefix rule. Linked rules key by site name already; parked rules key by
/// document-root, which is resolved through the live router to the current site
/// name (mirroring `ListSites`) so the output round-trips through
/// `orcker proxy remove <site> <prefix>`. A parked docroot with no current site
/// falls back to the raw key.
///
/// Domain fields are reported only for a proxy the router actually holds. A
/// name-shadowed proxy stays in the config but is never inserted, and the shared
/// domain maps are keyed by claimant name, so enriching it would report the
/// shadowing site's domains as the proxy's own.
async fn list_proxies(state: &DaemonState) -> Response {
    let cfg = state.config.lock().await;
    let router = state.router.read().await;
    let proxies = cfg
        .proxies
        .iter()
        .map(|p| {
            let (primary_domain, domains) = if router.proxy(p.name()).is_some() {
                site_entry_domains(&router, p.name(), cfg.tld.as_str())
            } else {
                (None, Vec::new())
            };
            orcker_ipc::ProxyEntry {
                name: p.name().to_owned(),
                target: p.target().to_string(),
                secure: p.secure(),
                primary_domain,
                domains,
            }
        })
        .collect();
    let mut rules = Vec::new();
    for (site, site_rules) in &cfg.proxy_rules.linked {
        for r in site_rules {
            rules.push(orcker_ipc::ProxyRuleEntry {
                site: site.clone(),
                prefix: r.prefix().to_owned(),
                target: r.target().to_string(),
            });
        }
    }
    for (docroot, site_rules) in &cfg.proxy_rules.parked {
        let site_name = router
            .iter()
            .find(|s| s.document_root().to_string_lossy().as_ref() == docroot.as_str())
            .map_or_else(|| docroot.clone(), |s| s.name().to_owned());
        for r in site_rules {
            rules.push(orcker_ipc::ProxyRuleEntry {
                site: site_name.clone(),
                prefix: r.prefix().to_owned(),
                target: r.target().to_string(),
            });
        }
    }
    Response::Proxies { proxies, rules }
}

/// Reply to [`Request::ListRoutes`]: every per-site path-prefix routing rule.
/// Parked rules key by document-root, resolved through the live router to the
/// current site name exactly as [`list_proxies`] does, so the output round-trips
/// through `orcker route remove <site> <prefix>`. A parked docroot with no current
/// site falls back to the raw key.
async fn list_routes(state: &DaemonState) -> Response {
    let cfg = state.config.lock().await;
    let router = state.router.read().await;
    let mut rules = Vec::new();
    for (site, site_rules) in &cfg.route_rules.linked {
        for r in site_rules {
            rules.push(orcker_ipc::RouteRuleEntry {
                site: site.clone(),
                prefix: r.prefix().to_owned(),
                target: r.target().to_owned(),
            });
        }
    }
    for (docroot, site_rules) in &cfg.route_rules.parked {
        let site_name = router
            .iter()
            .find(|s| s.document_root().to_string_lossy().as_ref() == docroot.as_str())
            .map_or_else(|| docroot.clone(), |s| s.name().to_owned());
        for r in site_rules {
            rules.push(orcker_ipc::RouteRuleEntry {
                site: site_name.clone(),
                prefix: r.prefix().to_owned(),
                target: r.target().to_owned(),
            });
        }
    }
    Response::Routes { rules }
}

/// Auto-detect the web subpath to serve for a project at `doc_root` (e.g.
/// `public` for Laravel). Shared by `SetWebRoot`'s auto-detect branch and
/// `Link`'s creation-time auto-detect.
fn detect_web_subpath(doc_root: &Path) -> String {
    orcker_core::detect(&orcker_platform::gather_project_signals(doc_root))
        .subpath
        .to_string_lossy()
        .into_owned()
}

/// Resolve a `SetWebRoot` request against `new`, doing the filesystem I/O
/// (containment check, or re-detection) the pure `mutate::apply` can't. A
/// **linked** site stores the chosen subpath on its `Site`; a **parked** site
/// stores it in `overrides[doc_root].web_root`. `path = None` resets to
/// auto-detect: re-detect now for linked, clear the override for parked.
fn resolve_web_root_mutation(
    new: &mut orcker_config::Config,
    router: &orcker_core::SiteRouter,
    name: &str,
    path: Option<&str>,
) -> Result<mutate::Applied, Response> {
    let name_lc = name.to_ascii_lowercase();

    if let Some(site) = new.linked.iter_mut().find(|s| s.name() == name_lc) {
        let doc_root = site.document_root().to_path_buf();
        let rel = if let Some(p) = path {
            resolve_web_root_within(&doc_root, p)?
        } else {
            detect_web_subpath(&doc_root)
        };
        site.set_web_subpath(&rel);
        return Ok(mutate::Applied {
            summary: web_root_summary(&name_lc, &rel),
        });
    }

    if let Some(parked) = router.get(&name_lc) {
        let key = parked.document_root().to_string_lossy().into_owned();
        if let Some(p) = path {
            let rel = resolve_web_root_within(parked.document_root(), p)?;
            new.overrides.entry(key).or_default().web_root = Some(rel.clone());
            return Ok(mutate::Applied {
                summary: web_root_summary(&name_lc, &rel),
            });
        }
        if let Some(ov) = new.overrides.get_mut(&key) {
            ov.web_root = None;
            if ov.php.is_none() && ov.secure.is_none() {
                new.overrides.remove(&key);
            }
        }
        return Ok(mutate::Applied {
            summary: format!("{name_lc} web root reset to auto-detect"),
        });
    }

    Err(Response::Error {
        code: ErrorCode::NotFound,
        message: mutate::not_found_site(new, &name_lc).to_string(),
    })
}

/// One-line summary for a web-root change.
fn web_root_summary(name: &str, rel: &str) -> String {
    if rel.is_empty() {
        format!("{name} now served from its project root")
    } else {
        format!("{name} now served from {rel}")
    }
}

/// Resolve a user-supplied served path against `doc_root` and return the
/// validated **relative** remainder (empty = serve the document root itself).
///
/// Rejects anything that escapes `doc_root`. Both sides are canonicalised
/// before comparison so a `\\?\` verbatim prefix from `fs::canonicalize` on
/// Windows doesn't spuriously fail the containment check against the
/// non-verbatim stored `document_root`.
fn resolve_web_root_within(doc_root: &Path, input: &str) -> Result<String, Response> {
    let candidate = {
        let p = Path::new(input);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            doc_root.join(p)
        }
    };
    let canon_candidate = std::fs::canonicalize(&candidate)
        .map_err(|e| invalid_path(format!("cannot resolve {}: {e}", candidate.display())))?;
    if !canon_candidate.is_dir() {
        return Err(invalid_path(format!(
            "served path is not a directory: {}",
            canon_candidate.display()
        )));
    }
    let canon_root = std::fs::canonicalize(doc_root)
        .map_err(|e| invalid_path(format!("cannot resolve {}: {e}", doc_root.display())))?;
    let rel = canon_candidate.strip_prefix(&canon_root).map_err(|_| {
        invalid_path(format!(
            "served path must be inside the site directory ({})",
            canon_root.display()
        ))
    })?;
    Ok(rel.to_string_lossy().into_owned())
}

/// Canonicalise `path` and require it to be an existing directory, or return a
/// ready-made `InvalidPath` error response.
fn canonicalize_dir(path: &Path) -> Result<PathBuf, Response> {
    match std::fs::canonicalize(path) {
        Ok(p) if p.is_dir() => Ok(p),
        Ok(p) => Err(invalid_path(format!("not a directory: {}", p.display()))),
        Err(e) => Err(invalid_path(format!(
            "cannot resolve {}: {e}",
            path.display()
        ))),
    }
}

fn invalid_path(message: String) -> Response {
    Response::Error {
        code: ErrorCode::InvalidPath,
        message,
    }
}

pub(crate) fn internal(message: String) -> Response {
    tracing::warn!(%message, "mutation failed");
    Response::Error {
        code: ErrorCode::Internal,
        message,
    }
}

fn lan_not_ready(message: String) -> Response {
    Response::Error {
        code: ErrorCode::LanNotReady,
        message,
    }
}

/// Apply a group mutation (create/delete/reorder/assign). Groups are a
/// config-only organisational overlay that never affects routing, so this uses
/// the lighter clone → apply → validate → save → commit path (like
/// [`set_dns_port`]) - **no** router rebuild and **no** `watch_dirty` notify,
/// which would only provoke a needless parked-dir rescan.
async fn handle_group_mutation(req: Request, state: &DaemonState) -> Response {
    let mut cfg_guard = state.config.lock().await;
    let mut new = cfg_guard.clone();

    // Groups ignore `default_php`, but `apply` still takes it; capture before the
    // mutable borrow of `new`.
    let live_default = new.php.default;
    let applied = match mutate::apply(
        &mut new,
        &*state.router.read().await,
        &req,
        None,
        live_default,
    ) {
        Ok(a) => a,
        Err(e) => {
            return Response::Error {
                code: mutate::error_code(&e),
                message: e.to_string(),
            }
        }
    };

    if let Err(e) = new.validate() {
        return internal(format!("config validation failed: {e}"));
    }
    if let Err(e) = new.save(&state.config_path) {
        return internal(format!("config save failed: {e}"));
    }

    *cfg_guard = new;
    tracing::info!(summary = %applied.summary, "applied group mutation");
    Response::Ok
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::case_sensitive_file_extension_comparisons
)]
mod tests {
    use super::*;

    use crate::test_support::state_in;

    #[test]
    fn self_forward_matches_only_loopback_on_bound_ports() {
        let bound = [8080, 8443];
        assert!(is_self_forward("http://127.0.0.1:8080", &bound));
        assert!(is_self_forward("https://localhost:8443", &bound));
        assert!(is_self_forward("http://[::1]:8080", &bound));
        assert!(!is_self_forward("http://127.0.0.1:3000", &bound));
        assert!(!is_self_forward("http://192.168.1.5:8080", &bound));
        assert!(!is_self_forward("http://example.com:8080", &bound));
        assert!(!is_self_forward("not-a-url", &bound));
    }

    #[tokio::test]
    async fn dispatch_ping_returns_pong() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(matches!(
            dispatch(Request::Ping, &state).await,
            Response::Pong
        ));
    }

    /// A name-shadowed proxy is never inserted, so its name in the shared domain
    /// maps belongs to the site that shadowed it. Reporting it must stay empty
    /// rather than advertising the site's domains as the proxy's.
    #[tokio::test]
    async fn list_proxies_reports_nothing_for_a_name_shadowed_proxy() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let target = orcker_core::UpstreamTarget::from_url_str("http://127.0.0.1:9011").unwrap();
        let shadowed = orcker_core::ProxySite::new("app", target).unwrap();
        {
            let mut cfg = state.config.lock().await;
            cfg.proxies.push(shadowed);
        }
        {
            let corp = orcker_core::Domain::parse_subpart("corp").unwrap();
            let site =
                orcker_core::Site::parked("app", "/srv/app", orcker_core::PhpVersion::new(8, 3))
                    .unwrap();
            let mut router = state.router.write().await;
            router
                .insert_with_domains(
                    site,
                    vec![orcker_core::Domain::apex("app"), corp.clone()],
                    corp,
                )
                .unwrap();
        }

        match dispatch(Request::ListProxies, &state).await {
            Response::Proxies { proxies, .. } => {
                let shadowed = proxies.iter().find(|p| p.name == "app").unwrap();
                assert_eq!(shadowed.primary_domain, None);
                assert!(shadowed.domains.is_empty());
            }
            other => panic!("expected Proxies, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_status_reports_runtime_facts() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(Request::Status, &state).await {
            Response::Status { report } => {
                assert_eq!(report.tld, "test");
                assert_eq!(report.daemon_pid, std::process::id());
                assert!(report.http.fell_back);
                assert_eq!(report.http.requested, 80);
                assert_eq!(report.http.bound, 8080);
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_proxies_reports_domains_only_for_a_customised_proxy() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let target = orcker_core::UpstreamTarget::from_url_str("http://127.0.0.1:9011").unwrap();
        let plain = orcker_core::ProxySite::new("reverb", target.clone()).unwrap();
        let custom = orcker_core::ProxySite::new("app", target).unwrap();
        {
            let mut cfg = state.config.lock().await;
            cfg.proxies.push(plain.clone());
            cfg.proxies.push(custom.clone());
        }
        {
            let corp = orcker_core::Domain::parse_subpart("corp").unwrap();
            let mut router = state.router.write().await;
            router.insert_proxy(plain).unwrap();
            router
                .insert_proxy_with_domains(
                    custom,
                    vec![orcker_core::Domain::apex("app"), corp.clone()],
                    corp,
                )
                .unwrap();
        }

        match dispatch(Request::ListProxies, &state).await {
            Response::Proxies { proxies, .. } => {
                let plain = proxies.iter().find(|p| p.name == "reverb").unwrap();
                assert_eq!(plain.primary_domain, None);
                assert!(plain.domains.is_empty());
                let custom = proxies.iter().find(|p| p.name == "app").unwrap();
                assert_eq!(custom.primary_domain.as_deref(), Some("corp.test"));
                assert_eq!(custom.domains, ["app.test", "corp.test"]);
            }
            other => panic!("expected Proxies, got {other:?}"),
        }
    }

    const SAMPLE_EML: &[u8] = b"From: Example <hello@example.com>\r\n\
To: test@test.com\r\n\
Subject: Captured\r\n\r\nhi\r\n";

    #[tokio::test]
    async fn dispatch_list_mails_empty_then_populated_then_cleared() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        match dispatch(Request::ListMails, &state).await {
            Response::Mails { mails } => assert!(mails.is_empty()),
            other => panic!("expected Mails, got {other:?}"),
        }

        state.mail_store.append(SAMPLE_EML).await.unwrap();
        let id = match dispatch(Request::ListMails, &state).await {
            Response::Mails { mails } => {
                assert_eq!(mails.len(), 1);
                assert_eq!(mails[0].subject, "Captured");
                mails[0].id.clone()
            }
            other => panic!("expected Mails, got {other:?}"),
        };

        match dispatch(Request::GetMail { id: id.clone() }, &state).await {
            Response::Mail { mail } => assert_eq!(mail.subject, "Captured"),
            other => panic!("expected Mail, got {other:?}"),
        }

        match dispatch(
            Request::GetMail {
                id: "999999".into(),
            },
            &state,
        )
        .await
        {
            Response::Error { code, .. } => assert!(matches!(code, ErrorCode::NotFound)),
            other => panic!("expected NotFound, got {other:?}"),
        }

        assert!(matches!(
            dispatch(Request::ClearMails, &state).await,
            Response::Ok
        ));
        match dispatch(Request::ListMails, &state).await {
            Response::Mails { mails } => assert!(mails.is_empty()),
            other => panic!("expected empty Mails, got {other:?}"),
        }
    }

    async fn status_mail(state: &DaemonState) -> orcker_ipc::MailStatus {
        match dispatch(Request::Status, state).await {
            Response::Status { report } => report.mail.expect("status should carry mail"),
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_status_includes_mail() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        let empty = status_mail(&state).await;
        assert!(empty.enabled);
        assert_eq!(empty.port, orcker_config::DEFAULT_MAIL_PORT);
        assert!(!empty.listening);
        assert_eq!(empty.count, 0);
        assert_eq!(empty.unread, 0);

        state
            .mail_store
            .append(b"From: a@b.c\r\nTo: d@e.f\r\nSubject: Hi\r\n\r\nbody\r\n")
            .await
            .unwrap();
        let seeded = status_mail(&state).await;
        assert_eq!(seeded.count, 1);
        assert_eq!(seeded.unread, 1);

        state
            .mail_store
            .mark_read(&["000000".to_string()])
            .await
            .unwrap();
        let read = status_mail(&state).await;
        assert_eq!(read.count, 1);
        assert_eq!(read.unread, 0);
    }

    #[tokio::test]
    async fn dispatch_set_mail_port_persists_and_rejects_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        match dispatch(Request::SetMailPort { port: 0 }, &state).await {
            Response::Error { code, .. } => assert!(matches!(code, ErrorCode::Internal)),
            other => panic!("expected Error, got {other:?}"),
        }

        assert!(matches!(
            dispatch(Request::SetMailPort { port: 3030 }, &state).await,
            Response::Ok
        ));
        assert_eq!(state.config.lock().await.mail.port, 3030);

        assert!(matches!(
            dispatch(Request::SetMailEnabled { enabled: true }, &state).await,
            Response::Ok
        ));
        assert!(state.config.lock().await.mail.enabled);

        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert_eq!(reloaded.mail.port, 3030);
        assert!(reloaded.mail.enabled);
    }

    #[tokio::test]
    async fn dispatch_list_sites_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => assert!(sites.is_empty()),
            other => panic!("expected Sites, got {other:?}"),
        }
    }

    /// The `Sites` listing is the only place the effective front-controller mode
    /// is derived now: the daemon reads the runtime `is_wordpress` fact out of
    /// `state.wordpress_sites` and resolves it against the site's stored
    /// override via [`orcker_core::Site::uses_front_controller`].
    ///
    /// Columns: `(name, web_subpath, stored_override, expected)`. Rows 1-3 are
    /// the derived default (a framework served from a subdir funnels; plain
    /// root-served PHP and `WordPress` in any layout execute directly), rows 4-5
    /// the override winning over it.
    #[tokio::test]
    async fn list_sites_derives_front_controller_from_the_wordpress_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let cases: &[(&str, &str, Option<bool>, bool)] = &[
            ("app", "public", None, true),
            ("plain", "", None, false),
            ("blog", "web", None, false),
            ("forced", "", Some(true), true),
            ("off", "public", Some(false), false),
        ];
        {
            let mut router = state.router.write().await;
            for (name, subpath, stored, _) in cases {
                let mut site = orcker_core::Site::parked(
                    name,
                    format!("/srv/{name}"),
                    orcker_core::PhpVersion::new(8, 3),
                )
                .unwrap();
                site.set_web_subpath(*subpath);
                site.set_front_controller(*stored);
                router.insert(site).unwrap();
            }
        }
        state
            .wordpress_sites
            .write()
            .await
            .insert("blog".to_owned(), true);

        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => {
                for (name, subpath, stored, expected) in cases {
                    let entry = sites
                        .iter()
                        .find(|s| s.site.name() == *name)
                        .unwrap_or_else(|| panic!("site {name} not listed"));
                    assert_eq!(
                        entry.uses_front_controller, *expected,
                        "{name}: subpath={subpath:?} override={stored:?}"
                    );
                }
                let blog = sites
                    .iter()
                    .find(|s| s.site.name() == "blog")
                    .unwrap_or_else(|| panic!("site blog not listed"));
                assert!(blog.is_wordpress, "the registry fact must reach the entry");
            }
            other => panic!("expected Sites, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn park_lists_child_and_persists() {
        let tmp = tempfile::tempdir().unwrap();
        let sites_root = tmp.path().join("sites");
        std::fs::create_dir_all(sites_root.join("blog")).unwrap();
        let state = state_in(tmp.path());

        let resp = dispatch(
            Request::Park {
                path: sites_root.clone(),
            },
            &state,
        )
        .await;
        assert!(matches!(resp, Response::Ok), "got {resp:?}");

        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => {
                let names: Vec<&str> = sites.iter().map(|e| e.site.name()).collect();
                assert_eq!(names, vec!["blog"]);
            }
            other => panic!("expected Sites, got {other:?}"),
        }
        assert!(state.config_path.exists());
        assert!(!state.config.lock().await.parked.paths.is_empty());
    }

    #[tokio::test]
    async fn link_then_duplicate_is_already_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let docroot = tmp.path().join("foo");
        std::fs::create_dir_all(&docroot).unwrap();
        let state = state_in(tmp.path());

        let ok = dispatch(
            Request::Link {
                name: "foo".into(),
                path: docroot.clone(),
            },
            &state,
        )
        .await;
        assert!(matches!(ok, Response::Ok), "got {ok:?}");

        let dup = dispatch(
            Request::Link {
                name: "foo".into(),
                path: docroot,
            },
            &state,
        )
        .await;
        match dup {
            Response::Error { code, .. } => assert_eq!(code, ErrorCode::AlreadyExists),
            other => panic!("expected AlreadyExists error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn link_auto_detects_web_root_for_laravel() {
        let tmp = tempfile::tempdir().unwrap();
        let docroot = tmp.path().join("app");
        std::fs::create_dir_all(docroot.join("public")).unwrap();
        std::fs::write(docroot.join("artisan"), b"").unwrap();
        std::fs::write(docroot.join("public/index.php"), b"").unwrap();
        let state = state_in(tmp.path());

        let ok = dispatch(
            Request::Link {
                name: "app".into(),
                path: docroot,
            },
            &state,
        )
        .await;
        assert!(matches!(ok, Response::Ok), "got {ok:?}");
        assert_eq!(
            web_subpath_of(&state, "app").await,
            std::path::PathBuf::from("public")
        );
    }

    #[tokio::test]
    async fn link_plain_php_serves_document_root() {
        let tmp = tempfile::tempdir().unwrap();
        let docroot = tmp.path().join("plain");
        std::fs::create_dir_all(&docroot).unwrap();
        let state = state_in(tmp.path());

        let ok = dispatch(
            Request::Link {
                name: "plain".into(),
                path: docroot,
            },
            &state,
        )
        .await;
        assert!(matches!(ok, Response::Ok), "got {ok:?}");
        assert_eq!(
            web_subpath_of(&state, "plain").await,
            std::path::PathBuf::new()
        );
    }

    #[tokio::test]
    async fn set_web_root_explicit_then_auto_on_linked_site() {
        let tmp = tempfile::tempdir().unwrap();
        let docroot = tmp.path().join("app");
        std::fs::create_dir_all(docroot.join("public")).unwrap();
        std::fs::write(docroot.join("artisan"), b"").unwrap();
        std::fs::write(docroot.join("public/index.php"), b"").unwrap();
        let state = state_in(tmp.path());

        dispatch(
            Request::Link {
                name: "app".into(),
                path: docroot.clone(),
            },
            &state,
        )
        .await;

        let ok = dispatch(
            Request::SetWebRoot {
                name: "app".into(),
                path: Some("public".into()),
            },
            &state,
        )
        .await;
        assert!(matches!(ok, Response::Ok), "got {ok:?}");
        let subpath = web_subpath_of(&state, "app").await;
        assert_eq!(subpath, std::path::PathBuf::from("public"));

        let ok = dispatch(
            Request::SetWebRoot {
                name: "app".into(),
                path: None,
            },
            &state,
        )
        .await;
        assert!(matches!(ok, Response::Ok), "got {ok:?}");
        assert_eq!(
            web_subpath_of(&state, "app").await,
            std::path::PathBuf::from("public")
        );
    }

    #[tokio::test]
    async fn set_web_root_outside_document_root_is_invalid_path() {
        let tmp = tempfile::tempdir().unwrap();
        let docroot = tmp.path().join("app");
        std::fs::create_dir_all(&docroot).unwrap();
        std::fs::create_dir_all(tmp.path().join("outside")).unwrap();
        let state = state_in(tmp.path());
        dispatch(
            Request::Link {
                name: "app".into(),
                path: docroot,
            },
            &state,
        )
        .await;

        let resp = dispatch(
            Request::SetWebRoot {
                name: "app".into(),
                path: Some(tmp.path().join("outside").to_string_lossy().into_owned()),
            },
            &state,
        )
        .await;
        match resp {
            Response::Error { code, .. } => assert_eq!(code, ErrorCode::InvalidPath),
            other => panic!("expected InvalidPath, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_web_root_unknown_site_is_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let resp = dispatch(
            Request::SetWebRoot {
                name: "ghost".into(),
                path: None,
            },
            &state,
        )
        .await;
        match resp {
            Response::Error { code, .. } => assert_eq!(code, ErrorCode::NotFound),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    /// Helper: read a site's web_subpath via `ListSites`.
    async fn web_subpath_of(state: &DaemonState, name: &str) -> std::path::PathBuf {
        match dispatch(Request::ListSites, state).await {
            Response::Sites { sites } => sites
                .iter()
                .find(|s| s.site.name() == name)
                .unwrap_or_else(|| panic!("site {name} not found"))
                .site
                .web_subpath()
                .to_path_buf(),
            other => panic!("expected Sites, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn park_nonexistent_is_invalid_path() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(
            Request::Park {
                path: tmp.path().join("does-not-exist"),
            },
            &state,
        )
        .await
        {
            Response::Error { code, .. } => assert_eq!(code, ErrorCode::InvalidPath),
            other => panic!("expected InvalidPath, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unlink_unknown_is_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(
            Request::Unlink {
                name: "ghost".into(),
            },
            &state,
        )
        .await
        {
            Response::Error { code, .. } => assert_eq!(code, ErrorCode::NotFound),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_parked_and_unpark_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let populated = tmp.path().join("populated");
        std::fs::create_dir_all(populated.join("blog")).unwrap();
        let empty = tmp.path().join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        let state = state_in(tmp.path());
        dispatch(Request::Park { path: populated }, &state).await;
        dispatch(Request::Park { path: empty }, &state).await;

        let parked = match dispatch(Request::ListParked, &state).await {
            Response::Parked { paths } => paths,
            other => panic!("expected Parked, got {other:?}"),
        };
        assert_eq!(parked.len(), 2, "both roots registered: {parked:?}");
        let mut sorted = parked.clone();
        sorted.sort();
        assert_eq!(parked, sorted, "ListParked must be sorted");
        let populated_root = parked
            .iter()
            .find(|p| p.ends_with("populated"))
            .unwrap()
            .clone();

        let resp = dispatch(
            Request::Unpark {
                path: populated_root.clone(),
            },
            &state,
        )
        .await;
        assert!(matches!(resp, Response::Ok), "got {resp:?}");

        match dispatch(Request::ListParked, &state).await {
            Response::Parked { paths } => {
                assert_eq!(paths.len(), 1);
                assert!(paths[0].ends_with("empty"));
            }
            other => panic!("expected Parked, got {other:?}"),
        }
        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => {
                assert!(
                    sites.iter().all(|s| s.site.name() != "blog"),
                    "blog should be gone after un-park: {sites:?}"
                );
            }
            other => panic!("expected Sites, got {other:?}"),
        }

        let resp = dispatch(
            Request::Unpark {
                path: populated_root,
            },
            &state,
        )
        .await;
        assert!(matches!(resp, Response::Ok), "absent un-park: got {resp:?}");
    }

    #[tokio::test]
    async fn set_secure_overrides_parked_keeping_kind_and_flips_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let sites_root = tmp.path().join("sites");
        std::fs::create_dir_all(sites_root.join("blog")).unwrap();
        let state = state_in(tmp.path());
        dispatch(Request::Park { path: sites_root }, &state).await;

        let resp = dispatch(
            Request::SetSecure {
                name: "Blog".into(),
                secure: true,
            },
            &state,
        )
        .await;
        assert!(matches!(resp, Response::Ok), "got {resp:?}");

        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => {
                let blog = sites.iter().find(|s| s.site.name() == "blog").unwrap();
                assert!(blog.site.secure());
                assert_eq!(blog.site.kind(), orcker_core::SiteKind::Parked);
            }
            other => panic!("expected Sites, got {other:?}"),
        }

        let resp = dispatch(
            Request::SetSecure {
                name: "blog".into(),
                secure: false,
            },
            &state,
        )
        .await;
        assert!(matches!(resp, Response::Ok), "got {resp:?}");
        match dispatch(Request::ListSites, &state).await {
            Response::Sites { sites } => {
                let blog = sites.iter().find(|s| s.site.name() == "blog").unwrap();
                assert!(!blog.site.secure());
            }
            other => panic!("expected Sites, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_daemon_info_reports_runtime_facts() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(Request::DaemonInfo, &state).await {
            Response::Info {
                dns_addr,
                tld,
                ca_path,
                ca_fingerprint,
                http_port,
                https_port,
                fallback_http,
                fallback_https,
                dns_port,
                lan_ip,
            } => {
                assert_eq!(dns_addr, state.dns_addr);
                assert_eq!(tld, "test");
                assert_eq!(ca_path, state.ca_path);
                assert_eq!(ca_fingerprint, state.ca_fingerprint.to_hex());
                assert_eq!(ca_fingerprint.len(), 64);
                assert_eq!(http_port, state.http.bound);
                assert_eq!(https_port, state.https.bound);
                assert_eq!(fallback_http, 8080);
                assert_eq!(fallback_https, 8443);
                assert_eq!(dns_port, state.config.lock().await.dns_port);
                assert_eq!(lan_ip, None, "LAN off by default -> no LAN IP");
            }
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_set_dns_port_rejects_zero_and_persists_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        match dispatch(Request::SetDnsPort { port: 0 }, &state).await {
            Response::Error { code, .. } => assert!(matches!(code, ErrorCode::Internal)),
            other => panic!("expected Error, got {other:?}"),
        }

        assert!(matches!(
            dispatch(Request::SetDnsPort { port: 5354 }, &state).await,
            Response::Ok
        ));
        assert_eq!(state.config.lock().await.dns_port, 5354);
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert_eq!(reloaded.dns_port, 5354);
    }

    #[tokio::test]
    async fn dispatch_group_mutations_persist_without_router_churn() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        for name in ["Blog", "Shop"] {
            assert!(matches!(
                dispatch(Request::CreateGroup { name: name.into() }, &state).await,
                Response::Ok
            ));
        }
        assert!(matches!(
            dispatch(
                Request::SetSiteGroup {
                    site: "api".into(),
                    group: Some("Blog".into()),
                },
                &state,
            )
            .await,
            Response::Ok
        ));

        // ListGroups reflects the mutations in memory...
        match dispatch(Request::ListGroups, &state).await {
            Response::Groups { order, members } => {
                assert_eq!(order, vec!["Blog".to_string(), "Shop".to_string()]);
                assert_eq!(members.get("api").map(String::as_str), Some("Blog"));
            }
            other => panic!("expected Groups, got {other:?}"),
        }
        // ...and they persisted to disk.
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert_eq!(
            reloaded.groups.order,
            vec!["Blog".to_string(), "Shop".to_string()]
        );
        assert_eq!(
            reloaded.groups.members.get("api").map(String::as_str),
            Some("Blog")
        );

        // Group mutations take the lighter commit path, so they must NOT signal
        // the parked-dir/router watcher (that would provoke a needless rescan).
        let not_notified = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            state.watch_dirty.notified(),
        )
        .await;
        assert!(
            not_notified.is_err(),
            "a group mutation must not notify watch_dirty"
        );

        // Contrast: a real site mutation DOES notify it - proving the probe works
        // and that the group path genuinely diverges from handle_mutation.
        let dir = tmp.path().join("sites");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(matches!(
            dispatch(Request::Park { path: dir }, &state).await,
            Response::Ok
        ));
        let notified = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            state.watch_dirty.notified(),
        )
        .await;
        assert!(
            notified.is_ok(),
            "a site mutation should notify watch_dirty"
        );
    }

    #[tokio::test]
    async fn dispatch_cached_update_status_uncached_reports_running_version() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        match dispatch(Request::CachedUpdateStatus, &state).await {
            Response::UpdateStatus {
                source,
                available,
                target,
                checked_at_epoch,
                ..
            } => {
                assert!(matches!(source, orcker_ipc::UpdateSource::Cached));
                assert!(!available);
                assert!(target.is_none());
                assert!(checked_at_epoch.is_none());
            }
            other => panic!("expected UpdateStatus, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn dispatch_restart_daemon_arms_flag_and_oks() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(!state
            .restart_requested
            .load(std::sync::atomic::Ordering::Acquire));
        let resp = dispatch(Request::RestartDaemon, &state).await;
        assert!(matches!(resp, Response::Ok), "got {resp:?}");
        assert!(state
            .restart_requested
            .load(std::sync::atomic::Ordering::Acquire));
    }

    // A tiny `latest.json` payload. Far-future versions so the target is always
    // newer than the daemon's compiled `current` version, regardless of the
    // build.
    const LATEST_MANIFEST: &str = r#"{
        "schema": 1,
        "stable": {"tag_name":"v99.0.1","prerelease":false,"draft":false,"assets":[]},
        "rc": {"tag_name":"v99.1.0-rc.1","prerelease":true,"draft":false,"assets":[]}
    }"#;

    /// Fake CDN downloader: serves a signed `latest.json` for the manifest URL
    /// and its detached signature for the `.minisig` URL. Feed `key()` to the
    /// self-update entry points as the verifying public key.
    struct ManifestDl(crate::test_support::SignedManifest);
    impl ManifestDl {
        fn new(manifest: &str) -> Self {
            Self(crate::test_support::sign_manifest(manifest))
        }
        fn key(&self) -> &str {
            &self.0.public_key
        }
    }

    #[async_trait::async_trait]
    impl crate::download::Downloader for ManifestDl {
        async fn download(&self, url: &str) -> Result<Vec<u8>, crate::download::DownloadError> {
            if url.ends_with(".minisig") {
                Ok(self.0.minisig.clone().into_bytes())
            } else {
                Ok(self.0.manifest.clone().into_bytes())
            }
        }
    }

    /// A downloader that fails every request, modelling being offline.
    /// Restored with the move SPEC-0002 made: `orcker_php::Downloader` ->
    /// `crate::download::Downloader`.
    struct FailingDl;
    #[async_trait::async_trait]
    impl crate::download::Downloader for FailingDl {
        async fn download(&self, url: &str) -> Result<Vec<u8>, crate::download::DownloadError> {
            Err(crate::download::DownloadError::Transport {
                url: url.to_owned(),
                reason: "boom".into(),
            })
        }
    }

    /// A *populated* cache must survive going offline: the two surviving
    /// `Cached` assertions both run against an empty cache
    /// (`check_update_rejects_tampered_manifest_signature` asserts
    /// `latest_stable == None`), so the fallback that actually serves a value
    /// had no test.
    #[tokio::test]
    async fn check_update_falls_back_to_cache_when_offline() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let dl = ManifestDl::new(LATEST_MANIFEST);
        crate::self_update::poll_and_refresh(&state, &dl, dl.key()).await;
        let resp = crate::self_update::check_update(None, &state, &FailingDl, dl.key()).await;
        match resp {
            Response::UpdateStatus {
                latest_stable,
                source,
                ..
            } => {
                assert_eq!(source, orcker_ipc::UpdateSource::Cached);
                assert_eq!(latest_stable.as_deref(), Some("99.0.1"));
            }
            other => panic!("expected UpdateStatus, got {other:?}"),
        }
    }

    /// `poll_and_refresh` is documented failure-tolerant: "a fetch error logs
    /// at `debug` and leaves the cache untouched". Only the happy path was
    /// exercised, by the test above. Retargeted from
    /// `poll_and_refresh_is_failure_tolerant`, which pinned the same guarantee
    /// on the deleted `php_updates::poll_and_refresh`.
    #[tokio::test]
    async fn poll_and_refresh_leaves_the_cache_untouched_on_a_fetch_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let dl = ManifestDl::new(LATEST_MANIFEST);
        crate::self_update::poll_and_refresh(&state, &dl, dl.key()).await;
        let before = state.orcker_update.read().await.len();
        assert!(before > 0, "the cache must be populated before the failure");

        crate::self_update::poll_and_refresh(&state, &FailingDl, dl.key()).await;
        assert_eq!(
            state.orcker_update.read().await.len(),
            before,
            "a failed fetch must not clear the cache"
        );
    }

    /// The `DoctorFix` dispatch path: nothing is auto-fixed - phase 0 has no
    /// auto-fixable finding left, so `plan_auto_fixes` is a stub - and every
    /// finding still reaches the user through `manual`. `Request::DoctorFix` is
    /// dispatched at `ipc_server.rs:201` and no test reached it.
    ///
    /// Scrubbed: the original also asserted a `Severity::Fail` in `manual`, which
    /// relied on `NoPhpInstalled`. A fresh daemon's findings are now
    /// `PortFallback`, `CaNotTrusted`, `CaNotTrustedByBrowsers` and
    /// `ResolverNotInstalled`, all `Warn`.
    #[tokio::test]
    async fn dispatch_doctor_fix_performs_nothing_and_reports_manual() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        match dispatch(Request::DoctorFix, &state).await {
            Response::DoctorFix { report } => {
                assert!(report.performed.is_empty(), "{:?}", report.performed);
                assert!(!report.manual.is_empty(), "{:?}", report.manual);
            }
            other => panic!("expected DoctorFix, got {other:?}"),
        }
    }
    #[tokio::test]
    async fn check_update_reports_both_channel_latests_live() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let dl = ManifestDl::new(LATEST_MANIFEST);
        let resp = crate::self_update::check_update(None, &state, &dl, dl.key()).await;
        match resp {
            Response::UpdateStatus {
                latest_stable,
                latest_edge,
                channel,
                source,
                ..
            } => {
                assert_eq!(latest_stable.as_deref(), Some("99.0.1"));
                assert_eq!(latest_edge.as_deref(), Some("99.1.0-rc.1"));
                assert_eq!(channel, orcker_ipc::Channel::Stable);
                assert_eq!(source, orcker_ipc::UpdateSource::Live);
            }
            other => panic!("expected UpdateStatus, got {other:?}"),
        }
        assert_eq!(state.orcker_update.read().await.len(), 2);
    }

    #[tokio::test]
    async fn check_update_edge_override_selects_prerelease_target() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let dl = ManifestDl::new(LATEST_MANIFEST);
        let resp = crate::self_update::check_update(
            Some(orcker_ipc::Channel::Edge),
            &state,
            &dl,
            dl.key(),
        )
        .await;
        match resp {
            Response::UpdateStatus {
                channel,
                target,
                available,
                ..
            } => {
                assert_eq!(channel, orcker_ipc::Channel::Edge);
                assert_eq!(target.as_deref(), Some("99.1.0-rc.1"));
                assert!(available);
            }
            other => panic!("expected UpdateStatus, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn check_update_rejects_tampered_manifest_signature() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        // Sign the manifest with one key, but verify against a *different* key:
        // the signature must fail and the live fetch must yield nothing.
        let dl = ManifestDl::new(LATEST_MANIFEST);
        let other = crate::test_support::sign_manifest(LATEST_MANIFEST);
        let resp = crate::self_update::check_update(None, &state, &dl, &other.public_key).await;
        match resp {
            Response::UpdateStatus {
                latest_stable,
                source,
                ..
            } => {
                assert_eq!(source, orcker_ipc::UpdateSource::Cached);
                assert_eq!(
                    latest_stable, None,
                    "a bad-signature manifest must not be trusted"
                );
            }
            other => panic!("expected UpdateStatus, got {other:?}"),
        }
        assert!(state.orcker_update.read().await.is_empty());
    }

    /// Fake downloader for the full stage flow. Serves the signed `latest.json`
    /// (and its `.minisig`) for the manifest URLs; the signed bytes (`b"test"`)
    /// for any artifact URL; the matching artifact signature for any `.sig` /
    /// `.minisig` URL; and a `SHA256SUMS`. The manifest and the artifact bytes
    /// are signed with a single key ([`StageDl::key`]), which is fed to
    /// `stage_update` to verify both the manifest and the artifact.
    struct StageDl {
        manifest: crate::test_support::SignedManifest,
        artifact_sig: String,
        sums: String,
    }

    #[async_trait::async_trait]
    impl crate::download::Downloader for StageDl {
        async fn download(&self, url: &str) -> Result<Vec<u8>, crate::download::DownloadError> {
            if url.contains("latest.json.minisig") {
                Ok(self.manifest.minisig.clone().into_bytes())
            } else if url.contains("latest.json") {
                Ok(self.manifest.manifest.clone().into_bytes())
            } else if url.ends_with("SHA256SUMS") {
                Ok(self.sums.clone().into_bytes())
            } else if url.ends_with(".minisig") || url.ends_with(".sig") {
                Ok(self.artifact_sig.clone().into_bytes())
            } else {
                Ok(b"test".to_vec())
            }
        }
    }
    impl StageDl {
        /// Build from a `stable` release object body and a `SHA256SUMS`. The
        /// manifest wraps `stable` as `latest.json`; the artifact bytes `b"test"`
        /// are signed with the same key so one public key verifies both.
        fn new(stable: &str, sums: String) -> Self {
            let manifest = format!(r#"{{"schema":1,"stable":{stable},"rc":null}}"#);
            let (manifest, artifact) = crate::test_support::sign_manifest_pair(&manifest, "test");
            Self {
                manifest,
                artifact_sig: artifact.minisig,
                sums,
            }
        }
        fn key(&self) -> &str {
            &self.manifest.public_key
        }
    }
    #[tokio::test]
    async fn stage_update_downloads_verifies_and_writes_artifact() {
        if !matches!(
            orcker_update::Platform::current(),
            orcker_update::Platform::MacOsAarch64
                | orcker_update::Platform::LinuxX86_64
                | orcker_update::Platform::LinuxAarch64
        ) {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        let mac = "Orcker_MacOS_AppleSilicon_v99-0-1.app.tar.gz";
        let deb = "Orcker_Linux_x86_64_v99-0-1.deb";
        let arm = "Orcker_Linux_Arm64_v99-0-1.deb";
        let pkg = "Orcker_Linux_x86_64_v99-0-1.pkg.tar.zst";
        let pkg_arm = "Orcker_Linux_Arm64_v99-0-1.pkg.tar.zst";
        let rpm = "Orcker_Linux_x86_64_v99-0-1.rpm";
        let rpm_arm = "Orcker_Linux_Arm64_v99-0-1.rpm";
        let stable = format!(
            r#"{{"tag_name":"v99.0.1","prerelease":false,"draft":false,"assets":[
                {{"name":"{mac}","browser_download_url":"https://h/{mac}","size":4}},
                {{"name":"{mac}.minisig","browser_download_url":"https://h/{mac}.minisig","size":1}},
                {{"name":"{deb}","browser_download_url":"https://h/{deb}","size":4}},
                {{"name":"{deb}.minisig","browser_download_url":"https://h/{deb}.minisig","size":1}},
                {{"name":"{arm}","browser_download_url":"https://h/{arm}","size":4}},
                {{"name":"{arm}.minisig","browser_download_url":"https://h/{arm}.minisig","size":1}},
                {{"name":"{pkg}","browser_download_url":"https://h/{pkg}","size":4}},
                {{"name":"{pkg}.minisig","browser_download_url":"https://h/{pkg}.minisig","size":1}},
                {{"name":"{pkg_arm}","browser_download_url":"https://h/{pkg_arm}","size":4}},
                {{"name":"{pkg_arm}.minisig","browser_download_url":"https://h/{pkg_arm}.minisig","size":1}},
                {{"name":"{rpm}","browser_download_url":"https://h/{rpm}","size":4}},
                {{"name":"{rpm}.minisig","browser_download_url":"https://h/{rpm}.minisig","size":1}},
                {{"name":"{rpm_arm}","browser_download_url":"https://h/{rpm_arm}","size":4}},
                {{"name":"{rpm_arm}.minisig","browser_download_url":"https://h/{rpm_arm}.minisig","size":1}},
                {{"name":"SHA256SUMS","browser_download_url":"https://h/SHA256SUMS","size":1}}
            ]}}"#
        );
        let h = orcker_update::sha256_hex(b"test");
        let sums = format!(
            "{h}  {mac}\n{h}  {deb}\n{h}  {arm}\n{h}  {pkg}\n{h}  {pkg_arm}\n{h}  {rpm}\n{h}  {rpm_arm}\n"
        );
        let dl = StageDl::new(&stable, sums);

        let resp = crate::self_update::stage_update(None, &state, &dl, dl.key()).await;
        match resp {
            Response::Staged {
                path,
                version,
                kind,
            } => {
                assert_eq!(version, "99.0.1");
                let p = std::path::Path::new(&path);
                assert!(p.exists(), "staged file should exist at {path}");
                assert_eq!(std::fs::read(p).unwrap(), b"test");
                let sibling = p.with_file_name(format!(
                    "{}.minisig",
                    p.file_name().and_then(|n| n.to_str()).unwrap()
                ));
                assert!(
                    sibling.exists(),
                    "staged .minisig sibling should exist at {}",
                    sibling.display()
                );
                let (expected_kind, expected_name) = match (
                    orcker_update::Platform::current(),
                    orcker_update::PkgFormat::current(),
                ) {
                    (orcker_update::Platform::MacOsAarch64, _) => {
                        (orcker_ipc::StagedArtifact::AppTarGz, mac)
                    }
                    (orcker_update::Platform::LinuxX86_64, orcker_update::PkgFormat::Deb) => {
                        (orcker_ipc::StagedArtifact::Deb, deb)
                    }
                    (orcker_update::Platform::LinuxX86_64, orcker_update::PkgFormat::Pacman) => {
                        (orcker_ipc::StagedArtifact::Pacman, pkg)
                    }
                    (orcker_update::Platform::LinuxAarch64, orcker_update::PkgFormat::Deb) => {
                        (orcker_ipc::StagedArtifact::Deb, arm)
                    }
                    (orcker_update::Platform::LinuxAarch64, orcker_update::PkgFormat::Pacman) => {
                        (orcker_ipc::StagedArtifact::Pacman, pkg_arm)
                    }
                    (orcker_update::Platform::LinuxX86_64, orcker_update::PkgFormat::Rpm) => {
                        (orcker_ipc::StagedArtifact::Rpm, rpm)
                    }
                    (orcker_update::Platform::LinuxAarch64, orcker_update::PkgFormat::Rpm) => {
                        (orcker_ipc::StagedArtifact::Rpm, rpm_arm)
                    }
                    (other, _) => panic!("unexpected platform for fixture: {other:?}"),
                };
                assert_eq!(kind, expected_kind);
                assert_eq!(
                    p.file_name().and_then(|n| n.to_str()),
                    Some(expected_name),
                    "staged basename should be the current platform+format's asset"
                );
            }
            other => panic!("expected Staged, got {other:?}"),
        }
    }

    /// A pre-N release layout carries only `.sig` signatures. An N-built client
    /// must still stage from it via the `select_asset` `.sig` fallback, and it
    /// always writes the sibling under the new `.minisig` name.
    #[tokio::test]
    async fn stage_update_stages_from_legacy_sig_only_layout() {
        if !matches!(
            orcker_update::Platform::current(),
            orcker_update::Platform::MacOsAarch64
                | orcker_update::Platform::LinuxX86_64
                | orcker_update::Platform::LinuxAarch64
        ) {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        let mac = "Orcker_MacOS_AppleSilicon_v99-0-1.app.tar.gz";
        let deb = "Orcker_Linux_x86_64_v99-0-1.deb";
        let arm = "Orcker_Linux_Arm64_v99-0-1.deb";
        let pkg = "Orcker_Linux_x86_64_v99-0-1.pkg.tar.zst";
        let pkg_arm = "Orcker_Linux_Arm64_v99-0-1.pkg.tar.zst";
        let rpm = "Orcker_Linux_x86_64_v99-0-1.rpm";
        let rpm_arm = "Orcker_Linux_Arm64_v99-0-1.rpm";
        let stable = format!(
            r#"{{"tag_name":"v99.0.1","prerelease":false,"draft":false,"assets":[
                {{"name":"{mac}","browser_download_url":"https://h/{mac}","size":4}},
                {{"name":"{mac}.sig","browser_download_url":"https://h/{mac}.sig","size":1}},
                {{"name":"{deb}","browser_download_url":"https://h/{deb}","size":4}},
                {{"name":"{deb}.sig","browser_download_url":"https://h/{deb}.sig","size":1}},
                {{"name":"{arm}","browser_download_url":"https://h/{arm}","size":4}},
                {{"name":"{arm}.sig","browser_download_url":"https://h/{arm}.sig","size":1}},
                {{"name":"{pkg}","browser_download_url":"https://h/{pkg}","size":4}},
                {{"name":"{pkg}.sig","browser_download_url":"https://h/{pkg}.sig","size":1}},
                {{"name":"{pkg_arm}","browser_download_url":"https://h/{pkg_arm}","size":4}},
                {{"name":"{pkg_arm}.sig","browser_download_url":"https://h/{pkg_arm}.sig","size":1}},
                {{"name":"{rpm}","browser_download_url":"https://h/{rpm}","size":4}},
                {{"name":"{rpm}.sig","browser_download_url":"https://h/{rpm}.sig","size":1}},
                {{"name":"{rpm_arm}","browser_download_url":"https://h/{rpm_arm}","size":4}},
                {{"name":"{rpm_arm}.sig","browser_download_url":"https://h/{rpm_arm}.sig","size":1}},
                {{"name":"SHA256SUMS","browser_download_url":"https://h/SHA256SUMS","size":1}}
            ]}}"#
        );
        let h = orcker_update::sha256_hex(b"test");
        let sums = format!(
            "{h}  {mac}\n{h}  {deb}\n{h}  {arm}\n{h}  {pkg}\n{h}  {pkg_arm}\n{h}  {rpm}\n{h}  {rpm_arm}\n"
        );
        let dl = StageDl::new(&stable, sums);

        match crate::self_update::stage_update(None, &state, &dl, dl.key()).await {
            Response::Staged { path, .. } => {
                let p = std::path::Path::new(&path);
                assert!(p.exists(), "staged file should exist at {path}");
                let sibling = p.with_file_name(format!(
                    "{}.minisig",
                    p.file_name().and_then(|n| n.to_str()).unwrap()
                ));
                assert!(
                    sibling.exists(),
                    "staged sibling should be written as .minisig even from a .sig layout"
                );
            }
            other => panic!("expected Staged from legacy .sig layout, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn stage_update_rejects_verification_failure_and_writes_nothing() {
        if !matches!(
            orcker_update::Platform::current(),
            orcker_update::Platform::MacOsAarch64
                | orcker_update::Platform::LinuxX86_64
                | orcker_update::Platform::LinuxAarch64
        ) {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        let mac = "Orcker_MacOS_AppleSilicon_v99-0-1.app.tar.gz";
        let deb = "Orcker_Linux_x86_64_v99-0-1.deb";
        let arm = "Orcker_Linux_Arm64_v99-0-1.deb";
        let pkg = "Orcker_Linux_x86_64_v99-0-1.pkg.tar.zst";
        let pkg_arm = "Orcker_Linux_Arm64_v99-0-1.pkg.tar.zst";
        let rpm = "Orcker_Linux_x86_64_v99-0-1.rpm";
        let rpm_arm = "Orcker_Linux_Arm64_v99-0-1.rpm";
        let stable = format!(
            r#"{{"tag_name":"v99.0.1","prerelease":false,"draft":false,"assets":[
                {{"name":"{mac}","browser_download_url":"https://h/{mac}","size":4}},
                {{"name":"{mac}.minisig","browser_download_url":"https://h/{mac}.minisig","size":1}},
                {{"name":"{deb}","browser_download_url":"https://h/{deb}","size":4}},
                {{"name":"{deb}.minisig","browser_download_url":"https://h/{deb}.minisig","size":1}},
                {{"name":"{arm}","browser_download_url":"https://h/{arm}","size":4}},
                {{"name":"{arm}.minisig","browser_download_url":"https://h/{arm}.minisig","size":1}},
                {{"name":"{pkg}","browser_download_url":"https://h/{pkg}","size":4}},
                {{"name":"{pkg}.minisig","browser_download_url":"https://h/{pkg}.minisig","size":1}},
                {{"name":"{pkg_arm}","browser_download_url":"https://h/{pkg_arm}","size":4}},
                {{"name":"{pkg_arm}.minisig","browser_download_url":"https://h/{pkg_arm}.minisig","size":1}},
                {{"name":"{rpm}","browser_download_url":"https://h/{rpm}","size":4}},
                {{"name":"{rpm}.minisig","browser_download_url":"https://h/{rpm}.minisig","size":1}},
                {{"name":"{rpm_arm}","browser_download_url":"https://h/{rpm_arm}","size":4}},
                {{"name":"{rpm_arm}.minisig","browser_download_url":"https://h/{rpm_arm}.minisig","size":1}},
                {{"name":"SHA256SUMS","browser_download_url":"https://h/SHA256SUMS","size":1}}
            ]}}"#
        );
        let bad = "0".repeat(64);
        let sums = format!(
            "{bad}  {mac}\n{bad}  {deb}\n{bad}  {arm}\n{bad}  {pkg}\n{bad}  {pkg_arm}\n{bad}  {rpm}\n{bad}  {rpm_arm}\n"
        );
        let dl = StageDl::new(&stable, sums);
        match crate::self_update::stage_update(None, &state, &dl, dl.key()).await {
            Response::Error { message, .. } => assert!(
                message.contains("checksum verification failed"),
                "expected a checksum verification failure, got: {message}"
            ),
            other => panic!("expected Error on checksum mismatch, got {other:?}"),
        }
        assert!(
            !state.dirs.cache.join("update").join(mac).exists()
                && !state.dirs.cache.join("update").join(deb).exists()
                && !state.dirs.cache.join("update").join(arm).exists()
                && !state.dirs.cache.join("update").join(pkg).exists()
                && !state.dirs.cache.join("update").join(pkg_arm).exists()
                && !state.dirs.cache.join("update").join(rpm).exists()
                && !state.dirs.cache.join("update").join(rpm_arm).exists(),
            "must not write an artifact when verification fails"
        );
    }

    #[tokio::test]
    async fn set_update_channel_persists_to_config() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert_eq!(
            dispatch(
                Request::SetUpdateChannel {
                    channel: orcker_ipc::Channel::Edge,
                },
                &state,
            )
            .await,
            Response::Ok
        );
        assert_eq!(state.config.lock().await.update_channel, "edge");
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert_eq!(reloaded.update_channel, "edge");
    }

    #[tokio::test]
    async fn set_symlink_protection_persists_config_and_updates_live_atomic() {
        use std::sync::atomic::Ordering::Relaxed;
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(state.symlink_protection.load(Relaxed), "seeded protected");

        assert_eq!(
            dispatch(Request::SetSymlinkProtection { enabled: false }, &state,).await,
            Response::Ok
        );
        assert!(
            !state.config.lock().await.symlink_protection,
            "in-memory config off"
        );
        assert!(!state.symlink_protection.load(Relaxed), "live atomic off");
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert!(!reloaded.symlink_protection, "persisted config off");
        assert!(
            !build_status_report(&state).await.symlink_protection,
            "status report off"
        );

        assert_eq!(
            dispatch(Request::SetSymlinkProtection { enabled: true }, &state).await,
            Response::Ok
        );
        assert!(
            state.symlink_protection.load(Relaxed),
            "live atomic back on"
        );
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert!(reloaded.symlink_protection, "persisted config back on");
        assert!(
            build_status_report(&state).await.symlink_protection,
            "status report back on"
        );
    }

    #[tokio::test]
    async fn set_mcp_enabled_persists_config_and_appears_in_status() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(
            !state.config.lock().await.mcp_enabled,
            "seeded opt-in default off"
        );
        assert!(
            !build_status_report(&state).await.mcp_enabled,
            "status report starts off"
        );

        assert_eq!(
            dispatch(Request::SetMcpEnabled { enabled: true }, &state).await,
            Response::Ok
        );
        assert!(state.config.lock().await.mcp_enabled, "in-memory config on");
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert!(reloaded.mcp_enabled, "persisted config on");
        assert!(
            build_status_report(&state).await.mcp_enabled,
            "status report on"
        );

        assert_eq!(
            dispatch(Request::SetMcpEnabled { enabled: false }, &state).await,
            Response::Ok
        );
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert!(!reloaded.mcp_enabled, "persisted config back off");
        assert!(
            !build_status_report(&state).await.mcp_enabled,
            "status report back off"
        );
    }

    #[tokio::test]
    async fn set_lan_enabled_persists_config_and_appears_in_status() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(!state.config.lock().await.lan_enabled, "seeded off");
        assert!(!build_status_report(&state).await.lan_enabled);

        assert_eq!(
            dispatch(Request::SetLanEnabled { enabled: true }, &state).await,
            Response::Ok
        );
        assert!(state.config.lock().await.lan_enabled, "in-memory config on");
        let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
        assert!(reloaded.lan_enabled, "persisted config on");
        assert!(build_status_report(&state).await.lan_enabled);

        assert_eq!(
            dispatch(Request::SetLanEnabled { enabled: false }, &state).await,
            Response::Ok
        );
        assert!(!state.config.lock().await.lan_enabled, "back off");
    }

    #[tokio::test]
    async fn disabling_lan_revokes_a_pending_remote_setup_code() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        *state.remote_setup_code.lock().await = Some(crate::state::RemoteSetupCode {
            value: "abc".to_owned(),
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(60),
            used: false,
        });
        assert_eq!(
            dispatch(Request::SetLanEnabled { enabled: false }, &state).await,
            Response::Ok
        );
        assert!(
            state.remote_setup_code.lock().await.is_none(),
            "disabling LAN clears any pending one-time code"
        );
    }

    #[tokio::test]
    async fn mint_remote_setup_code_rejects_when_lan_off() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        // LAN off by default → guarded.
        match dispatch(Request::MintRemoteSetupCode, &state).await {
            Response::Error { code, .. } => {
                assert_eq!(code, orcker_ipc::ErrorCode::LanNotReady);
            }
            other => panic!("expected LanNotReady error, got {other:?}"),
        }
    }

    // ---------- pure helpers ----------

    #[test]
    fn load_to_centi_clamps_and_rounds() {
        assert_eq!(load_to_centi(0.0), 0);
        assert_eq!(load_to_centi(-5.0), 0, "negative clamps to 0");
        assert_eq!(load_to_centi(1.234), 123, "rounded to hundredths");
        assert_eq!(load_to_centi(f64::from(u32::MAX)), u32::MAX, "saturates");
    }

    #[test]
    fn path_needs_setup_no_tools_is_some_false() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        #[cfg(unix)]
        assert_eq!(path_needs_setup(&state), Some(false));
        #[cfg(not(unix))]
        assert_eq!(path_needs_setup(&state), None);
    }

    // ---------- additional `dispatch` arms ----------

    #[tokio::test]
    async fn dispatch_set_fallback_ports_validates_and_persists() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());

        match dispatch(
            Request::SetFallbackPorts {
                http: 9000,
                https: 9000,
            },
            &state,
        )
        .await
        {
            Response::Error { code, .. } => assert!(matches!(code, ErrorCode::Internal)),
            other => panic!("expected Error, got {other:?}"),
        }

        match dispatch(
            Request::SetFallbackPorts {
                http: 8081,
                https: 8444,
            },
            &state,
        )
        .await
        {
            Response::Ok => {
                {
                    let cfg = state.config.lock().await;
                    assert_eq!(cfg.ports.fallback_http, 8081);
                    assert_eq!(cfg.ports.fallback_https, 8444);
                }
                let reloaded = orcker_config::Config::load(&state.config_path).unwrap();
                assert_eq!(reloaded.ports.fallback_http, 8081);
                assert_eq!(reloaded.ports.fallback_https, 8444);
            }
            Response::Error { code, message } => {
                assert!(matches!(code, ErrorCode::Internal));
                assert!(message.contains("elevated"), "{message}");
                let cfg = state.config.lock().await;
                assert_eq!(cfg.ports.fallback_http, 8080);
                assert_eq!(cfg.ports.fallback_https, 8443);
            }
            other => panic!("expected Ok or elevated Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_delete_mails_empty_is_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let state = state_in(tmp.path());
        assert!(matches!(
            dispatch(Request::DeleteMails { ids: vec![] }, &state).await,
            Response::Ok
        ));
    }

    // ---------- dump-server arms routed through `dispatch` ----------
}
