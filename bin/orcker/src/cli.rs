//! CLI surface (clap-derived).

use std::path::PathBuf;

/// Top-level parser. `orcker` is a thin `orcker-ipc` client of the `orckerd` daemon.
#[derive(clap::Parser, Debug)]
#[command(
    name = "orcker",
    version,
    about = "Orcker CLI - talks to the orckerd daemon"
)]
pub struct Cli {
    /// Emit machine-readable JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// CLI subcommands. Each maps to exactly one [`orcker_ipc::Request`].
#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    /// Check that the daemon is alive.
    Ping,
    /// List every parked or linked site.
    Sites,
    /// Park a directory: each of its child directories becomes a `.test` site.
    Park {
        /// Directory to park.
        path: PathBuf,
    },
    /// Link a single directory as a named site. With one argument, infers
    /// whichever of name/path is missing; with none, links the current
    /// directory under its folder name.
    Link {
        /// A site name (bare word), or a directory to link (its folder name
        /// becomes the site name). Omit entirely to link the current
        /// directory.
        name_or_path: Option<String>,
        /// Directory to serve, when the first argument is a name. Omit to
        /// use the current directory.
        path: Option<PathBuf>,
    },
    /// Remove a linked site by name.
    Unlink {
        /// Site name to remove.
        name: String,
    },
    /// Un-park a directory: removes it from the parked set so its child
    /// directories stop being served. Linked sites are untouched.
    Unpark {
        /// Directory to un-park (run `orcker list parked` to see the exact paths).
        path: PathBuf,
    },
    /// Install a component (currently: a PHP version).
    Install {
        /// What to install.
        #[command(subcommand)]
        target: InstallTarget,
    },
    /// Restart a component's process (currently: a PHP FPM pool).
    Restart {
        /// What to restart.
        #[command(subcommand)]
        target: RestartTarget,
    },
    /// Uninstall a component, or orcker itself.
    ///
    /// With a subcommand (`php`/`tool`) removes that component via the daemon.
    /// With no subcommand, fully uninstalls orcker from this machine: config,
    /// data, downloads, the PATH entry, the daemon service, and - when run with
    /// `sudo` - the system-level trust/resolver/port changes from `elevate`.
    /// Prompts for confirmation first unless `--yes` is given.
    Uninstall {
        /// What to uninstall; omit to uninstall orcker entirely.
        #[command(subcommand)]
        target: Option<UninstallTarget>,
        /// Skip the confirmation prompt (only affects the full uninstall).
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// List installed components (currently: PHP versions).
    List {
        /// What to list.
        #[command(subcommand)]
        target: ListTarget,
    },
    /// List installable dev tools (Composer, Node, Bun) and their install status.
    Tools,
    /// Check for a Orcker self-update (add `--yes` to apply it).
    Update {
        /// Apply the self-update: download, verify, and install the new version,
        /// then restart. Without it, only check and report.
        #[arg(long)]
        yes: bool,
        /// Use the edge (pre-release / RC) channel for this run. With `--yes`,
        /// also makes edge the saved default.
        #[arg(long, conflicts_with = "stable")]
        edge: bool,
        /// Use the stable channel for this run. With `--yes`, also resets the
        /// saved default to stable.
        #[arg(long)]
        stable: bool,
        /// Allow a downgrade (e.g. moving from a newer pre-release to stable).
        #[arg(long, requires = "yes")]
        force: bool,
    },
    /// Publish a local site to the internet via a Cloudflare Tunnel.
    Tunnel {
        /// What to do.
        #[command(subcommand)]
        action: TunnelAction,
    },
    /// Inspect emails captured by the built-in mail server.
    Mail {
        /// What to do.
        #[command(subcommand)]
        action: MailAction,
    },
    /// Show a snapshot of daemon, proxy, DNS, ports, CA, and PHP health.
    Status,
    /// Diagnose common problems; `orcker doctor fix` attempts safe repairs.
    Doctor {
        /// Optional action; omit to only report, `fix` to attempt repairs.
        #[command(subcommand)]
        action: Option<DoctorAction>,
    },
    /// Serve a site over HTTPS (promotes a parked site to a linked entry).
    Secure {
        /// Site name.
        name: String,
    },
    /// Stop serving a site over HTTPS.
    Unsecure {
        /// Site name.
        name: String,
    },
    /// Set the directory a site is served from (its web root), e.g.
    /// `orcker root myapp public` for a Laravel app. With `--auto` (or no path),
    /// reset the site to automatic framework detection.
    Root {
        /// Site name.
        name: String,
        /// Served directory, relative to the site's folder (or an absolute path
        /// inside it). Omit with `--auto` to reset to auto-detection.
        path: Option<String>,
        /// Reset the site to automatic web-root detection.
        #[arg(long)]
        auto: bool,
    },
    /// Route all requests through a site's front controller (`index.php`), or
    /// execute named `.php` files directly. Frameworks served from a subdir
    /// (Laravel, ...) default `on`; plain and `WordPress` sites default `off`.
    FrontController {
        /// Site name.
        name: String,
        /// `on` funnels every request through `index.php`; `off` executes the
        /// named `.php` directly.
        state: OnOff,
    },
    /// Manage a site's or whole-host proxy's routable domains (add/remove
    /// domains, subdomains and wildcards, and change the primary domain).
    Domain {
        /// What to do.
        #[command(subcommand)]
        action: DomainAction,
    },
    /// Manage reverse proxies: a whole-host proxy (`reverb.test` → a running
    /// service) or a path rule on an existing site (`app.test/app` → a service).
    Proxy {
        /// What to do.
        #[command(subcommand)]
        action: ProxyAction,
    },
    /// Manage a site's routing rules: URIs under a path prefix that match no
    /// real file are handled by a target inside the site (`/api` →
    /// `api/index.php`, or `/` → `index.html` for a JavaScript SPA).
    Route {
        /// What to do.
        #[command(subcommand)]
        action: RouteAction,
    },
    /// Grant orcker OS-level privileges (run via `sudo`). No subcommand = all.
    Elevate {
        /// Which privilege to grant; omit to grant all.
        #[command(subcommand)]
        target: Option<ElevateTarget>,
    },
    /// Revert what `elevate` configured (run via `sudo`). No subcommand = all.
    Unelevate {
        /// Which privilege to revert; omit to revert all.
        #[command(subcommand)]
        target: Option<ElevateTarget>,
    },
    /// Add or remove orcker's shim directory (`php`, `composer`, …) from your
    /// shell's PATH. Local - does not talk to the daemon.
    Path {
        /// What to do.
        #[command(subcommand)]
        action: PathAction,
    },
    /// Run a tool under the PHP version pinned to a site - the one its web
    /// requests use - instead of the global default. The site is the one
    /// containing the current directory, or `--site <name>`; outside any site,
    /// the global default is used. Everything after the tool is passed
    /// straight through, so `--site`/`--json` must come *before* it (e.g.
    /// `orcker exec --site blog php -v`). The bare `php` and `composer` shims are
    /// unaffected and still use the global default. `-h`/`--help` go to the
    /// tool, so use `orcker help exec` for this command's own help. Local -
    /// execs PHP directly. (Unix only.)
    // `disable_help_flag` because clap otherwise matches `-h`/`--help` before
    // `trailing_var_arg` starts collecting, so `orcker exec composer --help`
    // would print orcker's help instead of Composer's.
    /// Serve Orcker's tools to AI agents over MCP on stdin/stdout. Not meant to be
    /// run by hand: an agent spawns it. Register it once, e.g.
    /// `claude mcp add --scope user orcker -- orcker mcp`. Tools are served only
    /// when AI Agents is enabled in Orcker's General settings.
    Mcp,
    /// Expose your `.test` sites to other devices on the LAN, or check/disable
    /// that exposure. See `orcker remote-setup` to provision a device.
    Lan {
        /// What to do.
        #[command(subcommand)]
        action: LanAction,
    },
    /// Mint a one-time bootstrap command to provision another device (installs
    /// Orcker's CA and points its `.test` resolver here). Requires LAN mode on.
    RemoteSetup,
}

/// `orcker lan <action>`.
#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanAction {
    /// Turn LAN exposure on (persists + restarts the daemon to re-bind).
    Enable,
    /// Turn LAN exposure off (persists + restarts the daemon to re-bind).
    Disable,
    /// Show LAN exposure state: configured vs effective, the LAN IP, and the
    /// next privileged step if any.
    Status,
}

