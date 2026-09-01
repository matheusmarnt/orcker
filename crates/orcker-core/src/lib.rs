//! Pure domain types and host→site routing for Orcker.
//!
//! This crate is the foundation of the Orcker workspace: every other crate
//! depends on it. It is **pure**: no I/O, no async, no internal `orcker-*`
//! dependencies. Side effects belong behind traits in `orcker-platform` and
//! similar adapter crates.

#![forbid(unsafe_code)]

pub mod detect;
mod domain;
mod error;
mod host;
mod net;
mod php;
pub mod php_directives;
pub mod php_extensions;
pub mod php_pool;
pub mod php_settings;
pub mod ports;
mod project;
mod proxy;
mod route_rule;
mod router;
pub mod service_directives;
mod site;
mod tld;

/// `Server` header value the proxy stamps on its own (synthetic, non-forwarded)
/// responses. It doubles as the signature the macOS privileged-port redirect
/// probe looks for: confirming a connection to `127.0.0.1:80` reaches *this*
/// daemon's proxy - rather than some other process or a stale `pf` rule holding
/// the port - instead of merely confirming *something* answers.
///
/// It is a cross-crate contract: `orcker-proxy` sets it (`server.rs`) and
/// `orcker-platform`'s redirect probe (`port_redirect.rs`) checks for it.
/// Changing the value means updating both ends.
pub const PROXY_SERVER_ID: &str = "orcker";

/// Subject Common Name of orcker's local development CA.
///
/// A cross-crate contract: `bin/orckerd` stamps it onto the generated CA
/// (`startup.rs`), and `orcker-helper` checks for it before removing a CA from
/// the system trust store - so the privileged helper only ever deletes a cert
/// it can confirm is orcker's, never an unrelated trusted root. Changing the
/// value means re-generating existing users' CAs, so treat it as frozen.
pub const CA_COMMON_NAME: &str = "Orcker Local CA";

pub use detect::{detect, Detection, ProjectSignals};
pub use domain::{choose_primary, effective_domains, Domain};
pub use error::{
    CoreError, DomainErrorReason, PhpVersionErrorReason, ProxyNameErrorReason,
    ProxyRuleErrorReason, RouteRuleErrorReason, SiteNameErrorReason, TldErrorReason,
    UpstreamTargetErrorReason,
};
pub use net::is_lan_source;
pub use php::{PhpVersion, FIRST_SUPPORTED_MINOR};
pub use php_directives::{DirectiveError, DirectiveNameErrorReason};
pub use php_extensions::{ExtError, NameErrorReason, PathErrorReason};
pub use php_pool::{PoolNameErrorReason, PoolSettingError, PoolValueErrorReason};
pub use php_settings::{PhpSettingError, ValueErrorReason};
pub use ports::{allocate_port, PortProbe, FIRST_PROJECT_PORT, LAST_PROJECT_PORT};
pub use project::ContainerProject;
pub use proxy::{match_rule, validate_proxy_name, ProxyRule, ProxySite, UpstreamTarget};
pub use route_rule::RouteRule;
pub use router::{Route, RouterConfig, SiteRouter};
pub use site::{normalize_site_name, slugify_site_name, Site, SiteKind};
pub use tld::Tld;
