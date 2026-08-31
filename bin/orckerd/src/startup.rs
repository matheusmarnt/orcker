//! Daemon startup orchestration.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use interprocess::local_socket::tokio::Listener as IpcListener;
use interprocess::local_socket::ListenerOptions;
use tokio::sync::{Mutex, RwLock};

use orcker_core::{PhpVersion, Site, SiteRouter};
use orcker_platform::{
    ActivePaths, ActivePortBinder, LanIpProvider, Paths, PlatformDirs, PortBinder,
};
use orcker_tls::{CertAuthority, Validity};

use crate::args::ServeArgs;
use crate::cert_store::DaemonCertStore;
use crate::detect_cache::DetectCache;
use crate::error::DaemonError;
use crate::single_instance::InstanceLock;
use crate::state::DaemonState;

/// Loopback IP the embedded DNS responder binds on. The port comes from
/// [`orcker_config::Config::dns_port`] (default [`orcker_config::DEFAULT_DNS_PORT`],
/// not the mDNS-contended `5353`).
///
/// A **fixed** port (rather than ephemeral) is required so the resolver config
/// written by `orcker elevate resolver` - which hard-codes `DNS=127.0.0.1:<port>` -
/// stays valid across daemon restarts. `dns_port = 0` still means ephemeral
/// (dev/tests only); the kernel-assigned port is read back via
/// [`orcker_dns::Bound::local_addr`] and stored on [`Daemon::dns_addr`].
pub const DNS_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// The address to advertise for the DNS responder (reported by `DaemonInfo` and
/// baked into `/etc/resolver/<tld>` by `orcker elevate resolver`). In LAN mode the
/// responder binds the wildcard `0.0.0.0`, but the resolver config and the
/// host's own queries must target a concrete loopback address - so a wildcard
/// bound address is republished as `127.0.0.1` with the actual bound port.
/// Non-wildcard addresses pass through unchanged.
fn normalize_dns_advertise_addr(addr: SocketAddr) -> SocketAddr {
    match addr.ip() {
        IpAddr::V4(v4) if v4.is_unspecified() => {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), addr.port())
        }
        _ => addr,
    }
}

/// Everything `run()` needs to start the daemon's tasks.
pub struct Daemon {
    /// Shared runtime state: authoritative config + live router (the proxy and
    /// IPC mutation path both work through this).
    pub state: Arc<DaemonState>,
    /// Immutable TLD snapshot for the DNS responder. Taken at startup because
    /// the TLD never changes via an IPC mutation (no `SetTld`), so the DNS task
    /// must not reach into the config mutex.
    pub dns_tld: orcker_core::Tld,
    /// Where the config file was loaded from.
    pub config_path: PathBuf,
    /// Resolved per-user directories.
    pub dirs: PlatformDirs,
    /// Held until `run()` returns - releases on drop.
    pub lock: InstanceLock,
    /// TLS cert store for SNI lookups.
    pub cert_store: Arc<DaemonCertStore>,
    /// Bound HTTP listener. `None` when the daemon could bind neither the
    /// desired nor the fallback web ports - it then runs degraded (no proxy).
    pub http_listener: Option<tokio::net::TcpListener>,
    /// Bound HTTPS listener. `None` in the same degraded case as `http_listener`.
    pub https_listener: Option<tokio::net::TcpListener>,
    /// IPC listener (Unix socket on Unix, named pipe on Windows).
    pub ipc_listener: IpcListener,
    /// Bound DNS sockets (UDP+TCP), owned by the daemon and consumed when the
    /// DNS task is spawned. `None` when the DNS port couldn't bind - the daemon
    /// then runs degraded (no name resolution) rather than aborting, mirroring
    /// the `http_listener`/`https_listener` web degrade.
    pub dns_bound: Option<orcker_dns::Bound>,
    /// Actual DNS bind address, read back from the kernel after the bind. When
    /// the DNS port couldn't bind this stays the *wanted* address (so the
    /// resolver-install probe still has a target to report against).
    pub dns_addr: SocketAddr,
    /// The addresses the DNS responder answers `.test` with. Loopback-only
    /// unless LAN mode is on, in which case it split-horizons (see
    /// [`orcker_dns::AnswerAddrs`]). Consumed when the DNS task is spawned.
    pub dns_answer: orcker_dns::AnswerAddrs,
    /// The host's discovered LAN IPv4, when LAN mode is on and discovery
    /// succeeded. `None` otherwise (LAN off, or discovery failed - fail-closed).
    pub lan_ip: Option<Ipv4Addr>,
    /// Bound mail-capture SMTP listener, when capture is enabled and the port was
    /// free. `None` = disabled, or the bind failed (non-fatal). Consumed when the
    /// mail task is spawned.
    pub mail_listener: Option<tokio::net::TcpListener>,
}