/// A binary on/off toggle argument (e.g. `orcker front-controller <name> on`).
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnOff {
    /// Turn the setting on.
    On,
    /// Turn the setting off.
    Off,
}

impl OnOff {
    /// `true` for [`OnOff::On`].
    #[must_use]
    pub fn is_on(self) -> bool {
        matches!(self, OnOff::On)
    }
}

/// Action of `orcker path`.
#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathAction {
    /// Add orcker's bin dir to your shell startup file (idempotent).
    Install,
    /// Remove the orcker PATH block from your shell startup file.
    Uninstall,
    /// Print the shell snippet without modifying any file (for manual `eval`).
    Print,
}

/// Action of `orcker proxy`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum ProxyAction {
    /// Add a proxy. Two arguments create a whole-host proxy
    /// (`orcker proxy add reverb http://localhost:8080`); three attach a path rule
    /// to an existing site (`orcker proxy add myapp /app http://127.0.0.1:8080`).
    Add {
        /// A proxy name, which may be dotted (`api.account`), or a site name
        /// (path rule).
        first: String,
        /// The upstream URL (whole-host), or the path prefix (path rule).
        second: String,
        /// The upstream URL, when the second argument is a path prefix.
        third: Option<String>,
    },
    /// Remove a proxy. One argument removes a whole-host proxy
    /// (`orcker proxy remove reverb`); two remove a site's path rule
    /// (`orcker proxy remove myapp /app`).
    Remove {
        /// A whole-host proxy name, possibly dotted, or a site name (with a
        /// path prefix).
        target: String,
        /// The path prefix, when removing a site's path rule.
        prefix: Option<String>,
    },
    /// List whole-host proxies and per-site path rules.
    List,
}

