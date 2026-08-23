//! Which prebuilt artifact this machine needs, and whether an archive member
//! is safe to unpack.
//!
//! Orcker downloads prebuilt binaries (the updater, `cloudflared`, the Node and
//! Bun toolchains) and unpacks them into per-user directories. Both halves of
//! that job are the same two questions every time: what target am I, and is
//! this tar member trying to escape the staging directory.
//!
//! [`target_from`] and [`is_safe_member`] are pure and table-tested;
//! [`current_target`] is the one-line wrapper that reads the compile-time
//! `std::env::consts` pair. It lives outside `pure/` for that reason alone.

use thiserror::Error;

/// The running platform is one orcker publishes no artifacts for.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unsupported target: {detail}")]
pub struct UnsupportedTarget {
    /// Which half failed, and what it read.
    pub detail: String,
}

/// Target operating system for a prebuilt artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// Linux.
    Linux,
    /// macOS.
    Macos,
}

impl Os {
    /// The token used in artifact filenames.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Os::Linux => "linux",
            Os::Macos => "macos",
        }
    }
}

/// Target CPU architecture for a prebuilt artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// 64-bit x86.
    X86_64,
    /// 64-bit ARM.
    Aarch64,
}

impl Arch {
    /// The token used in artifact filenames.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        }
    }
}

/// Classify a `(os, arch)` pair in `std::env::consts` spelling, erroring on
/// anything orcker publishes no artifacts for (Windows, 32-bit, and friends).
///
/// # Errors
///
/// Returns [`UnsupportedTarget`] naming the half that was not recognised.
pub fn target_from(os: &str, arch: &str) -> Result<(Os, Arch), UnsupportedTarget> {
    let os = match os {
        "linux" => Os::Linux,
        "macos" => Os::Macos,
        other => {
            return Err(UnsupportedTarget {
                detail: format!("no prebuilt artifacts for OS {other:?}"),
            })
        }
    };
    let arch = match arch {
        "x86_64" => Arch::X86_64,
        "aarch64" => Arch::Aarch64,
        other => {
            return Err(UnsupportedTarget {
                detail: format!("no prebuilt artifacts for architecture {other:?}"),
            })
        }
    };
    Ok((os, arch))
}

/// Detect the running platform. Call this **before** any download.
///
/// # Errors
///
/// Returns [`UnsupportedTarget`] when this build targets a platform orcker
/// publishes no artifacts for.
pub fn current_target() -> Result<(Os, Arch), UnsupportedTarget> {
    target_from(std::env::consts::OS, std::env::consts::ARCH)
}

/// Zip-slip guard: a tar member name is safe to trust only if it is relative
/// and contains no `..`, root, or prefix components. An empty name is unsafe.
#[must_use]
pub fn is_safe_member(name: &str) -> bool {
    use std::path::Component;
    !name.is_empty()
        && std::path::Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
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

    #[test]
    fn target_from_maps_the_supported_pairs() {
        let cases = [
            ("linux", "x86_64", Os::Linux, Arch::X86_64),
            ("linux", "aarch64", Os::Linux, Arch::Aarch64),
            ("macos", "x86_64", Os::Macos, Arch::X86_64),
            ("macos", "aarch64", Os::Macos, Arch::Aarch64),
        ];
        for (os, arch, want_os, want_arch) in cases {
            assert_eq!(target_from(os, arch).unwrap(), (want_os, want_arch));
        }
    }

    #[test]
    fn target_from_rejects_and_names_the_failing_half() {
        let err = target_from("windows", "x86_64").unwrap_err();
        assert!(err.detail.contains("windows"), "{}", err.detail);
        let err = target_from("linux", "i686").unwrap_err();
        assert!(err.detail.contains("i686"), "{}", err.detail);
    }

    #[test]
    fn current_target_is_supported_on_every_ci_platform() {
        assert!(current_target().is_ok());
    }

    #[test]
    fn is_safe_member_rejects_traversal_and_absolute_paths() {
        for good in ["bin/php", "./bin/php", "a"] {
            assert!(is_safe_member(good), "{good}");
        }
        for bad in ["", "/etc/passwd", "../escape", "a/../../b", "a/.."] {
            assert!(!is_safe_member(bad), "{bad}");
        }
    }

    #[test]
    fn tokens_match_the_artifact_filename_spelling() {
        assert_eq!(Os::Linux.as_str(), "linux");
        assert_eq!(Os::Macos.as_str(), "macos");
        assert_eq!(Arch::X86_64.as_str(), "x86_64");
        assert_eq!(Arch::Aarch64.as_str(), "aarch64");
    }
}
