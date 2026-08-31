//! Pure typed stack model and `docker-compose.yml` renderer for Orcker.
//!
//! This crate is **pure**: no I/O, no clock or env reads, no async, no internal
//! `orcker-*` dependencies. It takes a validated [`StackConfig`] and returns the
//! rendered compose file as a `String`; writing it to disk belongs to the I/O
//! edge.

#![forbid(unsafe_code)]

mod compose;
mod config;
mod error;
mod php;
mod site_name;

pub use compose::render_compose;
pub use config::{DbEngine, Ports, Preset, StackConfig};
pub use error::{
    PhpVersionErrorReason, PortErrorReason, PortField, SiteNameErrorReason, StackError,
};
pub use php::PhpVersion;
pub use site_name::SiteName;