/// Top-level startup: resolve platform dirs, then run the shared
/// `bring_up_with_dirs` pipeline.
pub async fn bring_up(args: &ServeArgs) -> Result<Daemon, DaemonError> {
    let dirs = ActivePaths::new().resolve()?;
    let cfg_path = args
        .config
        .clone()
        .unwrap_or_else(|| dirs.config.join("orcker.toml"));
    let config = load_or_default_config(&cfg_path)?;
    bring_up_with_dirs(dirs, config, cfg_path).await
}

/// Integration-test entry point.
///
/// Skips `ActivePaths::resolve` so the test can hand the daemon a
/// `tempfile`-rooted `PlatformDirs`. The body is identical to `bring_up`
/// from step 2 onwards.
#[doc(hidden)]
#[allow(clippy::too_many_lines)]
pub async fn bring_up_with_dirs(
    dirs: PlatformDirs,
    config: orcker_config::Config,
    config_path: PathBuf,
) -> Result<Daemon, DaemonError> {
    let lock = InstanceLock::acquire(&dirs)?;

    let ca = load_or_generate_ca(&dirs)?;
    let ca_path = dirs.data.join("ca.cert.pem");
    let ca_fingerprint = orcker_platform::CaFingerprint::new(ca.fingerprint_sha256());

    let cert_store = Arc::new(DaemonCertStore::new(ca, dirs.data.join("leaves")));

    let detect_cache = Arc::new(DetectCache::new());
    let dns_tld = config.tld.clone();
    let (router, wordpress_sites, laravel_sites) = build_router(&config, &dirs, &detect_cache)?;
    if router.is_empty() {
        tracing::info!("no sites configured - every request will 404 until a site is added");
    }
    let router = Arc::new(RwLock::new(router));
    let wordpress_sites = Arc::new(RwLock::new(wordpress_sites));
    let laravel_sites = Arc::new(RwLock::new(laravel_sites));

    let cfg_http = config.ports.http;
    let cfg_https = config.ports.https;
    let fb_http = config.ports.fallback_http;
    let fb_https = config.ports.fallback_https;
    let binder = ActivePortBinder::new();
    let (http_listener, tls_listener, bound_http, bound_https, web_unbound) = match binder
        .bind_pair(
            config.lan_enabled,
            (cfg_http, cfg_https),
            (fb_http, fb_https),
        ) {
        Ok(pair) => {
            let bound_http = pair.http.port().map_err(|source| DaemonError::Io {
                path: PathBuf::from("<http listener>"),
                source,
            })?;
            let bound_https = pair.https.port().map_err(|source| DaemonError::Io {
                path: PathBuf::from("<https listener>"),
                source,
            })?;
            if (bound_http, bound_https) != (cfg_http, cfg_https) {
                tracing::warn!(
                        http = bound_http,
                        https = bound_https,
                        wanted_http = cfg_http,
                        wanted_https = cfg_https,
                        "bound rootless fallback ports; .test URLs will need explicit ports until setcap or a port-redirector is configured"
                    );
            }
            let http_listener = into_tokio_listener(pair.http.listener)?;
            let tls_listener = into_tokio_listener(pair.https.listener)?;
            (
                Some(http_listener),
                Some(tls_listener),
                bound_http,
                bound_https,
                None,
            )
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                fallback_http = fb_http,
                fallback_https = fb_https,
                "could not bind web ports; serving disabled - free the ports or change the fallback ports in Settings, then restart"
            );
            (
                None,
                None,
                0,
                0,
                Some(orcker_ipc::UnboundWeb {
                    http: fb_http,
                    https: fb_https,
                }),
            )
        }
    };

    let tunnel_manager = Arc::new(Mutex::new(crate::tunnel::new_manager()));

    let ipc_listener = build_ipc_listener(&dirs)?;

    // LAN mode: discover the host's routable IPv4 (fail-closed - a discovery
    // error leaves `lan_ip = None`, so DNS answers fall back to loopback while
    // the daemon still serves locally), bind the responder on `0.0.0.0` so other
    // devices can reach it, and answer `.test` with split-horizon.
    let lan_ip: Option<Ipv4Addr> = if config.lan_enabled {
        match orcker_platform::ActiveLanIpProvider::new().lan_ipv4() {
            Ok(ip) => Some(ip),
            Err(e) => {
                tracing::warn!(error = %e, "LAN IP discovery failed; .test answers fall back to loopback");
                None
            }
        }
    } else {
        None
    };
    let dns_answer = if config.lan_enabled {
        orcker_dns::AnswerAddrs::lan(lan_ip)
    } else {
        orcker_dns::AnswerAddrs::loopback()
    };

    let cfg_dns = config.dns_port;
    let dns_ip = if config.lan_enabled {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    } else {
        DNS_IP
    };
    let dns_want = SocketAddr::new(dns_ip, cfg_dns);
    let (dns_bound, dns_addr, dns_unbound) = match orcker_dns::Bound::bind(dns_want).await {
        Ok(bound) => {
            let addr = normalize_dns_advertise_addr(bound.local_addr());
            tracing::info!(dns = %addr, "DNS responder bound");
            (Some(bound), addr, None)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                dns = %dns_want,
                "could not bind DNS responder; name resolution disabled - free dns_port or change it in Settings, then restart"
            );
            (None, dns_want, Some(cfg_dns))
        }
    };

    let mail_enabled = config.mail.enabled;
    let mail_port = config.mail.port;
    let mail_store = Arc::new(
        orcker_mail::Store::open(dirs.data.join("mail")).map_err(|e| DaemonError::Io {
            path: dirs.data.join("mail"),
            source: std::io::Error::other(e.to_string()),
        })?,
    );
    let mail_listener = if mail_enabled {
        match orcker_mail::bind(mail_port).await {
            Ok(listener) => {
                tracing::info!(port = mail_port, "mail capture SMTP server bound");
                Some(listener)
            }
            Err(e) => {
                tracing::warn!(port = mail_port, error = %e, "mail capture disabled: could not bind port");
                None
            }
        }
    } else {
        None
    };
    let mail_listening = mail_listener.is_some();
    let symlink_protection_on = config.symlink_protection;

    let state = Arc::new(DaemonState {
        config: Mutex::new(config),
        router,
        dirs: dirs.clone(),
        config_path: config_path.clone(),
        dns_addr,
        ca_path,
        ca_fingerprint,
        orcker_update: tokio::sync::RwLock::new(Vec::new()),
        update_snapshot: tokio::sync::RwLock::new(crate::self_update::load_snapshot(&dirs)),
        tunnel_manager,
        cloudflared_resolution: tokio::sync::RwLock::new(None),
        mail_store,
        mail: crate::state::MailRuntime {
            listening: mail_listening,
        },
        http: orcker_ipc::PortStatus {
            requested: cfg_http,
            bound: bound_http,
            fell_back: bound_http != cfg_http,
        },
        https: orcker_ipc::PortStatus {
            requested: cfg_https,
            bound: bound_https,
            fell_back: bound_https != cfg_https,
        },
        redirect_https_port: Arc::new(std::sync::atomic::AtomicU16::new(bound_https)),
        symlink_protection: Arc::new(std::sync::atomic::AtomicBool::new(symlink_protection_on)),
        web_unbound,
        dns_unbound,
        boot_id: rand_boot_id(),
        started_at: std::time::Instant::now(),
        shutdown_tx: tokio::sync::watch::channel(false).0,
        restart_requested: std::sync::atomic::AtomicBool::new(false),
        detect_cache,
        engine_status: Arc::new(crate::engine_status::EngineStatusCache::new()),
        watch_dirty: tokio::sync::Notify::new(),
        shim_reconcile: tokio::sync::Mutex::new(()),
        tool_mutate: tokio::sync::Mutex::new(()),
        tunnel_mutate: tokio::sync::Mutex::new(()),
        php_mutate: tokio::sync::Mutex::new(()),
        php_settings_mutate: tokio::sync::Mutex::new(()),
        jobs: crate::jobs::JobRegistry::default(),
        reserved_names: tokio::sync::Mutex::new(std::collections::HashSet::new()),
        wordpress_sites,
        laravel_sites,
        lan_ip,
        lan_setup_bound: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        remote_setup_code: Mutex::new(None),
        lan_setup_script_sha256: Mutex::new(None),
    });

    warm_engine_status(&state);

    Ok(Daemon {
        state,
        dns_tld,
        config_path,
        dirs,
        lock,
        cert_store,
        http_listener,
        https_listener: tls_listener,
        ipc_listener,
        dns_bound,
        dns_addr,
        dns_answer,
        lan_ip,
        mail_listener,
    })
}

