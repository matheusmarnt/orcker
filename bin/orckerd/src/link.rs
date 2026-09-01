//! Planning for `orcker link`: register a directory as a container project.
//!
//! The decision is pure and lives here; the caller performs the I/O it names
//! (reading or writing the project's `orcker.yml`, saving the config). Port
//! probing is injected as [`orcker_core::PortProbe`] so the plan is testable
//! with a fake.

use std::path::{Path, PathBuf};

use orcker_config::{Config, ConfigError, OrckerYml};
use orcker_core::{allocate_port, ContainerProject, CoreError, PhpVersion, PortProbe};

/// What linking `root` would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkPlan {
    /// The project is not registered yet: add `project` to the config, and
    /// write `write_yml` to `<root>/orcker.yml` when it is `Some` (the file was
    /// absent).
    Link {
        /// The registry entry to persist.
        project: ContainerProject,
        /// The descriptor to create, or `None` when the project already ships
        /// one and it was read instead.
        write_yml: Option<OrckerYml>,
    },
    /// The project is already registered under the same name and root, on the
    /// same port. Nothing to do (R5 idempotence).
    AlreadyLinked {
        /// The existing entry.
        project: ContainerProject,
    },
}

/// Why a link could not be planned.
#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    /// The site name was invalid, or no loopback port was free.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// The project's `orcker.yml` was unreadable or invalid.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The requested name is already taken by a *different* project or by a
    /// site, so linking would shadow it.
    #[error("site {name:?} already exists")]
    NameTaken {
        /// The colliding name.
        name: String,
    },
    /// The requested port is already allocated to another project.
    #[error("port {port} is already allocated to project {owner:?}")]
    PortTaken {
        /// The requested port.
        port: u16,
        /// The project already holding it.
        owner: String,
    },
    /// The project directory has no usable name and none was given.
    #[error("cannot derive a site name from {}", root.display())]
    NoName {
        /// The directory that produced no name.
        root: PathBuf,
    },
}

/// Decides what `orcker link` should do for `root`.
///
/// `existing_yml` is the already-read `<root>/orcker.yml`, or `None` when the
/// file is absent. `requested_port` pins the allocation (the SPEC-0005 spike
/// flow); otherwise a port is allocated from the free range.
///
/// # Errors
///
/// See [`LinkError`].
pub fn plan_link(
    cfg: &Config,
    root: &Path,
    requested_name: Option<&str>,
    requested_port: Option<u16>,
    existing_yml: Option<&OrckerYml>,
    probe: &dyn PortProbe,
) -> Result<LinkPlan, LinkError> {
    let name =
        derive_name(root, requested_name, existing_yml).ok_or_else(|| LinkError::NoName {
            root: root.to_path_buf(),
        })?;

    if let Some(existing) = cfg.projects.iter().find(|p| p.name() == name) {
        if existing.root() == root {
            return Ok(LinkPlan::AlreadyLinked {
                project: existing.clone(),
            });
        }
        return Err(LinkError::NameTaken { name });
    }
    if cfg.linked.iter().any(|s| s.name() == name) || cfg.proxies.iter().any(|p| p.name() == name) {
        return Err(LinkError::NameTaken { name });
    }

    let port = match requested_port {
        Some(requested) => {
            if let Some(owner) = cfg.projects.iter().find(|p| p.port() == requested) {
                return Err(LinkError::PortTaken {
                    port: requested,
                    owner: owner.name().to_owned(),
                });
            }
            requested
        }
        None => allocate_port(&orcker_config::taken_ports(cfg), probe)?,
    };

    let write_yml = match existing_yml {
        Some(_) => None,
        None => Some(OrckerYml::new(
            &name,
            DEFAULT_PHP,
            DEFAULT_DB,
            DEFAULT_PRESET,
        )?),
    };

    Ok(LinkPlan::Link {
        project: ContainerProject::new(&name, root, port)?,
        write_yml,
    })
}

/// PHP version a freshly created `orcker.yml` declares.
///
/// R4 fixes all three of `php`, `db` and `preset` as literals for v1, so this
/// does **not** follow the daemon's configured default PHP: the descriptor is
/// the project's own record and Phase 1 (`orcker new --php`, FR-020) is where
/// the version becomes a choice.
const DEFAULT_PHP: PhpVersion = PhpVersion::new(8, 4);

/// Database engine a freshly created `orcker.yml` declares (R4).
const DEFAULT_DB: &str = "postgres";

