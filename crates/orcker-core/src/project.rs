//! Container projects: a directory served by its own container stack.
//!
//! A container project is registered by `orcker link` and routed by the proxy
//! to `http://127.0.0.1:<port>`, the loopback port allocated for it by
//! [`crate::ports`]. It is deliberately *not* a [`crate::Site`]: a `Site` is
//! served from disk (document root, web subpath, PHP version), while a project
//! is served by containers and reaches the router as a
//! [`ProxySite`](crate::ProxySite), exactly the shape the SPEC-0005 spike
//! proved out.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::proxy::{ProxySite, UpstreamTarget};

/// A linked container project.
///
/// `name` is a DNS label, validated and lowercased at construction and
/// immutable afterwards, exactly like [`crate::Site`]'s. `root` is the project
/// directory (where `orcker.yml` lives); like a site's document root it is not
/// validated here, because this crate is pure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerProject {
    name: String,
    root: PathBuf,
    port: u16,
    #[serde(default)]
    secure: bool,
}

impl ContainerProject {
    /// Registers a project under `name`, rooted at `root`, on loopback `port`.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidSiteName`] when `name` is not a DNS label.
    pub fn new(name: &str, root: impl Into<PathBuf>, port: u16) -> Result<Self, CoreError> {
        Ok(Self {
            name: crate::site::validate_and_lowercase_name(name)?,
            root: root.into(),
            port,
            secure: false,
        })
    }

    /// The validated, lowercased DNS-label name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The project directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The allocated loopback port.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Whether the site is served over HTTPS.
    #[must_use]
    pub fn secure(&self) -> bool {
        self.secure
    }

    /// Sets whether the site is served over HTTPS.
    pub fn set_secure(&mut self, secure: bool) {
        self.secure = secure;
    }

    /// The loopback upstream the proxy forwards to.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidUpstreamTarget`] only if the port is unusable as a
    /// URL port, which the type system already rules out for a linked project.
    pub fn upstream(&self) -> Result<UpstreamTarget, CoreError> {
        UpstreamTarget::from_url_str(&format!("http://127.0.0.1:{}", self.port))
    }

    /// The whole-host proxy entry this project contributes to the router.
    ///
    /// # Errors
    ///
    /// Propagates [`Self::upstream`], or [`CoreError::InvalidProxyName`] if the
    /// stored name is not a valid proxy name.
    pub fn proxy_site(&self) -> Result<ProxySite, CoreError> {
        let mut proxy = ProxySite::new(&self.name, self.upstream()?)?;
        proxy.set_secure(self.secure);
        Ok(proxy)
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
    use crate::error::SiteNameErrorReason;

    #[test]
    fn upstream_is_the_allocated_loopback_port() {
        let project = ContainerProject::new("spike", "/srv/spike", 20007).unwrap();
        let upstream = project.upstream().unwrap();
        assert_eq!(upstream.host(), "127.0.0.1");
        assert_eq!(upstream.port(), 20007);
        assert!(!upstream.secure(), "the upstream itself is plain HTTP");
    }

    #[test]
    fn name_is_validated_and_lowercased() {
        let project = ContainerProject::new("Spike", "/srv/spike", 20000).unwrap();
        assert_eq!(project.name(), "spike");

        match ContainerProject::new("not_a_label", "/srv/x", 20000) {
            Err(CoreError::InvalidSiteName { reason, .. }) => {
                assert_eq!(reason, SiteNameErrorReason::InvalidCharacter);
            }
            other => panic!("expected InvalidSiteName, got {other:?}"),
        }
    }

    #[test]
    fn proxy_site_carries_the_name_upstream_and_secure_flag() {
        let mut project = ContainerProject::new("spike", "/srv/spike", 20007).unwrap();
        project.set_secure(true);
        let proxy = project.proxy_site().unwrap();
        assert_eq!(proxy.name(), "spike");
        assert_eq!(proxy.target().port(), 20007);
        assert!(proxy.secure());
    }
}