/// Probe Docker once in the background so the first `orcker status` after boot
/// answers from the cache.
///
/// Deliberately detached: a stopped engine costs a connect timeout, and the
/// daemon must be serving IPC before that resolves. A failure is not an error -
/// it becomes a problem entry in the snapshot the next caller reads.
fn warm_engine_status(state: &Arc<DaemonState>) {
    let cache = Arc::clone(&state.engine_status);
    tokio::spawn(async move {
        let status = cache.get().await;
        tracing::info!(
            reachable = status.reachable,
            engine_version = status.engine_version.as_deref().unwrap_or("none"),
            problems = status.problems.len(),
            "docker environment probed"
        );
    });
}

/// Site name → `is_wordpress`, refreshed on every router rebuild - see
/// [`build_routing`] and `DaemonState.wordpress_sites`.
pub(crate) type WordpressSites = std::collections::HashMap<String, bool>;

/// Cache of site name -> is-Laravel, built alongside [`WordpressSites`].
pub(crate) type LaravelSites = std::collections::HashMap<String, bool>;

/// Build a fresh routing table from the config: scan every parked root for
/// child-directory sites, then add the explicitly linked sites (linked wins on
/// name collision). Shared by startup and the IPC mutation path so both
/// produce identical routing. Also returns the `WordPress`-detection cache
/// (site name → `is_wordpress`) that populates `DaemonState.wordpress_sites` -
/// see [`build_routing`].
pub(crate) fn build_router(
    cfg: &orcker_config::Config,
    dirs: &PlatformDirs,
    detect_cache: &DetectCache,
) -> Result<(SiteRouter, WordpressSites, LaravelSites), DaemonError> {
    let (router, wordpress_sites, laravel_sites, _watch_roots) =
        build_routing(cfg, dirs, detect_cache)?;
    Ok((router, wordpress_sites, laravel_sites))
}

