//! The daemon's HTTP download edge.
//!
//! [`Downloader`] is transport-agnostic (only `async-trait`, no `reqwest`) so
//! tests inject a fake; [`ReqwestDownloader`] is the one production impl.
//! SHA-256 verification of the fetched bytes is the caller's job, not the
//! downloader's - every consumer here checks a published digest or a minisign
//! signature before trusting what came back.
//!
//! Consumers: self-update, the tool installers (Composer, Node, Bun) and the
//! `cloudflared` install. It lives in the binary because `reqwest` does.

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

/// Error returned by [`Downloader::download`].
///
/// Carries a flattened message rather than wrapping a transport type so that
/// test fakes can construct it without pulling in `reqwest`, and so the public
/// surface stays transport-agnostic.
#[derive(Debug, Error)]
pub enum DownloadError {
    /// The transfer failed - connection, TLS, timeout, or a non-success HTTP
    /// status.
    #[error("download failed for {url}: {reason}")]
    Transport {
        /// The URL that failed to download.
        url: String,
        /// Flattened underlying error.
        reason: String,
    },
}

/// Fetches bytes over HTTP.
#[async_trait]
pub trait Downloader: Send + Sync + 'static {
    /// Fetch the body bytes at `url`.
    ///
    /// # Errors
    ///
    /// Returns [`DownloadError::Transport`] on connection, TLS, timeout or
    /// non-success status.
    async fn download(&self, url: &str) -> Result<Vec<u8>, DownloadError>;

    /// Fetch the body bytes at `url`, reporting progress as
    /// `(bytes_so_far, total_bytes)`: `total` is `None` when the server sends no
    /// `Content-Length`. The default ignores `progress` and delegates to
    /// [`Self::download`]; [`ReqwestDownloader`] overrides it so long installs
    /// can show a live byte count instead of appearing to hang.
    ///
    /// # Errors
    ///
    /// Same as [`Self::download`].
    async fn download_with_progress(
        &self,
        url: &str,
        progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
    ) -> Result<Vec<u8>, DownloadError> {
        let _ = progress;
        self.download(url).await
    }
}

/// `reqwest`-backed downloader (rustls, no OpenSSL; follows redirects).
pub struct ReqwestDownloader {
    client: reqwest::Client,
}

impl ReqwestDownloader {
    /// Construct a fresh client. Sets a `User-Agent` (some hosts - notably the
    /// GitHub API used for Bun releases - reject requests without one); falls
    /// back to the default client if the builder fails.
    ///
    /// Bounds the two ways a download can wedge indefinitely: a `connect_timeout`
    /// for a connection that never establishes, and a `read_timeout` (idle/stall
    /// timeout between reads) for a body that stops mid-stream. reqwest's default
    /// is *unbounded*: the cause of a PHP install "spinning" for minutes until
    /// the kernel gives up. Deliberately no hard overall `.timeout()`, so a
    /// slow-but-progressing large download isn't killed.
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("orcker/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for ReqwestDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Downloader for ReqwestDownloader {
    async fn download(&self, url: &str) -> Result<Vec<u8>, DownloadError> {
        self.download_with_progress(url, &|_, _| {}).await
    }

    /// Streams the body chunk-by-chunk so progress can be reported and a
    /// mid-stream stall trips `read_timeout` rather than buffering forever. The
    /// `Content-Length` capacity hint is clamped (the header is server-controlled;
    /// a bogus huge value would otherwise abort the daemon on a failed
    /// allocation), and the `Vec` still grows past the cap as needed.
    async fn download_with_progress(
        &self,
        url: &str,
        progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
    ) -> Result<Vec<u8>, DownloadError> {
        let transport = |e: reqwest::Error| DownloadError::Transport {
            url: url.to_owned(),
            reason: e.to_string(),
        };
        let mut resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(transport)?
            .error_for_status()
            .map_err(transport)?;
        let total = resp.content_length();
        let cap = total.map_or(0, |n| n.min(64 * 1024 * 1024) as usize);
        let mut buf = Vec::with_capacity(cap);
        progress(0, total);
        while let Some(chunk) = resp.chunk().await.map_err(transport)? {
            buf.extend_from_slice(&chunk);
            progress(buf.len() as u64, total);
        }
        Ok(buf)
    }
}

/// Lowercase hex SHA-256 of `bytes`, for checking a downloaded artifact against
/// its published digest.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_the_empty_input_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
