//! Orcker CLI - a thin `orcker-ipc` client of the `orckerd` daemon.
//!
//! Binary-only crates don't expose a Rust API to integration tests under
//! `tests/`. This lib publishes the CLI's modules so `tests/cli_e2e.rs` can
//! drive the pure mapping (`map`) and the transport (`transport`) against a
//! daemon booted on a tempdir. All behaviour lives in the modules; `main.rs`
//! is a thin wrapper around [`run`].

#![forbid(unsafe_code)]

pub mod apply;
pub mod cli;
#[cfg(unix)]
#[cfg(unix)]
#[cfg(unix)]
pub mod elevate;
pub mod error;
#[cfg(unix)]
#[cfg(unix)]
pub mod map;
pub mod mcp_cmd;
pub mod path_cmd;
#[cfg(unix)]
#[cfg(unix)]
pub mod transport;
pub mod uninstall;
#[cfg(unix)]
use std::process::ExitCode;

pub use error::ClientError;

use cli::{Cli, Command};

/// Map the parsed command to a request, exchange it with the daemon, and
/// render the response. Returns the process exit code:
/// `0` success, `1` daemon error response, `2` usage error, `69` daemon
/// unreachable, `74` other transport/IO failure.
#[allow(clippy::too_many_lines)]
pub async fn run(cli: Cli) -> ExitCode {
    match &cli.command {
        Command::Elevate { target } => return elevate::run_elevate(*target, false).await,
        Command::Unelevate { target } => return elevate::run_elevate(*target, true).await,
        Command::Path { action } => return path_cmd::run(*action),
        Command::Mcp => return mcp_cmd::run().await,
        #[cfg_attr(not(unix), allow(unused_variables))]
        #[cfg_attr(not(unix), allow(unused_variables))]
        #[cfg_attr(not(unix), allow(unused_variables))]
        Command::Domain {
            action: crate::cli::DomainAction::List { site },
        } => return run_domain_list(site.as_deref(), cli.json).await,
        Command::Sites => return run_sites(cli.json).await,
        Command::Status => return run_status(cli.json).await,
        Command::Route {
            action: crate::cli::RouteAction::List { site: Some(site) },
        } => return run_route_list(site, cli.json).await,
        Command::Uninstall { target: None, yes } => return uninstall::run(*yes),
        Command::Install {
            target: crate::cli::InstallTarget::Tool { id },
        } if !cli.json => return stream_install_tool(id, cli.json).await,
        Command::Tunnel {
            action: crate::cli::TunnelAction::Install,
        } if !cli.json => {
            return stream_tunnel_job(orcker_ipc::Request::InstallCloudflaredStreamed).await
        }
        Command::Tunnel {
            action: crate::cli::TunnelAction::Login,
        } if !cli.json => return stream_tunnel_job(orcker_ipc::Request::CloudflaredLogin).await,
        Command::Lan {
            action: crate::cli::LanAction::Enable,
        } => return run_lan_toggle(true, cli.json).await,
        Command::Lan {
            action: crate::cli::LanAction::Disable,
        } => return run_lan_toggle(false, cli.json).await,
        Command::Lan {
            action: crate::cli::LanAction::Status,
        } => return run_lan_status(cli.json).await,
        Command::Update {
            yes: true,
            edge,
            stable,
            force,
        } => return run_self_update_apply(cli.json, *edge, *stable, *force).await,
        _ => {}
    }

    let req = match &cli.command {
        Command::Link { path, name, port } => {
            resolve_link_project(path.as_deref(), name.as_deref(), *port)
        }
        _ => map::to_request(&cli.command)
            .map(canonicalize_unpark)
            .and_then(canonicalize_park_path),
    };
    let req = match req {
        Ok(r) => r,
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(2);
        }
    };

    match transport::exchange(&req).await {
        Ok(resp) => {
            let r = map::render(&resp, cli.json);
            if !r.stdout.is_empty() {
                println!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprintln!("{}", r.stderr);
            }
            if r.code == 0
                && matches!(
                    cli.command,
                    Command::Install {
                        target: crate::cli::InstallTarget::Tool { .. }
                    }
                )
            {
                path_cmd::ensure_installed_after_tool(cli.json);
            }
            ExitCode::from(r.code)
        }
        Err(e) if e.is_daemon_down() => {
            if matches!(cli.command, Command::Doctor { .. }) {
                let resp = daemon_down_response();
                let r = map::render(&resp, cli.json);
                if !r.stdout.is_empty() {
                    println!("{}", r.stdout);
                }
                return ExitCode::from(r.code);
            }
            eprintln!("orcker: {e}");
            ExitCode::from(69)
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            ExitCode::from(74)
        }
    }
}