/// Action of `orcker route`.
///
/// Distinct from [`ProxyAction`]: a proxy rule forwards to a running HTTP
/// service, a routing rule resolves to a file inside the site's own web root.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum RouteAction {
    /// Add a routing rule. Requests under `prefix` that match no real file are
    /// handled by `target`, a path relative to the site's web root. A `.php`
    /// target runs as a nested front controller; anything else is served as a
    /// static file.
    Add {
        /// Site name.
        site: String,
        /// Path prefix, e.g. `/api` (or `/` to catch everything).
        prefix: String,
        /// Target relative to the site's web root, e.g. `api/index.php`.
        target: String,
    },
    /// Remove a routing rule from a site by its path prefix.
    Remove {
        /// Site name.
        site: String,
        /// Path prefix to remove.
        prefix: String,
    },
    /// List routing rules. A site whose web root holds an `index.html` and no
    /// `index.php` already gets SPA routing automatically, with no rule.
    List {
        /// Site name; omit to list every site's rules.
        site: Option<String>,
    },
}

/// Action of `orcker domain`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum DomainAction {
    /// List a site's domains (primary marked), or all sites' domains with no
    /// argument.
    List {
        /// Site name; omit to list every site's domains.
        site: Option<String>,
    },
    /// Add a domain to a site or whole-host proxy: an exact host
    /// (`api.myapp.test`) or a single-label wildcard (`*.myapp.test`).
    Add {
        /// Site or proxy name.
        site: String,
        /// Full domain FQDN under the configured TLD.
        domain: String,
    },
    /// Remove a domain from a site or whole-host proxy. At least one exact
    /// domain must remain.
    Remove {
        /// Site or proxy name.
        site: String,
        /// Full domain FQDN to remove.
        domain: String,
    },
    /// Set a site's or whole-host proxy's primary (canonical) domain. Must be an
    /// exact domain; it is added if not already present.
    Primary {
        /// Site or proxy name.
        site: String,
        /// Full domain FQDN to make primary.
        domain: String,
    },
    /// Reset a site's or whole-host proxy's domains to the default (its
    /// `{name}.{tld}` apex only).
    Reset {
        /// Site or proxy name.
        site: String,
    },
}