/// Like [`build_router`], but also returns the project roots the filesystem
/// watcher should keep watching: parked sites whose web root could **not** be
/// resolved yet (no framework/web-dir detected, no manual override). Resolved
/// sites are deliberately *not* watched - "don't watch what we already know".
///
/// The `WordPress`-detection map is computed here, once per rebuild, over the
/// finished router's sites - not on the `ListSites` poll path - so a marker
/// file is only ever stat'd on a mutation or a filesystem-watcher tick, no
/// matter how often the GUI polls. See `crate::wordpress_detect`.
pub(crate) fn build_routing(
    cfg: &orcker_config::Config,
    dirs: &PlatformDirs,
    detect_cache: &DetectCache,
) -> Result<(SiteRouter, WordpressSites, LaravelSites, Vec<PathBuf>), DaemonError> {
    let (sites, watch_roots) = scan_sites(cfg, cfg.php.default, dirs, detect_cache)?;
    let router = crate::site_domains::build(cfg, sites);
    let wordpress_sites = router
        .iter()
        .map(|site| {
            (
                site.name().to_owned(),
                crate::wordpress_detect::is_wordpress(&site.served_root()),
            )
        })
        .collect();
    let laravel_sites = router
        .iter()
        .map(|site| {
            (
                site.name().to_owned(),
                crate::laravel_detect::is_laravel(site.document_root()),
            )
        })
        .collect();
    Ok((router, wordpress_sites, laravel_sites, watch_roots))
}

// ──────────────────────────────────────────────────────────────────────

fn load_or_default_config(
    cfg_path: &std::path::Path,
) -> Result<orcker_config::Config, DaemonError> {
    match orcker_config::Config::load(cfg_path) {
        Ok(c) => Ok(c),
        Err(orcker_config::ConfigError::Io { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            tracing::info!(
                config = %cfg_path.display(),
                "config file not found - using defaults for first-run boot"
            );
            Ok(orcker_config::Config::default())
        }
        Err(e) => {
            tracing::error!(
                config = %cfg_path.display(),
                error = %e,
                "invalid config file - refusing to start"
            );
            Err(DaemonError::from(e))
        }
    }
}

fn load_or_generate_ca(dirs: &PlatformDirs) -> Result<CertAuthority, DaemonError> {
    let ca_pem_path = dirs.data.join("ca.cert.pem");
    let ca_key_path = dirs.data.join("ca.key.pem");
    if ca_pem_path.exists() && ca_key_path.exists() {
        crate::secure_fs::restrict_writes_to_owner(&ca_pem_path).map_err(|source| {
            DaemonError::Io {
                path: ca_pem_path.clone(),
                source,
            }
        })?;
        crate::secure_fs::restrict_to_owner(&ca_key_path).map_err(|source| DaemonError::Io {
            path: ca_key_path.clone(),
            source,
        })?;
        let cert_pem = std::fs::read_to_string(&ca_pem_path).map_err(|source| DaemonError::Io {
            path: ca_pem_path.clone(),
            source,
        })?;
        let key_pem = std::fs::read_to_string(&ca_key_path).map_err(|source| DaemonError::Io {
            path: ca_key_path.clone(),
            source,
        })?;
        Ok(CertAuthority::from_pem(&cert_pem, &key_pem)?)
    } else {
        let validity = ca_validity()?;
        let ca = CertAuthority::generate(orcker_core::CA_COMMON_NAME, validity)?;
        std::fs::create_dir_all(&dirs.data).map_err(|source| DaemonError::Io {
            path: dirs.data.clone(),
            source,
        })?;
        std::fs::write(&ca_pem_path, ca.cert_pem()).map_err(|source| DaemonError::Io {
            path: ca_pem_path.clone(),
            source,
        })?;
        std::fs::write(&ca_key_path, ca.key_pem()).map_err(|source| DaemonError::Io {
            path: ca_key_path.clone(),
            source,
        })?;
        crate::secure_fs::restrict_to_owner(&ca_key_path).map_err(|source| DaemonError::Io {
            path: ca_key_path.clone(),
            source,
        })?;
        crate::secure_fs::restrict_writes_to_owner(&ca_pem_path).map_err(|source| {
            DaemonError::Io {
                path: ca_pem_path.clone(),
                source,
            }
        })?;
        tracing::warn!(
            ca_pem = %ca_pem_path.display(),
            "generated new CA; trust-store install is deferred to a separate `orckerd install` (not in MVP)"
        );
        Ok(ca)
    }
}

fn ca_validity() -> Result<Validity, DaemonError> {
    let now = time::OffsetDateTime::now_utc();
    Ok(Validity::new(
        now - time::Duration::days(1),
        now + time::Duration::days(10 * 365),
    )?)
}

pub(crate) fn scan_sites(
    cfg: &orcker_config::Config,
    default_php: PhpVersion,
    _dirs: &PlatformDirs,
    detect_cache: &DetectCache,
) -> Result<(Vec<Site>, Vec<PathBuf>), DaemonError> {
    let mut parked: Vec<Site> = Vec::new();
    let mut watch_roots: Vec<PathBuf> = Vec::new();
    let linked_names: std::collections::HashSet<&str> =
        cfg.linked.iter().map(orcker_core::Site::name).collect();

    for parked_root in &cfg.parked.paths {
        let entries = match std::fs::read_dir(parked_root) {
            Ok(it) => it,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!(path = %parked_root, "parked path does not exist; skipping");
                continue;
            }
            Err(source) => {
                return Err(DaemonError::Io {
                    path: PathBuf::from(parked_root),
                    source,
                });
            }
        };
        for entry in entries.flatten() {
            let Some(site_name) = parked_site_name(&entry, &linked_names) else {
                continue;
            };
            let doc_root = entry.path();
            if let Some(site) = build_parked_site(
                &site_name,
                &doc_root,
                default_php,
                cfg,
                detect_cache,
                &mut watch_roots,
            ) {
                parked.push(site);
            }
        }
    }

    parked.extend(cfg.linked.iter().cloned());
    Ok((parked, watch_roots))
}