/// `orcker lan enable|disable`: a two-request flow that persists the flag then
/// **enforces** the daemon restart that re-binds the listeners (a listen
/// socket's bind address is fixed at bind time, so a hint is not enough for a
/// security-toggling command). Captures `boot_id` before, sends `RestartDaemon`,
/// and polls `Status` across the re-exec socket gap until `boot_id` changes.
async fn run_lan_toggle(enabled: bool, json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};

    let before = match fetch_boot_id().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
    };

    match transport::exchange(&Request::SetLanEnabled { enabled }).await {
        Ok(Response::Ok) => {}
        Ok(Response::Error { message, .. }) => {
            eprintln!("orcker: {message}");
            return ExitCode::from(1);
        }
        Ok(other) => {
            eprintln!("orcker: unexpected response: {other:?}");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
    }

    if let Err(e) = restart_and_await_boot_change(before).await {
        eprintln!(
            "orcker: LAN {} was saved, but the daemon restart could not be confirmed: {e}",
            if enabled { "enable" } else { "disable" }
        );
        eprintln!("      restart it manually so the change takes effect.");
        return ExitCode::from(74);
    }

    if json {
        println!("{{\"lan_enabled\":{enabled},\"restarted\":true}}");
        return ExitCode::SUCCESS;
    }

    if enabled {
        println!("LAN exposure enabled and the daemon restarted.");
        println!();
        if cfg!(target_os = "macos") {
            println!("Next: install the LAN redirect (one-time, needs root):");
            println!("    sudo orcker elevate lan");
            println!(
                "(this also requires `sudo orcker elevate ports` — run it first if you haven't)."
            );
        } else {
            println!("Ensure `sudo orcker elevate ports` has been run so 80/443 bind, and open");
            println!("80/443/1053 to your LAN in the host firewall (see `orcker lan status`).");
        }
        println!();
        println!("Then provision a device with:  orcker remote-setup");
        println!("Check exposure at any time with:  orcker lan status");
    } else {
        println!(
            "LAN exposure disabled and the daemon restarted (listeners are back on loopback)."
        );
        if cfg!(target_os = "macos") {
            println!();
            println!("The macOS pf LAN redirect is separate privileged state — remove it with:");
            println!("    sudo orcker unelevate lan");
            println!("Until you do, `orcker lan status` will flag it as residual.");
        }
    }
    ExitCode::SUCCESS
}

/// `orcker status`: the daemon health report plus the Docker section.
///
/// The only command that needs two responses. `EngineStatus` is a separate
/// request because the probe talks to the Docker daemon, so its answer comes
/// from a cache the daemon owns rather than being rebuilt inside every
/// `StatusReport`. Anything other than an `EngineStatus` reply leaves the
/// section absent, puts the real reason on stderr (see [`map::docker_section`])
/// and still exits with the daemon report's code - a stopped engine is
/// something `orcker status` reports, not something it exits non-zero on (R8).
async fn run_status(json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};
    let report = match transport::exchange(&Request::Status).await {
        Ok(Response::Status { report }) => report,
        Ok(other) => {
            eprintln!("orcker: unexpected response: {other:?}");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
    };
    let (docker, note) = map::docker_section(transport::exchange(&Request::EngineStatus).await);
    if let Some(note) = note {
        eprintln!("orcker: {note}");
    }

    let r = map::render_status(&report, docker.as_ref(), json);
    if !r.stdout.is_empty() {
        println!("{}", r.stdout);
    }
    if !r.stderr.is_empty() {
        eprintln!("{}", r.stderr);
    }
    ExitCode::from(r.code)
}

/// `orcker sites`: on-disk sites and container projects in one view.
///
/// Two requests, one rendering, the same shape as [`run_status`]: projects live
/// in their own reply rather than as a field on `Response::Sites`. A daemon that
/// does not know `ListProjects` simply contributes no projects.
async fn run_sites(json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};
    let sites = match transport::exchange(&Request::ListSites).await {
        Ok(Response::Sites { sites }) => sites,
        Ok(other) => {
            eprintln!("orcker: unexpected response: {other:?}");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
    };
    let projects = match transport::exchange(&Request::ListProjects).await {
        Ok(Response::Projects { projects }) => projects,
        _ => Vec::new(),
    };

    let r = map::render_sites(&sites, &projects, json);
    if !r.stdout.is_empty() {
        println!("{}", r.stdout);
    }
    if !r.stderr.is_empty() {
        eprintln!("{}", r.stderr);
    }
    ExitCode::from(r.code)
}