/// Action of `orcker tunnel`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum TunnelAction {
    /// Download the `cloudflared` binary (required before sharing a site).
    Install,
    /// Share a site publicly via a Quick Tunnel (a random `*.trycloudflare.com`
    /// URL). Requires `cloudflared` to be installed.
    Share {
        /// Site name (e.g. `app` or `app.test`).
        site: String,
    },
    /// Stop sharing a site.
    Stop {
        /// Site name whose tunnel to stop.
        site: String,
    },
    /// Show live tunnels and `cloudflared` install status.
    Status,
    /// Log in to a Cloudflare account (opens a browser) for Named Tunnels.
    Login,
    /// Create a named tunnel on the logged-in account.
    Create {
        /// The tunnel name to create.
        name: String,
    },
    /// Delete a named tunnel from the account and forget it locally.
    Delete {
        /// The tunnel name to delete.
        name: String,
    },
    /// List the named tunnels recorded locally.
    List,
    /// Route a public hostname to a named tunnel (creates the DNS record).
    Route {
        /// Tunnel name (or UUID) to route to.
        tunnel: String,
        /// Public hostname to create (on your Cloudflare domain).
        hostname: String,
    },
    /// Set (or clear, with `--clear`) a site's persisted public hostname.
    SetHost {
        /// Site name.
        site: String,
        /// The public hostname to assign. Required unless `--clear` is given, so
        /// forgetting it can't silently clear the mapping.
        #[arg(required_unless_present = "clear")]
        hostname: Option<String>,
        /// Clear the site's hostname instead of setting one.
        #[arg(long, conflicts_with = "hostname")]
        clear: bool,
    },
    /// (Re)start the named tunnel, exposing every site that has a hostname set.
    /// Requires login and a created tunnel.
    Publish,
    /// Stop the named tunnel (takes every named site offline).
    Unpublish,
}

/// Action of `orcker mail`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum MailAction {
    /// List captured emails (newest first).
    List,
    /// Show one captured email's headers and body by id.
    Show {
        /// The email id (from `orcker mail list`).
        id: String,
    },
    /// Delete every captured email.
    Clear,
}

/// Action of `orcker doctor`.
#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorAction {
    /// Attempt safe, unprivileged repairs (e.g. restart a crashed FPM pool).
    Fix,
}

/// Target of `orcker restart`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum RestartTarget {
    /// Restart the daemon itself (briefly interrupts all sites + this command).
    Daemon,
}

/// Target of `orcker uninstall`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum UninstallTarget {
    /// Uninstall a dev tool (`composer`, `node`, `bun`, `laravel`, `wp-cli`).
    Tool {
        /// Tool id: `composer`, `node`, `bun`, `laravel`, or `wp-cli`.
        id: String,
    },
}

/// Target of `orcker install`.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum InstallTarget {
    /// Install a dev tool (`composer`, `node`, `bun`, `laravel`, `wp-cli`) at
    /// its latest release.
    Tool {
        /// Tool id: `composer`, `node`, `bun`, `laravel`, or `wp-cli`.
        id: String,
    },
}

/// Target of `orcker list`.
#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListTarget {
    /// List the registered parked directory roots (including empty ones, which
    /// produce no sites and so don't appear in `orcker sites`).
    Parked,
}

/// A single privilege managed by `orcker elevate` / `orcker unelevate`.
#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevateTarget {
    /// Trust the local CA in the OS system store.
    Trust,
    /// Route `*.<tld>` queries to orcker's DNS responder.
    Resolver,
    /// Allow the daemon to bind privileged ports 80/443 (setcap).
    Ports,
    /// Install the macOS LAN pf redirect so other devices reach 80/443 (macOS
    /// only; on Linux this reuses the `ports` setcap grant). Run after
    /// `orcker lan enable`.
    Lan,
}