/// Filter one parked-directory entry to its site name, or `None` to skip it
/// (non-UTF-8, hidden, not a directory, no valid name after normalising, or
/// shadowed by a linked site - linked wins on a name collision). The name is run
/// through [`orcker_core::normalize_site_name`]: an already-valid name is kept, and
/// one the validator would reject (`_`, `.`, ...) is slugified rather than
/// dropped.
fn parked_site_name(
    entry: &std::fs::DirEntry,
    linked_names: &std::collections::HashSet<&str>,
) -> Option<String> {
    let file_name = entry.file_name();
    let Some(name) = file_name.to_str() else {
        tracing::debug!(
            path = %entry.path().display(),
            "skipping non-UTF-8 directory name"
        );
        return None;
    };
    if name.starts_with('.') {
        return None;
    }
    if !entry.metadata().ok()?.is_dir() {
        return None;
    }
    let Some(site_name) = orcker_core::normalize_site_name(name) else {
        tracing::debug!(name = %name, "skipping parked dir with no valid site name");
        return None;
    };
    if linked_names.contains(site_name.as_str()) {
        return None;
    }
    Some(site_name)
}

/// Build a parked [`Site`] for `doc_root`, re-applying any persisted per-site
/// override (kept parked - no promotion to linked) and resolving its web root.
/// A manual `web_root` override pins it; otherwise it auto-detects, and an
/// unresolved detection serves the root provisionally and pushes `doc_root` onto
/// `watch_roots`. Returns `None` (logging) for an invalid site name.
fn build_parked_site(
    site_name: &str,
    doc_root: &std::path::Path,
    default_php: PhpVersion,
    cfg: &orcker_config::Config,
    detect_cache: &DetectCache,
    watch_roots: &mut Vec<PathBuf>,
) -> Option<Site> {
    let mut site = match Site::parked(site_name, doc_root, default_php) {
        Ok(site) => site,
        Err(e) => {
            tracing::debug!(
                name = %site_name,
                error = %e,
                "skipping invalid parked-site name"
            );
            return None;
        }
    };

    let key = doc_root.to_string_lossy().into_owned();
    let ov = cfg.overrides.get(&key);
    if let Some(ov) = ov {
        if let Some(php) = ov.php {
            site.set_php(php);
        }
        if let Some(secure) = ov.secure {
            site.set_secure(secure);
        }
        if let Some(wp_auto_login) = ov.wp_auto_login {
            site.set_wp_auto_login(wp_auto_login);
        }
        if ov.wp_auto_login_user.is_some() {
            site.set_wp_auto_login_user(ov.wp_auto_login_user.clone());
        }
        if ov.front_controller.is_some() {
            site.set_front_controller(ov.front_controller);
        }
    }

    if let Some(rel) = ov.and_then(|o| o.web_root.as_deref()) {
        site.set_web_subpath(rel);
    } else {
        let det = detect_cache.detect(doc_root);
        site.set_web_subpath(det.subpath);
        if !det.resolved {
            watch_roots.push(doc_root.to_path_buf());
        }
    }

    Some(site)
}