/// `orcker lan status`: a LAN-focused view of the daemon's `Status`, showing
/// configured-vs-effective state so "enabled but not exposed" (and, on macOS,
/// "disabled but pf still redirecting") are both visible.
async fn run_lan_status(json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};
    let report = match transport::exchange(&Request::Status).await {
        Ok(Response::Status { report }) => report,
        Ok(other) => {
            eprintln!("orcker: unexpected response: {other:?}");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
    };

    if json {
        let ip = report
            .lan_ip
            .map_or_else(|| "null".to_owned(), |i| format!("\"{i}\""));
        let bound = report
            .lan_setup_bound
            .map_or_else(|| "null".to_owned(), |b| b.to_string());
        println!(
            "{{\"lan_enabled\":{},\"lan_ip\":{ip},\"lan_setup_bound\":{bound}}}",
            report.lan_enabled
        );
        return ExitCode::SUCCESS;
    }

    if !report.lan_enabled {
        println!("LAN exposure: OFF (sites are served on loopback only).");
        println!("Enable it with:  orcker lan enable");
        #[cfg(target_os = "macos")]
        println!(
            "Note: if you previously ran `sudo orcker elevate lan`, remove the residual pf rule \
             with `sudo orcker unelevate lan`."
        );
        return ExitCode::SUCCESS;
    }

    println!("LAN exposure: ON (configured).");
    match report.lan_ip {
        Some(ip) => println!("  LAN address:      {ip}"),
        None => println!("  LAN address:      <discovery failed — answers fall back to loopback>"),
    }
    match report.lan_setup_bound {
        Some(true) => println!(
            "  Bootstrap:        listening (run `orcker remote-setup` to provision a device)"
        ),
        Some(false) => {
            println!("  Bootstrap:        NOT bound (port busy? check `lan_setup_port`)");
        }
        None => {}
    }
    if cfg!(target_os = "macos") {
        println!(
            "  macOS redirect:   run `sudo orcker elevate lan` (and `elevate ports`) if 80/443 \
             aren't reachable from the LAN yet."
        );
    } else {
        println!(
            "  Linux:            ensure `sudo orcker elevate ports` is applied and the host \
             firewall allows 80/443/1053 from your LAN."
        );
    }
    ExitCode::SUCCESS
}

/// Read the running daemon's `boot_id` (a per-process random id used to detect a
/// completed restart across the pid-preserving re-exec).
async fn fetch_boot_id() -> Result<Option<u64>, ClientError> {
    use orcker_ipc::{Request, Response};
    match transport::exchange(&Request::Status).await? {
        Response::Status { report } => Ok(report.boot_id),
        other => Err(ClientError::Usage(format!(
            "unexpected response to Status: {other:?}"
        ))),
    }
}

/// Send `RestartDaemon`, then poll `Status` (tolerating the transient
/// connection failure while the daemon re-execs) until `boot_id` differs from
/// `before` or a bounded timeout elapses.
async fn restart_and_await_boot_change(before: Option<u64>) -> Result<(), ClientError> {
    use orcker_ipc::Request;
    // The daemon writes `Ok` and flushes *before* it re-execs, so this returns
    // normally; a transient error is tolerated too (the socket may already be
    // tearing down).
    let _ = transport::exchange(&Request::RestartDaemon).await;

    let deadline = std::time::Duration::from_secs(15);
    let step = std::time::Duration::from_millis(200);
    let mut waited = std::time::Duration::ZERO;
    loop {
        tokio::time::sleep(step).await;
        waited += step;
        if let Ok(now) = fetch_boot_id().await {
            if now != before && now.is_some() {
                return Ok(());
            }
        }
        if waited >= deadline {
            return Err(ClientError::Usage(
                "timed out waiting for the daemon to come back up".to_owned(),
            ));
        }
    }
}

/// `orcker route list <site>`: one `ListRoutes` round-trip, narrowed to `site`
/// client-side. The unfiltered form goes through the normal `render` path.
async fn run_route_list(site: &str, json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};
    match transport::exchange(&Request::ListRoutes).await {
        Ok(Response::Routes { rules }) => {
            let r = map::render_routes(&rules, Some(site), json);
            if !r.stdout.is_empty() {
                println!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprintln!("{}", r.stderr);
            }
            ExitCode::from(r.code)
        }
        Ok(other) => {
            let r = map::render(&other, json);
            if !r.stdout.is_empty() {
                println!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprintln!("{}", r.stderr);
            }
            ExitCode::from(r.code)
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            ExitCode::from(69)
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            ExitCode::from(74)
        }
    }
}

/// `orcker domain list [site]`: a local two-request flow. Needs the TLD (via
/// `DaemonInfo`) to render an effectively-default site's `{name}.{tld}` domain,
/// then lists sites and renders a domain-focused view.
async fn run_domain_list(site: Option<&str>, json: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};
    let tld = match transport::exchange(&Request::DaemonInfo).await {
        Ok(Response::Info { tld, .. }) => tld,
        Ok(_) => {
            eprintln!("orcker: unexpected daemon response");
            return ExitCode::from(74);
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(74);
        }
    };

    match transport::exchange(&Request::ListSites).await {
        Ok(Response::Sites { sites }) => {
            let r = map::render_domains(&sites, &tld, site, json);
            if !r.stdout.is_empty() {
                println!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprintln!("{}", r.stderr);
            }
            ExitCode::from(r.code)
        }
        Ok(other) => {
            let r = map::render(&other, json);
            if !r.stdout.is_empty() {
                println!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprintln!("{}", r.stderr);
            }
            ExitCode::from(r.code)
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            ExitCode::from(69)
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            ExitCode::from(74)
        }
    }
}

