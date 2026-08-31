//! The typed stack model the renderer consumes.

use crate::error::{PortErrorReason, PortField, StackError};
use crate::php::PhpVersion;
use crate::site_name::SiteName;

/// The database engine a project's stack runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DbEngine {
    /// Postgres, the reference engine.
    Postgres,
}

/// How much of the reference topology a project's stack renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Preset {
    /// Full production-parity topology (app + nginx + database).
    Reference,
}

/// The pair of loopback ports a stack publishes.
///
/// Validated together because the failure that matters (two services fighting
/// over one port) is a property of the pair, not of either value alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ports {
    http_loopback: u16,
    vite: u16,
}

impl Ports {
    /// Validates and constructs the pair.
    ///
    /// Port 0 is rejected because it asks the kernel for an ephemeral port,
    /// which a stack persisted in a repository cannot rely on; an equal pair is
    /// rejected because the two services would fight over one loopback socket.
    pub fn new(http_loopback: u16, vite: u16) -> Result<Self, StackError> {
        if http_loopback == 0 {
            return Err(StackError::InvalidPort {
                field: PortField::HttpLoopback,
                value: http_loopback,
                reason: PortErrorReason::Zero,
            });
        }
        if vite == 0 {
            return Err(StackError::InvalidPort {
                field: PortField::Vite,
                value: vite,
                reason: PortErrorReason::Zero,
            });
        }
        if vite == http_loopback {
            return Err(StackError::InvalidPort {
                field: PortField::Vite,
                value: vite,
                reason: PortErrorReason::DuplicatesHttpLoopback,
            });
        }
        Ok(Self {
            http_loopback,
            vite,
        })
    }

    /// The loopback port nginx publishes for HTTP.
    #[must_use]
    pub fn http_loopback(self) -> u16 {
        self.http_loopback
    }

    /// The loopback port the app container publishes for the Vite dev server.
    #[must_use]
    pub fn vite(self) -> u16 {
        self.vite
    }
}

/// A fully validated stack model: everything the renderer needs, nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackConfig {
    site: SiteName,
    php: PhpVersion,
    db: DbEngine,
    preset: Preset,
    ports: Ports,
    uid: u32,
    gid: u32,
}

impl StackConfig {
    /// Assembles a stack model from already-validated parts.
    pub fn new(
        site: SiteName,
        php: PhpVersion,
        db: DbEngine,
        preset: Preset,
        ports: Ports,
        uid: u32,
        gid: u32,
    ) -> Result<Self, StackError> {
        Ok(Self {
            site,
            php,
            db,
            preset,
            ports,
            uid,
            gid,
        })
    }

    /// The project's site name.
    #[must_use]
    pub fn site(&self) -> &SiteName {
        &self.site
    }

    /// The PHP version the app image is built for.
    #[must_use]
    pub fn php(&self) -> PhpVersion {
        self.php
    }

    /// The database engine.
    #[must_use]
    pub fn db(&self) -> DbEngine {
        self.db
    }

    /// The rendered preset.
    #[must_use]
    pub fn preset(&self) -> Preset {
        self.preset
    }

    /// The published loopback ports.
    #[must_use]
    pub fn ports(&self) -> Ports {
        self.ports
    }

    /// Host user id applied to the app image build.
    #[must_use]
    pub fn uid(&self) -> u32 {
        self.uid
    }

    /// Host group id applied to the app image build.
    #[must_use]
    pub fn gid(&self) -> u32 {
        self.gid
    }
}
