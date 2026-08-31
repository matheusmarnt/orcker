//! PHP versions Orcker builds reference images for.

use std::fmt;
use std::str::FromStr;

use crate::error::{PhpVersionErrorReason, StackError};

/// A PHP minor version with a published Orcker reference image.
///
/// Deliberately a closed enum rather than a numeric pair: the renderer must not
/// be able to emit a `PHP_VERSION` build arg no image exists for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhpVersion {
    /// PHP 8.1.
    V81,
    /// PHP 8.2.
    V82,
    /// PHP 8.3.
    V83,
    /// PHP 8.4.
    V84,
    /// PHP 8.5.
    V85,
}

impl PhpVersion {
    /// The `major.minor` form used in build args and on the command line.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V81 => "8.1",
            Self::V82 => "8.2",
            Self::V83 => "8.3",
            Self::V84 => "8.4",
            Self::V85 => "8.5",
        }
    }
}

impl fmt::Display for PhpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PhpVersion {
    type Err = StackError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse(input).map_err(|reason| StackError::InvalidPhpVersion {
            input: input.to_owned(),
            reason,
        })
    }
}

fn parse(input: &str) -> Result<PhpVersion, PhpVersionErrorReason> {
    if input.is_empty() {
        return Err(PhpVersionErrorReason::Empty);
    }
    let (major, minor) = input
        .split_once('.')
        .ok_or(PhpVersionErrorReason::Malformed)?;
    let major: u16 = major
        .parse()
        .map_err(|_| PhpVersionErrorReason::Malformed)?;
    let minor: u16 = minor
        .parse()
        .map_err(|_| PhpVersionErrorReason::Malformed)?;

    match (major, minor) {
        (8, 1) => Ok(PhpVersion::V81),
        (8, 2) => Ok(PhpVersion::V82),
        (8, 3) => Ok(PhpVersion::V83),
        (8, 4) => Ok(PhpVersion::V84),
        (8, 5) => Ok(PhpVersion::V85),
        _ => Err(PhpVersionErrorReason::Unsupported),
    }
}