/// `orcker update --yes`: the self-update apply path.
///
/// Persists the channel when `--edge`/`--stable` is given, checks the channel,
/// and when a newer version is available asks the daemon to download + verify
/// the artifact ([`Request::StageUpdate`]) and then applies it **in-process**
/// (the CLI is a short-lived terminal process: it swaps the bundle it runs from,
/// off its old inode, then exits). The detached-subprocess applier is only for
/// the GUI, which must quit during the swap.
#[allow(clippy::too_many_lines, clippy::fn_params_excessive_bools)]
async fn run_self_update_apply(json: bool, edge: bool, stable: bool, force: bool) -> ExitCode {
    use orcker_ipc::{Request, Response};

    if json {
        eprintln!("orcker: --json is not supported with `update --yes` (apply); use it for the check-only `orcker update`");
        return ExitCode::from(2);
    }

    let channel_override = map::channel_from_flags(edge, stable);

    if channel_override.is_some() {
        let name = if edge { "edge" } else { "stable" };
        match transport::exchange(&Request::SetUpdateChannel {
            channel: channel_override.unwrap_or(orcker_ipc::Channel::Stable),
        })
        .await
        {
            Ok(Response::Ok) => {
                if !json {
                    println!("orcker: update channel set to {name}");
                }
            }
            Ok(Response::Error { message, .. }) => {
                eprintln!("orcker: {message}");
                return ExitCode::from(1);
            }
            Ok(_) => {
                eprintln!("orcker: unexpected response setting update channel");
                return ExitCode::from(74);
            }
            Err(e) if e.is_daemon_down() => {
                eprintln!("orcker: {e}");
                return ExitCode::from(69);
            }
            Err(e) => {
                eprintln!("orcker: {e}");
                return ExitCode::from(74);
            }
        }
    }

    let status = match transport::exchange(&Request::CheckUpdate {
        channel: channel_override,
    })
    .await
    {
        Ok(Response::UpdateStatus {
            current,
            latest_stable,
            available,
            target,
            ahead_of_stable,
            ..
        }) => (current, latest_stable, available, target, ahead_of_stable),
        Ok(Response::Error { message, .. }) => {
            eprintln!("orcker: {message}");
            return ExitCode::from(1);
        }
        Ok(_) => {
            eprintln!("orcker: unexpected response checking for updates");
            return ExitCode::from(74);
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(74);
        }
    };
    let (current, latest_stable, available, target, ahead_of_stable) = status;

    if !available {
        if ahead_of_stable && force {
            println!(
                "orcker: you're on pre-release {current} (ahead of stable {}); automated \
                 downgrade isn't supported yet - reinstall the stable build manually",
                latest_stable.as_deref().unwrap_or("unknown")
            );
        } else if ahead_of_stable {
            println!(
                "orcker: on pre-release {current}, ahead of stable {} - staying put (use --force \
                 to force a downgrade once supported)",
                latest_stable.as_deref().unwrap_or("unknown")
            );
        } else {
            println!("orcker: already up to date ({current})");
        }
        return ExitCode::SUCCESS;
    }

    if !json {
        println!(
            "orcker: downloading and verifying {}…",
            target.as_deref().unwrap_or("the update")
        );
    }
    let (path, kind) = match transport::exchange(&Request::StageUpdate {
        channel: channel_override,
    })
    .await
    {
        Ok(Response::Staged { path, kind, .. }) => (path, kind),
        Ok(Response::Error { message, .. }) => {
            eprintln!("orcker: {message}");
            return ExitCode::from(1);
        }
        Ok(_) => {
            eprintln!("orcker: unexpected response staging the update");
            return ExitCode::from(74);
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(74);
        }
    };

    tokio::task::spawn_blocking(move || apply::run(std::path::Path::new(&path), kind, false, false))
        .await
        .unwrap_or_else(|e| {
            eprintln!("orcker: applier task failed: {e}");
            ExitCode::from(74)
        })
}

