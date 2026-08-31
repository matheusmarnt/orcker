//! Docker-flavoured version parsing and ordering.
//!
//! Neither Docker nor compose reports strict semver: `docker compose` prefixes
//! a `v`, release candidates carry a `-rc.N` suffix, Docker Desktop appends
//! `-desktop.1`, and the supported minimums this crate ships are
//! two-component. So this is a deliberately small parser over
//! `major[.minor[.patch]]` with everything after the numbers ignored.

use std::cmp::Ordering;
use std::fmt;

/// A `major.minor.patch` version, with missing components read as `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major component.
    pub major: u32,
    /// Minor component, `0` when the source omitted it.
    pub minor: u32,
    /// Patch component, `0` when the source omitted it.
    pub patch: u32,
}

impl Version {
    /// A version from its three components.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse `raw`, tolerating a leading `v`, surrounding whitespace, a missing
    /// patch (or minor) component, and any `-rc.1` / `+build` suffix.
    ///
    /// Returns `None` when no leading numeric component is present.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let numeric = trimmed.strip_prefix('v').unwrap_or(trimmed);
        let head = numeric.split(['-', '+']).next()?;
        let mut parts = head.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = component(parts.next())?;
        let patch = component(parts.next())?;
        Some(Self::new(major, minor, patch))
    }

    /// Whether `self` is at or above `min`.
    #[must_use]
    pub fn satisfies(self, min: Self) -> bool {
        matches!(self.cmp(&min), Ordering::Greater | Ordering::Equal)
    }
}

/// An absent component reads as `0`; a present but non-numeric one fails the
/// whole parse rather than silently becoming `0`.
fn component(part: Option<&str>) -> Option<u32> {
    match part {
        None => Some(0),
        Some(p) => p.parse().ok(),
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
