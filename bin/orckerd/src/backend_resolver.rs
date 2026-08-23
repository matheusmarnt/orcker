//! The proxy's document-root seams: backend resolution and login tokens.
//!
//! Yerd resolved a linked site to the FPM pool serving its PHP version. That
//! pool is gone with the native runtime, and the container that replaces it is
//! not wired yet (SPEC-0003+), so every document-root site now resolves to a
//! typed error naming the gap rather than to a backend.
//!
//! Sites registered as proxy entries are unaffected: they never reach this
//! resolver, the proxy forwards them to their configured upstream directly.

use async_trait::async_trait;

use orcker_proxy::{Backend, BackendResolver, LoginTokenConsumer, ProxyError};

/// Resolver for document-root sites while the fork has no runtime to serve
/// them with.
pub struct DaemonBackendResolver;

#[async_trait]
impl BackendResolver for DaemonBackendResolver {
    async fn backend_for(&self, site: &orcker_core::Site) -> Result<Backend, ProxyError> {
        Err(ProxyError::BackendResolver {
            host: site.name().to_owned(),
            source: Box::new(std::io::Error::other(
                "no runtime is wired for document-root sites yet; register the site \
                 as a proxy entry, or wait for the Docker engine",
            )),
        })
    }
}

/// Token consumer for the `WordPress` one-click login.
///
/// Always `None`: minting died with the native runtime (the flow needed an
/// `auto_prepend_file` injected into the FPM pool), so no token can be valid.
/// Fail-closed by construction rather than by an empty registry that a future
/// caller could accidentally populate.
pub struct NoLoginTokens;

impl LoginTokenConsumer for NoLoginTokens {
    fn consume(&self, site: &str, token: &str) -> Option<String> {
        let _ = (site, token);
        None
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use orcker_core::{PhpVersion, Site};

    #[tokio::test]
    async fn every_document_root_site_resolves_to_a_typed_error() {
        let site = Site::linked("blog", "/srv/www/blog", PhpVersion::new(8, 3)).unwrap();
        let err = DaemonBackendResolver.backend_for(&site).await.unwrap_err();
        match err {
            ProxyError::BackendResolver { host, .. } => assert_eq!(host, "blog"),
            other => panic!("expected BackendResolver, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn direct_script_execution_stays_off_by_default() {
        let site = Site::linked("blog", "/srv/www/blog", PhpVersion::new(8, 3)).unwrap();
        assert!(
            !DaemonBackendResolver
                .allows_direct_script_execution(&site)
                .await
        );
    }

    #[test]
    fn no_login_token_is_ever_accepted() {
        assert_eq!(NoLoginTokens.consume("blog", "deadbeef"), None);
    }
}