/// Install a dev tool as a streamed job, printing its output line by line until
/// the job reaches a terminal state. Mirrors the GUI's streamed install.
async fn stream_install_tool(id: &str, json: bool) -> ExitCode {
    use orcker_ipc::{JobState, Request, Response};
    use std::time::Duration;

    let job_id = match transport::exchange(&Request::InstallToolStreamed {
        tool: id.to_owned(),
    })
    .await
    {
        Ok(Response::JobStarted { job_id }) => job_id,
        Ok(Response::Error { message, .. }) => {
            eprintln!("orcker: {message}");
            return ExitCode::from(1);
        }
        Ok(_) => {
            eprintln!("orcker: unexpected response starting install");
            return ExitCode::from(74);
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(74);
        }
    };

    let mut cursor = 0u64;
    loop {
        match transport::exchange(&Request::JobStatus {
            job_id: job_id.clone(),
            cursor,
        })
        .await
        {
            Ok(Response::JobProgress {
                state,
                log,
                next_cursor,
                error,
                ..
            }) => {
                for line in &log {
                    println!("{line}");
                }
                cursor = next_cursor;
                match state {
                    JobState::Running => tokio::time::sleep(Duration::from_millis(400)).await,
                    JobState::Succeeded => {
                        path_cmd::ensure_installed_after_tool(json);
                        return ExitCode::SUCCESS;
                    }
                    JobState::Failed => {
                        if let Some(e) = error {
                            eprintln!("orcker: {e}");
                        }
                        return ExitCode::from(1);
                    }
                    JobState::Cancelled => {
                        eprintln!("orcker: install cancelled");
                        return ExitCode::from(1);
                    }
                }
            }
            Ok(Response::Error { message, .. }) => {
                eprintln!("orcker: {message}");
                return ExitCode::from(1);
            }
            Ok(_) => {
                eprintln!("orcker: unexpected response polling install");
                return ExitCode::from(74);
            }
            Err(e) if e.is_daemon_down() => {
                eprintln!("orcker: {e}");
                return ExitCode::from(69);
            }
            Err(e) => {
                eprintln!("orcker: {e}");
                return ExitCode::from(74);
            }
        }
    }
}

/// Run a streamed tunnel job (`cloudflared` install or account login), printing
/// progress lines (including the login auth URL) as they arrive. Mirrors
/// [`stream_install_tool`].
async fn stream_tunnel_job(req: orcker_ipc::Request) -> ExitCode {
    use orcker_ipc::{JobState, Request, Response};
    use std::time::Duration;

    let noun = if matches!(req, Request::CloudflaredLogin) {
        "login"
    } else {
        "install"
    };

    let job_id = match transport::exchange(&req).await {
        Ok(Response::JobStarted { job_id }) => job_id,
        Ok(Response::Error { message, .. }) => {
            eprintln!("orcker: {message}");
            return ExitCode::from(1);
        }
        Ok(_) => {
            eprintln!("orcker: unexpected response starting {noun}");
            return ExitCode::from(74);
        }
        Err(e) if e.is_daemon_down() => {
            eprintln!("orcker: {e}");
            return ExitCode::from(69);
        }
        Err(e) => {
            eprintln!("orcker: {e}");
            return ExitCode::from(74);
        }
    };

    let mut cursor = 0u64;
    loop {
        match transport::exchange(&Request::JobStatus {
            job_id: job_id.clone(),
            cursor,
        })
        .await
        {
            Ok(Response::JobProgress {
                state,
                log,
                next_cursor,
                error,
                ..
            }) => {
                for line in &log {
                    println!("{line}");
                }
                cursor = next_cursor;
                match state {
                    JobState::Running => tokio::time::sleep(Duration::from_millis(400)).await,
                    JobState::Succeeded => return ExitCode::SUCCESS,
                    JobState::Failed => {
                        if let Some(e) = error {
                            eprintln!("orcker: {e}");
                        }
                        return ExitCode::from(1);
                    }
                    JobState::Cancelled => {
                        eprintln!("orcker: {noun} cancelled");
                        return ExitCode::from(1);
                    }
                }
            }
            Ok(Response::Error { message, .. }) => {
                eprintln!("orcker: {message}");
                return ExitCode::from(1);
            }
            Ok(_) => {
                eprintln!("orcker: unexpected response polling {noun}");
                return ExitCode::from(74);
            }
            Err(e) if e.is_daemon_down() => {
                eprintln!("orcker: {e}");
                return ExitCode::from(69);
            }
            Err(e) => {
                eprintln!("orcker: {e}");
                return ExitCode::from(74);
            }
        }
    }
}

/// Best-effort: rewrite an `Unpark` request's path to its canonical form so a
/// relative or symlinked path the user typed matches the canonical string the
/// daemon stored when the directory was parked. The daemon matches `unpark`
/// *exactly* (it deliberately does not canonicalise - so a directory deleted
/// from disk is still removable by its exact stored path); doing it here, at the
/// I/O boundary, keeps `map::to_request` pure. A path that can't be canonicalised
/// (e.g. already deleted) is left exactly as typed.
fn canonicalize_unpark(req: orcker_ipc::Request) -> orcker_ipc::Request {
    if let orcker_ipc::Request::Unpark { path } = &req {
        if let Ok(canon) = std::fs::canonicalize(path) {
            return orcker_ipc::Request::Unpark {
                path: canon.to_string_lossy().into_owned(),
            };
        }
    }
    req
}

