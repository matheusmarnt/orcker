//! Persisted loopback-port allocations for container projects.
//!
//! The allocation *policy* is pure and lives in `orcker_core::ports`; this
//! module supplies it with the set of ports the persisted config already hands
//! out, so a port stays reserved for its project across daemon restarts
//! (FR-013).

use std::collections::BTreeSet;

use crate::schema::Config;

/// Every loopback port the config currently reserves for a container project.
#[must_use]
pub fn taken_ports(config: &Config) -> BTreeSet<u16> {
    config
        .projects
        .iter()
        .map(orcker_core::ContainerProject::port)
        .collect()
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
    use orcker_core::{allocate_port, ContainerProject, PortProbe};

    struct AllFree;

    impl PortProbe for AllFree {
        fn is_free(&self, _port: u16) -> bool {
            true
        }
    }

    fn config_with_projects() -> Config {
        Config {
            projects: vec![
                ContainerProject::new("spike", "/srv/spike", 20000).unwrap(),
                ContainerProject::new("shop", "/srv/shop", 20001).unwrap(),
            ],
            ..Config::default()
        }
    }

    #[test]
    fn allocation_roundtrip() {
        let config = config_with_projects();
        let toml = config.to_toml().unwrap();
        let back = Config::from_toml(&toml).unwrap();

        assert_eq!(
            back.projects, config.projects,
            "projects survive a config round-trip, ports included"
        );
        assert_eq!(
            taken_ports(&back),
            [20000, 20001].into_iter().collect::<BTreeSet<u16>>(),
            "the persisted allocations are what the allocator must avoid"
        );
        assert_eq!(
            allocate_port(&taken_ports(&back), &AllFree),
            Ok(20002),
            "a restarted daemon hands the next project a fresh port"
        );
    }

    #[test]
    fn unlinking_frees_the_port() {
        let mut config = config_with_projects();
        config.projects.retain(|p| p.name() != "spike");

        assert_eq!(
            taken_ports(&config),
            [20001].into_iter().collect::<BTreeSet<u16>>()
        );
        assert_eq!(
            allocate_port(&taken_ports(&config), &AllFree),
            Ok(20000),
            "the freed port is the next one handed out"
        );
    }
}
