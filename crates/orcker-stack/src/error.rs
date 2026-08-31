//! Typed errors for the pure stack model.

use thiserror::Error;

/// Every way a stack model can be rejected.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StackError {
    /// A site name failed DNS-label validation.
    #[error("invalid site name {input:?}: {reason}")]
    InvalidSiteName {
        /// The rejected input, verbatim.
        input: String,
        /// Why it was rejected.
        reason: SiteNameErrorReason,
    },
    /// A PHP version string is malformed or outside the supported range.
    #[error("invalid PHP version {input:?}: {reason}")]
    InvalidPhpVersion {
        /// The rejected input, verbatim.
        input: String,
        /// Why it was rejected.
        reason: PhpVersionErrorReason,
    },
    /// A published port is unusable.
    #[error("invalid {field} port {value}: {reason}")]
    InvalidPort {
        /// Which port of the pair was rejected.
        field: PortField,
        /// The rejected value.
        value: u16,
        /// Why it was rejected.
        reason: PortErrorReason,
    },
}

/// Why a site name was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SiteNameErrorReason {
    /// The name was empty.
    #[error("empty")]
    Empty,
    /// A byte outside `[a-z0-9-]` was present.
    #[error("only lowercase letters, digits and hyphens are allowed")]
    InvalidCharacter,
    /// The name started or ended with a hyphen.
    #[error("must not start or end with a hyphen")]
    LeadingOrTrailingHyphen,
    /// The name exceeded the 63-octet DNS label limit.
    #[error("longer than 63 characters")]
    TooLong,
}

/// Why a PHP version was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PhpVersionErrorReason {
    /// The input was empty.
    #[error("empty")]
    Empty,
    /// The input was not a `major.minor` pair of decimal numbers.
    #[error("expected a major.minor version like \"8.3\"")]
    Malformed,
    /// The version parsed but is outside the range Orcker builds images for.
    #[error("outside the supported range 8.1..=8.5")]
    Unsupported,
}

/// Which port of the published pair an [`StackError::InvalidPort`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PortField {
    /// The loopback port nginx publishes for HTTP.
    #[error("http loopback")]
    HttpLoopback,
    /// The loopback port the app container publishes for the Vite dev server.
    #[error("vite")]
    Vite,
}

/// Why a port was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PortErrorReason {
    /// Port 0 asks the kernel for an ephemeral port, which a persisted stack
    /// cannot rely on.
    #[error("must not be zero")]
    Zero,
    /// Two services cannot publish the same loopback port.
    #[error("duplicates the http loopback port")]
    DuplicatesHttpLoopback,
}
