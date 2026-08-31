//! Validated site-name newtype.

use std::fmt;
use std::str::FromStr;

use crate::error::{SiteNameErrorReason, StackError};

/// A validated project site name (a single DNS label, e.g. `"acme-shop"`).
///
/// Doubles as the compose project network name and the prefix of the named
/// database volume, so it is constrained to what both Docker and DNS accept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SiteName(String);

impl SiteName {
    /// Validates and constructs from a `&str`.
    pub fn new(raw: &str) -> Result<Self, StackError> {
        validate(raw).map_err(|reason| StackError::InvalidSiteName {
            input: raw.to_owned(),
            reason,
        })?;
        Ok(Self(raw.to_owned()))
    }

    /// Returns the validated name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SiteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SiteName {
    type Err = StackError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Pinned, ordered validation: emptiness, then charset, then hyphen placement,
/// then length. The order decides which reason an input that breaks several
/// rules reports, so it is part of the contract the tests pin.
fn validate(raw: &str) -> Result<(), SiteNameErrorReason> {
    if raw.is_empty() {
        return Err(SiteNameErrorReason::Empty);
    }
    for &b in raw.as_bytes() {
        let ok = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
        if !ok {
            return Err(SiteNameErrorReason::InvalidCharacter);
        }
    }
    if raw.starts_with('-') || raw.ends_with('-') {
        return Err(SiteNameErrorReason::LeadingOrTrailingHyphen);
    }
    if raw.len() > 63 {
        return Err(SiteNameErrorReason::TooLong);
    }
    Ok(())
}