/// Absolutise and canonicalise a `Park` request against the user's current
/// directory before it reaches the daemon. The daemon's cwd differs from the
/// user's shell, so a relative path like `.` would otherwise resolve there.
fn canonicalize_park_path(req: orcker_ipc::Request) -> Result<orcker_ipc::Request, ClientError> {
    use orcker_ipc::Request;
    match req {
        Request::Park { path } => {
            let abs = absolutise(&path)?;
            let canon = std::fs::canonicalize(&abs).map_err(|e| {
                ClientError::Usage(format!("cannot resolve {}: {e}", path.display()))
            })?;
            Ok(Request::Park { path: canon })
        }
        other => Ok(other),
    }
}

/// Make a (possibly relative) path absolute by joining it onto the current directory.
/// Does not require the path to exist (used for a backup destination).
fn absolutise(path: &std::path::Path) -> Result<std::path::PathBuf, ClientError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir()
        .map_err(|e| ClientError::Usage(format!("cannot resolve current directory: {e}")))?;
    Ok(cwd.join(path))
}

/// Whether a single positional argument to `orcker link` should be treated as
/// a directory rather than a bare site name: contains a path separator, or
/// is `.`/`..`. Bare words (`orcker link project`) are always names, even if a
/// same-named subdirectory happens to exist.
fn looks_like_path(s: &str) -> bool {
    s == "." || s == ".." || s.contains('/') || s.contains(std::path::MAIN_SEPARATOR)
}

/// Resolve `orcker link`'s optional name/path into a concrete `Request::Link`.
/// Cwd-dependent (reads `std::env::current_dir()` when path is omitted) and
/// therefore not part of the pure `map::to_request` pipeline - kept here at
/// the I/O boundary, mirroring `absolutise`/`canonicalize_db_paths`.
///
/// Public so `tests/cli_e2e.rs` can drive the same CLI-side resolution this
/// crate's `run()` uses, then exchange the result with a real daemon.
/// Builds a [`orcker_ipc::Request::LinkProject`] from `orcker link`'s
/// arguments: an omitted `path` means the current directory, and an omitted
/// `--name` leaves the daemon to derive one from the directory or the
/// project's existing `orcker.yml`.
///
/// The path is resolved here (the CLI is the only side that knows the user's
/// working directory); the daemon canonicalises it again before use.
///
/// # Errors
///
/// [`ClientError::Usage`] when the current directory cannot be read, the path
/// cannot be made absolute, or `--name` is not a valid site name.
pub fn resolve_link_project(
    path: Option<&std::path::Path>,
    name: Option<&str>,
    port: Option<u16>,
) -> Result<orcker_ipc::Request, ClientError> {
    let resolved_path = match path {
        Some(p) => absolutise(p)?,
        None => std::env::current_dir()
            .map_err(|e| ClientError::Usage(format!("cannot resolve current directory: {e}")))?,
    };
    if let Some(name) = name {
        map::validate_name(name)?;
    }
    Ok(orcker_ipc::Request::LinkProject {
        path: resolved_path,
        name: name.map(str::to_owned),
        port,
    })
}

/// Builds a [`orcker_ipc::Request::Link`] for the inherited on-disk site link,
/// inferring whichever of name/path was omitted.
///
/// Not reachable from `orcker link` any more: that command now links container
/// projects (see [`resolve_link_project`]). Kept because the daemon still
/// serves `Request::Link` for the GUI, and the CLI's end-to-end tests drive
/// that path through here.
///
/// # Errors
///
/// [`ClientError::Usage`] when the current directory cannot be read, the path
/// cannot be made absolute, or no valid site name can be derived.
pub fn resolve_link(
    name_or_path: Option<&str>,
    path: Option<&std::path::Path>,
) -> Result<orcker_ipc::Request, ClientError> {
    let explicit_path = path.or_else(|| {
        name_or_path
            .filter(|s| looks_like_path(s))
            .map(std::path::Path::new)
    });
    let explicit_name = if path.is_some() {
        name_or_path
    } else {
        name_or_path.filter(|s| !looks_like_path(s))
    };

    let resolved_path = match explicit_path {
        Some(p) => absolutise(p)?,
        None => std::env::current_dir()
            .map_err(|e| ClientError::Usage(format!("cannot resolve current directory: {e}")))?,
    };

    let resolved_name = if let Some(n) = explicit_name {
        map::validate_name(n)?;
        n.to_owned()
    } else {
        let normalized = normalize_lexically(&resolved_path);
        let folder = normalized
            .file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        orcker_core::slugify_site_name(&folder).ok_or_else(|| {
            ClientError::Usage(format!(
                "cannot derive a site name from '{}'; run `orcker link <name> {}` to set one explicitly",
                resolved_path.display(),
                resolved_path.display(),
            ))
        })?
    };

    Ok(orcker_ipc::Request::Link {
        name: resolved_name,
        path: resolved_path,
    })
}