/// A per-process id clients use to detect that a restart actually completed.
/// Derived from the pid + wall-clock nanos (which differ across restarts), so
/// it changes even though the in-place re-exec preserves the pid. Needs no RNG
/// dependency; collisions are irrelevant - only a *change* is observed.
fn rand_boot_id() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    std::process::id().hash(&mut h);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
        .hash(&mut h);
    // Mask to 52 bits so the value is always exactly representable as a JS
    // double (the GUI compares it after `JSON.parse`); above 2^53 precision is
    // lost and two distinct ids could round equal.
    h.finish() & ((1u64 << 52) - 1)
}

fn into_tokio_listener(
    std_listener: std::net::TcpListener,
) -> Result<tokio::net::TcpListener, DaemonError> {
    std_listener
        .set_nonblocking(true)
        .map_err(|source| DaemonError::Io {
            path: PathBuf::from("<tcp listener>"),
            source,
        })?;
    tokio::net::TcpListener::from_std(std_listener).map_err(|source| DaemonError::Io {
        path: PathBuf::from("<tcp listener>"),
        source,
    })
}

fn build_ipc_listener(dirs: &PlatformDirs) -> Result<IpcListener, DaemonError> {
    #[cfg(unix)]
    let socket_path = dirs.runtime.join("orcker.sock");
    #[cfg(unix)]
    let name = {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        let _ = std::fs::remove_file(&socket_path);
        let err_path = socket_path.clone();
        socket_path
            .clone()
            .to_fs_name::<GenericFilePath>()
            .map_err(|source| DaemonError::Io {
                path: err_path,
                source,
            })?
    };
    #[cfg(windows)]
    let name = {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        let pipe = format!("orcker-{}", std::process::id());
        pipe.clone()
            .to_ns_name::<GenericNamespaced>()
            .map_err(|source| DaemonError::Io {
                path: PathBuf::from(&pipe),
                source,
            })?
    };
    let listener = ListenerOptions::new()
        .name(name)
        .create_tokio()
        .map_err(|source| DaemonError::Io {
            path: dirs.runtime.clone(),
            source,
        })?;
    #[cfg(unix)]
    crate::secure_fs::restrict_to_owner(&socket_path).map_err(|source| DaemonError::Io {
        path: socket_path,
        source,
    })?;
    Ok(listener)
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

    fn make_dirs(tmp: &std::path::Path) -> PlatformDirs {
        PlatformDirs {
            config: tmp.join("c"),
            data: tmp.join("d"),
            state: tmp.join("s"),
            cache: tmp.join("ca"),
            runtime: tmp.join("r"),
        }
    }

    #[test]
    fn scan_sites_walks_parked_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("app1")).unwrap();
        std::fs::create_dir_all(parked_root.join("app2")).unwrap();
        std::fs::create_dir_all(parked_root.join(".hidden")).unwrap();

        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());

        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let mut names: Vec<&str> = sites.iter().map(orcker_core::Site::name).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["app1", "app2"]);
    }

    #[test]
    fn scan_sites_slugifies_underscored_parked_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("foo_bar")).unwrap();

        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());

        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let names: Vec<&str> = sites.iter().map(orcker_core::Site::name).collect();
        assert_eq!(names, vec!["foo-bar"]);
    }

    #[test]
    fn scan_sites_detects_web_root_and_collects_unresolved() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        let laravel = parked_root.join("app");
        std::fs::create_dir_all(laravel.join("public")).unwrap();
        std::fs::write(laravel.join("artisan"), b"").unwrap();
        std::fs::write(laravel.join("public/index.php"), b"").unwrap();
        std::fs::create_dir_all(parked_root.join("empty")).unwrap();

        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        let dirs = make_dirs(tmp.path());
        let (sites, watch_roots) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();

        let app = sites.iter().find(|s| s.name() == "app").unwrap();
        assert_eq!(app.web_subpath(), std::path::Path::new("public"));
        let empty = sites.iter().find(|s| s.name() == "empty").unwrap();
        assert_eq!(empty.web_subpath(), std::path::Path::new(""));
        assert_eq!(watch_roots, vec![parked_root.join("empty")]);
    }

    #[test]
    fn scan_sites_web_root_override_pins_and_skips_watching() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("app")).unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        cfg.overrides.insert(
            parked_root.join("app").to_string_lossy().into_owned(),
            orcker_config::SiteOverride {
                php: None,
                secure: None,
                web_root: Some("public".to_string()),
                wp_auto_login: None,
                wp_auto_login_user: None,
                front_controller: None,
            },
        );
        let dirs = make_dirs(tmp.path());
        let (sites, watch_roots) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let app = sites.iter().find(|s| s.name() == "app").unwrap();
        assert_eq!(app.web_subpath(), std::path::Path::new("public"));
        assert!(watch_roots.is_empty());
    }

    #[test]
    fn scan_sites_missing_parked_root_is_warning_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked.paths.insert(
            tmp.path()
                .join("does-not-exist")
                .to_string_lossy()
                .into_owned(),
        );
        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        assert!(sites.is_empty());
    }

    #[test]
    fn scan_sites_linked_wins_over_parked_collision() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("collide")).unwrap();

        let linked = Site::linked(
            "collide",
            tmp.path().join("linked-collide"),
            PhpVersion::new(8, 3),
        )
        .unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.linked.push(linked);
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());

        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(
            sites[0].document_root(),
            tmp.path().join("linked-collide").as_path()
        );
    }

    /// Build a config with `Sites/blog` parked and an override keyed by blog's
    /// document_root (the same string scan_sites computes).
    fn cfg_with_parked_blog_override(
        tmp: &std::path::Path,
        ov: orcker_config::SiteOverride,
    ) -> orcker_config::Config {
        let parked_root = tmp.join("Sites");
        std::fs::create_dir_all(parked_root.join("blog")).unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        let key = parked_root.join("blog").to_string_lossy().into_owned();
        cfg.overrides.insert(key, ov);
        cfg
    }

    #[test]
    fn scan_sites_applies_php_override() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = cfg_with_parked_blog_override(
            tmp.path(),
            orcker_config::SiteOverride {
                php: Some(PhpVersion::new(8, 5)),
                secure: None,
                web_root: None,
                wp_auto_login: None,
                wp_auto_login_user: None,
                front_controller: None,
            },
        );
        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let blog = sites.iter().find(|s| s.name() == "blog").unwrap();
        assert_eq!(blog.php(), PhpVersion::new(8, 5));
        assert!(!blog.secure());
        assert_eq!(blog.kind(), orcker_core::SiteKind::Parked);
    }

    #[test]
    fn scan_sites_applies_secure_override() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = cfg_with_parked_blog_override(
            tmp.path(),
            orcker_config::SiteOverride {
                php: None,
                secure: Some(true),
                web_root: None,
                wp_auto_login: None,
                wp_auto_login_user: None,
                front_controller: None,
            },
        );
        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let blog = sites.iter().find(|s| s.name() == "blog").unwrap();
        assert!(blog.secure());
        assert_eq!(blog.php(), PhpVersion::new(8, 3));
        assert_eq!(blog.kind(), orcker_core::SiteKind::Parked);
    }

    #[test]
    fn scan_sites_orphan_override_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("blog")).unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        cfg.overrides.insert(
            parked_root.join("ghost").to_string_lossy().into_owned(),
            orcker_config::SiteOverride {
                php: Some(PhpVersion::new(8, 5)),
                secure: Some(true),
                web_root: None,
                wp_auto_login: None,
                wp_auto_login_user: None,
                front_controller: None,
            },
        );
        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let blog = sites.iter().find(|s| s.name() == "blog").unwrap();
        assert_eq!(blog.php(), PhpVersion::new(8, 3));
        assert!(!blog.secure());
    }

    #[test]
    fn scan_sites_linked_collision_leaves_override_dormant() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("blog")).unwrap();
        let linked =
            Site::linked("blog", tmp.path().join("real-blog"), PhpVersion::new(7, 4)).unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.linked.push(linked);
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        cfg.overrides.insert(
            parked_root.join("blog").to_string_lossy().into_owned(),
            orcker_config::SiteOverride {
                php: Some(PhpVersion::new(8, 5)),
                secure: Some(true),
                web_root: None,
                wp_auto_login: None,
                wp_auto_login_user: None,
                front_controller: None,
            },
        );
        let dirs = make_dirs(tmp.path());
        let (sites, _) =
            scan_sites(&cfg, PhpVersion::new(8, 3), &dirs, &DetectCache::new()).unwrap();
        let blog = sites.iter().find(|s| s.name() == "blog").unwrap();
        assert_eq!(blog.kind(), orcker_core::SiteKind::Linked);
        assert_eq!(blog.php(), PhpVersion::new(7, 4));
        assert!(!blog.secure());
    }

    #[test]
    fn rand_boot_id_fits_52_bits() {
        let id = rand_boot_id();
        assert!(id < (1u64 << 52), "boot id {id} exceeds 52 bits");
    }

    #[test]
    fn build_router_and_routing_empty_config_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = make_dirs(tmp.path());
        let cfg = orcker_config::Config::default();
        let cache = DetectCache::new();
        let (router, wordpress_sites, laravel_sites) = build_router(&cfg, &dirs, &cache).unwrap();
        assert!(router.is_empty());
        assert!(wordpress_sites.is_empty());
        assert!(laravel_sites.is_empty());
        let (router2, wordpress_sites2, laravel_sites2, watch_roots) =
            build_routing(&cfg, &dirs, &cache).unwrap();
        assert!(router2.is_empty());
        assert!(wordpress_sites2.is_empty());
        assert!(laravel_sites2.is_empty());
        assert!(watch_roots.is_empty());
    }

    #[test]
    fn build_routing_includes_parked_site() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("shop")).unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        let dirs = make_dirs(tmp.path());
        let (router, wordpress_sites, _laravel_sites, watch_roots) =
            build_routing(&cfg, &dirs, &DetectCache::new()).unwrap();
        assert!(!router.is_empty());
        assert_eq!(wordpress_sites.get("shop"), Some(&false));
        assert_eq!(watch_roots, vec![parked_root.join("shop")]);
    }

    #[test]
    fn build_routing_detects_wordpress_site() {
        let tmp = tempfile::tempdir().unwrap();
        let parked_root = tmp.path().join("Sites");
        std::fs::create_dir_all(parked_root.join("blog")).unwrap();
        std::fs::write(parked_root.join("blog").join("wp-config.php"), b"<?php").unwrap();
        let mut cfg = orcker_config::Config::default();
        cfg.parked
            .paths
            .insert(parked_root.to_string_lossy().into_owned());
        let dirs = make_dirs(tmp.path());
        let (router, wordpress_sites, _laravel_sites, _watch_roots) =
            build_routing(&cfg, &dirs, &DetectCache::new()).unwrap();
        assert!(!router.is_empty());
        assert_eq!(wordpress_sites.get("blog"), Some(&true));
    }

    #[test]
    fn parked_site_name_filters_and_slugifies() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("MyApp")).unwrap();
        std::fs::create_dir_all(root.join("foo_bar")).unwrap();
        std::fs::create_dir_all(root.join("ex.com")).unwrap();
        std::fs::create_dir_all(root.join("keep--me")).unwrap();
        std::fs::create_dir_all(root.join(".hidden")).unwrap();
        std::fs::create_dir_all(root.join("linked")).unwrap();
        std::fs::write(root.join("afile"), b"x").unwrap();

        let mut linked = std::collections::HashSet::new();
        linked.insert("linked");

        let mut got: std::collections::BTreeMap<String, Option<String>> =
            std::collections::BTreeMap::default();
        for entry in std::fs::read_dir(root).unwrap().flatten() {
            let key = entry.file_name().to_string_lossy().into_owned();
            got.insert(key, parked_site_name(&entry, &linked));
        }
        assert_eq!(got.get("MyApp").unwrap().as_deref(), Some("myapp"));
        assert_eq!(got.get("foo_bar").unwrap().as_deref(), Some("foo-bar"));
        assert_eq!(got.get("ex.com").unwrap().as_deref(), Some("ex-com"));
        assert_eq!(got.get("keep--me").unwrap().as_deref(), Some("keep--me"));
        assert_eq!(got.get(".hidden").unwrap().as_deref(), None);
        assert_eq!(got.get("afile").unwrap().as_deref(), None);
        assert_eq!(got.get("linked").unwrap().as_deref(), None);
    }

    #[test]
    fn load_or_default_config_uses_defaults_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("absent").join("orcker.toml");
        let cfg = load_or_default_config(&path).unwrap();
        assert_eq!(cfg.tld, orcker_config::Config::default().tld);
    }

    #[test]
    fn load_or_default_config_loads_existing_then_rejects_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("orcker.toml");
        orcker_config::Config::default().save(&path).unwrap();
        assert!(load_or_default_config(&path).is_ok());
        std::fs::write(&path, b"this is = not valid = toml {{{").unwrap();
        assert!(load_or_default_config(&path).is_err());
    }

    #[test]
    fn load_or_generate_ca_generates_then_reloads() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = make_dirs(tmp.path());
        std::fs::create_dir_all(&dirs.data).unwrap();
        let ca = load_or_generate_ca(&dirs).unwrap();
        let fp1 = ca.fingerprint_sha256();
        assert!(dirs.data.join("ca.cert.pem").is_file());
        assert!(dirs.data.join("ca.key.pem").is_file());
        let ca2 = load_or_generate_ca(&dirs).unwrap();
        assert_eq!(ca2.fingerprint_sha256(), fp1);
    }

    #[tokio::test]
    async fn into_tokio_listener_converts_a_bound_socket() {
        let std_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let local = std_listener.local_addr().unwrap();
        let tokio_listener = into_tokio_listener(std_listener).unwrap();
        assert_eq!(tokio_listener.local_addr().unwrap(), local);
    }

    #[test]
    fn ca_validity_spans_the_past_into_the_future() {
        let v = ca_validity().unwrap();
        let now = time::OffsetDateTime::now_utc();
        assert!(v.not_before() < now);
        assert!(v.not_after() > now);
    }
}