/// Stack preset a freshly created `orcker.yml` declares (R4).
const DEFAULT_PRESET: &str = "reference";

/// The real [`PortProbe`]: the daemon's I/O edge for "is this loopback port
/// free?". A port is free when a listener can be bound and immediately dropped.
/// Only `127.0.0.1` is probed, matching where a project's stack publishes.
pub struct TcpPortProbe;

impl PortProbe for TcpPortProbe {
    fn is_free(&self, port: u16) -> bool {
        std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).is_ok()
    }
}

/// The site name a link would use: the explicit one, else the descriptor's,
/// else the directory's own name, normalised to a DNS label.
#[must_use]
pub fn derive_name(
    root: &Path,
    requested: Option<&str>,
    existing_yml: Option<&OrckerYml>,
) -> Option<String> {
    if let Some(requested) = requested {
        return orcker_core::normalize_site_name(requested);
    }
    if let Some(yml) = existing_yml {
        return orcker_core::normalize_site_name(&yml.site);
    }
    root.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(orcker_core::slugify_site_name)
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

    struct AllFree;

    impl PortProbe for AllFree {
        fn is_free(&self, _port: u16) -> bool {
            true
        }
    }

    #[test]
    fn idempotent_relink() {
        let mut cfg = Config::default();
        let root = Path::new("/srv/spike");

        let first = plan_link(&cfg, root, None, None, None, &AllFree).unwrap();
        let LinkPlan::Link { project, write_yml } = first else {
            panic!("a fresh directory must plan a link, got {first:?}");
        };
        assert_eq!(project.name(), "spike", "name derived from the directory");
        assert_eq!(project.port(), 20000, "first project takes the first port");
        let yml = write_yml.expect("an absent orcker.yml must be created");
        assert_eq!(yml.site, "spike");
        assert_eq!(
            yml.php, DEFAULT_PHP,
            "R4 fixes the created descriptor's php literal"
        );

        cfg.projects.push(project.clone());

        let second = plan_link(&cfg, root, None, None, Some(&yml), &AllFree).unwrap();
        assert_eq!(
            second,
            LinkPlan::AlreadyLinked {
                project: project.clone()
            },
            "relinking the same directory changes nothing"
        );
        assert_eq!(
            cfg.projects,
            vec![project],
            "the config is untouched by a relink"
        );
    }

    #[test]
    fn an_existing_descriptor_is_read_not_overwritten() {
        let cfg = Config::default();
        let yml = OrckerYml::new("blog", PhpVersion::new(8, 3), "mysql", "minimal").unwrap();

        let plan = plan_link(
            &cfg,
            Path::new("/srv/anything"),
            None,
            None,
            Some(&yml),
            &AllFree,
        )
        .unwrap();

        match plan {
            LinkPlan::Link { project, write_yml } => {
                assert_eq!(project.name(), "blog", "the descriptor names the site");
                assert!(
                    write_yml.is_none(),
                    "an existing orcker.yml is never rewritten"
                );
            }
            LinkPlan::AlreadyLinked { .. } => panic!("expected a link, not a relink"),
        }
    }

    #[test]
    fn an_explicit_port_is_honoured_and_a_taken_one_is_rejected() {
        let mut cfg = Config::default();

        let plan = plan_link(
            &cfg,
            Path::new("/srv/spike"),
            Some("spike"),
            Some(20007),
            None,
            &AllFree,
        )
        .unwrap();
        let LinkPlan::Link { project, .. } = plan else {
            panic!("expected a link");
        };
        assert_eq!(project.port(), 20007);

        cfg.projects.push(project);

        match plan_link(
            &cfg,
            Path::new("/srv/other"),
            Some("other"),
            Some(20007),
            None,
            &AllFree,
        ) {
            Err(LinkError::PortTaken { port, owner }) => {
                assert_eq!(port, 20007);
                assert_eq!(owner, "spike");
            }
            other => panic!("expected PortTaken, got {other:?}"),
        }
    }

    #[test]
    fn a_name_already_used_by_a_different_root_is_rejected() {
        let mut cfg = Config::default();
        cfg.projects
            .push(ContainerProject::new("spike", "/srv/spike", 20000).unwrap());

        match plan_link(
            &cfg,
            Path::new("/srv/elsewhere"),
            Some("spike"),
            None,
            None,
            &AllFree,
        ) {
            Err(LinkError::NameTaken { name }) => assert_eq!(name, "spike"),
            other => panic!("expected NameTaken, got {other:?}"),
        }
    }
}