/// Lexically resolve `.`/`..` components out of `path` without touching the
/// filesystem (no `canonicalize` - `resolve_link` deliberately doesn't
/// require the target to exist), so `Path::file_name()` sees the folder
/// actually being linked. `Path::file_name()` already normalises a trailing
/// `.` away on its own, but a trailing `..` survives as-is (it can't tell
/// what it cancels without looking further back), so e.g. `orcker link ..`
/// would otherwise fail to derive the parent folder's name even though it's
/// well-defined.
fn normalize_lexically(path: &std::path::Path) -> std::path::PathBuf {
    use std::path::{Component, PathBuf};
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                } else {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// A synthetic `daemon_down` FAIL diagnosis, used when `orcker doctor` can't reach
/// the daemon. Routed through `map::render` so it honours `--json` and exits 1.
fn daemon_down_response() -> orcker_ipc::Response {
    orcker_ipc::Response::Diagnoses {
        items: vec![orcker_ipc::Diagnosis {
            code: orcker_ipc::DiagnosisCode::DaemonDown,
            severity: orcker_ipc::Severity::Fail,
            title: "Daemon not running".to_owned(),
            detail: "Could not reach the orcker daemon over its IPC socket.".to_owned(),
            remedy: Some("start the daemon: orckerd".to_owned()),
        }],
    }
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
    use orcker_ipc::Request;
    use std::path::{Path, PathBuf};

    // ─── canonicalize_unpark ────────────────────────────────────────

    #[test]
    fn canonicalize_unpark_resolves_existing_path() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("sub");
        std::fs::create_dir(&nested).unwrap();
        let req = Request::Unpark {
            path: nested.to_string_lossy().into_owned(),
        };
        let out = canonicalize_unpark(req);
        let Request::Unpark { path } = out else {
            panic!("expected Unpark");
        };
        let canon = std::fs::canonicalize(&nested).unwrap();
        assert_eq!(path, canon.to_string_lossy());
    }

    #[test]
    fn canonicalize_unpark_leaves_missing_path_untouched() {
        let raw = "/no/such/dir/that/exists/anywhere-xyz";
        let req = Request::Unpark {
            path: raw.to_owned(),
        };
        match canonicalize_unpark(req) {
            Request::Unpark { path } => assert_eq!(path, raw),
            _ => panic!("expected Unpark"),
        }
    }

    #[test]
    fn canonicalize_unpark_passes_through_other_requests() {
        assert_eq!(canonicalize_unpark(Request::Ping), Request::Ping);
        let listed = canonicalize_unpark(Request::ListSites);
        assert_eq!(listed, Request::ListSites);
    }

    // ─── canonicalize_park_path ─────────────────────────────────────

    #[test]
    fn canonicalize_park_path_resolves_dot_against_cwd() {
        let out = canonicalize_park_path(Request::Park {
            path: PathBuf::from("."),
        })
        .unwrap();

        let Request::Park { path } = out else {
            panic!("expected Park");
        };
        assert_eq!(path, std::fs::canonicalize(".").unwrap());
    }

    #[test]
    fn canonicalize_park_path_canonicalises_absolute_path() {
        let tmp = tempfile::tempdir().unwrap();
        let out = canonicalize_park_path(Request::Park {
            path: tmp.path().to_path_buf(),
        })
        .unwrap();
        let Request::Park { path } = out else {
            panic!("expected Park");
        };
        assert_eq!(path, std::fs::canonicalize(tmp.path()).unwrap());
    }

    #[test]
    fn canonicalize_park_path_missing_dir_is_usage_error() {
        let err = canonicalize_park_path(Request::Park {
            path: PathBuf::from("/no/such/park/root-xyz"),
        })
        .unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)));
    }

    #[test]
    fn canonicalize_park_path_passes_through_other_requests() {
        assert_eq!(
            canonicalize_park_path(Request::Ping).unwrap(),
            Request::Ping
        );
    }

    // ─── canonicalize_db_paths ──────────────────────────────────────

    // ─── absolutise ─────────────────────────────────────────────────

    #[test]
    fn absolutise_returns_absolute_path_unchanged() {
        let abs = Path::new("/etc/hosts");
        assert_eq!(absolutise(abs).unwrap(), abs.to_path_buf());
    }

    #[test]
    fn absolutise_joins_relative_onto_cwd() {
        let rel = Path::new("some/where.sql");
        let out = absolutise(rel).unwrap();
        assert!(out.is_absolute());
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(out, cwd.join(rel));
    }

    // ─── looks_like_path ────────────────────────────────────────────

    #[test]
    fn looks_like_path_classifies_bare_words_vs_paths() {
        let cases: &[(&str, bool)] = &[
            ("project", false),
            ("my-app", false),
            (".", true),
            ("..", true),
            ("./project", true),
            ("~/sites/example", true),
            ("/abs/path", true),
        ];
        for (input, expected) in cases {
            assert_eq!(looks_like_path(input), *expected, "input {input:?}");
        }
    }

    // ─── resolve_link ───────────────────────────────────────────────

    #[test]
    fn resolve_link_no_args_uses_cwd_and_derives_name() {
        let req = resolve_link(None, None).unwrap();
        let Request::Link { name, path } = req else {
            panic!("expected Link");
        };
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(path, cwd);
        let folder = cwd.file_name().and_then(|s| s.to_str()).unwrap_or("");
        assert_eq!(Some(name), orcker_core::slugify_site_name(folder));
    }

    #[test]
    fn resolve_link_bare_word_uses_cwd_as_path() {
        let req = resolve_link(Some("myapp"), None).unwrap();
        let Request::Link { name, path } = req else {
            panic!("expected Link");
        };
        assert_eq!(name, "myapp");
        assert_eq!(path, std::env::current_dir().unwrap());
    }

    #[test]
    fn resolve_link_single_path_arg_derives_name() {
        let req = resolve_link(Some("../my-app"), None).unwrap();
        let Request::Link { name, path } = req else {
            panic!("expected Link");
        };
        assert_eq!(name, "my-app");
        assert!(path.is_absolute());
        assert!(path.ends_with("../my-app"));
    }

    #[test]
    fn resolve_link_explicit_name_and_path_unchanged() {
        let req = resolve_link(Some("blog"), Some(Path::new("rel/blog"))).unwrap();
        let Request::Link { name, path } = req else {
            panic!("expected Link");
        };
        assert_eq!(name, "blog");
        assert!(path.is_absolute());
        assert!(path.ends_with("rel/blog"));
    }

    /// "weird/???" looks like a path (contains '/'), so the name is derived
    /// from its final component "???", which slugifies to `None`.
    #[test]
    fn resolve_link_undecipherable_name_errors() {
        let err = resolve_link(Some("weird/???"), None).unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)), "got: {err:?}");
    }

    #[test]
    fn resolve_link_rejects_invalid_explicit_name() {
        let err = resolve_link(Some("bad name"), Some(Path::new("/x"))).unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)), "got: {err:?}");
    }

    /// "" doesn't look like a path, so it's routed to explicit-name
    /// validation, which rejects an empty name.
    #[test]
    fn resolve_link_empty_string_name_errors() {
        let err = resolve_link(Some(""), None).unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)), "got: {err:?}");
    }

    /// "/" normalises to the filesystem root, which has no `Normal`
    /// component left to derive a name from.
    #[test]
    fn resolve_link_root_path_has_no_file_name_errors() {
        let err = resolve_link(Some("/"), None).unwrap_err();
        assert!(matches!(err, ClientError::Usage(_)), "got: {err:?}");
    }

    #[test]
    fn resolve_link_trailing_curdir_derives_name() {
        let req = resolve_link(Some("some/dir/."), None).unwrap();
        let Request::Link { name, .. } = req else {
            panic!("expected Link");
        };
        assert_eq!(name, "dir");
    }

    #[test]
    fn resolve_link_trailing_parentdir_derives_name() {
        let req = resolve_link(Some("some/parent/child/.."), None).unwrap();
        let Request::Link { name, .. } = req else {
            panic!("expected Link");
        };
        assert_eq!(name, "parent");
    }

    // ─── normalize_lexically ────────────────────────────────────────

    #[test]
    fn normalize_lexically_cases() {
        let cases: &[(&str, &str)] = &[
            ("/home/user/myapp/.", "/home/user/myapp"),
            ("/home/user/myapp/..", "/home/user"),
            ("/a/b/../c", "/a/c"),
            ("/../../foo", "/../../foo"),
            ("/", "/"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                normalize_lexically(Path::new(input)),
                Path::new(expected),
                "input {input:?}"
            );
        }
    }

    // ─── daemon_down_response ───────────────────────────────────────

    #[test]
    fn daemon_down_response_is_single_fail_diagnosis() {
        match daemon_down_response() {
            orcker_ipc::Response::Diagnoses { items } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].code, orcker_ipc::DiagnosisCode::DaemonDown);
                assert_eq!(items[0].severity, orcker_ipc::Severity::Fail);
                assert!(items[0].remedy.is_some());
            }
            other => panic!("expected Diagnoses, got {other:?}"),
        }
    }

    // ─── PATH helpers ───────────────────────────────────────────────
}
