//! `orcker.yml` v1: the project-owned, versioned description of a container
//! project.
//!
//! The file lives in the project directory and is committed with the repo, so
//! it is the source of truth for the project's own settings (`orcker.toml`
//! only persists the registry entry and the allocated port). The reader is a
//! hand-written strict reader for the flat v1 schema rather than a YAML crate:
//! the workspace deliberately carries no YAML dependency, and v1 is five
//! scalar keys.
//!
//! Forward compatibility (FR-024): unknown top-level keys, and any nested block
//! or list under them, are ignored rather than rejected, so a file written by a
//! newer Orcker still loads here.

use std::collections::BTreeMap;

use orcker_core::PhpVersion;

use crate::error::{ConfigError, OrckerYmlErrorReason};

/// File name of a project's Orcker descriptor.
pub const FILE_NAME: &str = "orcker.yml";

/// The only schema version this build reads and writes.
pub const SCHEMA_VERSION: u32 = 1;

/// Database engines a project may declare.
pub const KNOWN_DB_ENGINES: &[&str] = &["postgres", "mysql"];

/// Stack presets a project may declare.
pub const KNOWN_PRESETS: &[&str] = &["reference", "minimal"];

/// A parsed `orcker.yml` v1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrckerYml {
    /// Schema version of the file. Always [`SCHEMA_VERSION`] for v1.
    pub schema_version: u32,
    /// The site name the project is served under.
    pub site: String,
    /// PHP version the stack runs.
    pub php: PhpVersion,
    /// Database engine, one of [`KNOWN_DB_ENGINES`].
    pub db: String,
    /// Stack preset, one of [`KNOWN_PRESETS`].
    pub preset: String,
}

impl OrckerYml {
    /// Builds a v1 descriptor with validated values.
    ///
    /// # Errors
    ///
    /// [`ConfigError::OrckerYml`] when `db` or `preset` is not a known value,
    /// or `site` is not a DNS label.
    pub fn new(site: &str, php: PhpVersion, db: &str, preset: &str) -> Result<Self, ConfigError> {
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            site: validated_site(site)?,
            php,
            db: one_of("db", db, KNOWN_DB_ENGINES)?,
            preset: one_of("preset", preset, KNOWN_PRESETS)?,
        })
    }

    /// Reads a v1 descriptor, ignoring unknown top-level keys and any nested
    /// content under them.
    ///
    /// # Errors
    ///
    /// [`ConfigError::OrckerYml`] for a malformed line, a duplicate or missing
    /// required key, an unsupported `schema_version`, or an invalid value.
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        let mut found: BTreeMap<String, String> = BTreeMap::new();
        for (index, raw) in input.lines().enumerate() {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
                continue;
            }
            if raw.starts_with([' ', '\t']) {
                continue;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                return Err(malformed(index + 1));
            };
            let key = key.trim();
            if !KNOWN_KEYS.contains(&key) {
                continue;
            }
            if found
                .insert(key.to_owned(), unquote(value).to_owned())
                .is_some()
            {
                return Err(ConfigError::OrckerYml {
                    reason: OrckerYmlErrorReason::DuplicateKey {
                        key: key.to_owned(),
                    },
                });
            }
        }

        let version_raw = required(&found, "schema_version")?;
        let schema_version = version_raw
            .parse::<u32>()
            .map_err(|_| invalid("schema_version", version_raw))?;
        if schema_version != SCHEMA_VERSION {
            return Err(ConfigError::OrckerYml {
                reason: OrckerYmlErrorReason::UnsupportedSchemaVersion {
                    found: schema_version,
                },
            });
        }

        let php_raw = required(&found, "php")?;
        Ok(Self {
            schema_version,
            site: validated_site(required(&found, "site")?)?,
            php: php_raw
                .parse::<PhpVersion>()
                .map_err(|_| invalid("php", php_raw))?,
            db: one_of("db", required(&found, "db")?, KNOWN_DB_ENGINES)?,
            preset: one_of("preset", required(&found, "preset")?, KNOWN_PRESETS)?,
        })
    }

    /// Renders the canonical v1 form, ending with a newline.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "schema_version: {}\nsite: {}\nphp: \"{}\"\ndb: {}\npreset: {}\n",
            self.schema_version, self.site, self.php, self.db, self.preset
        )
    }
}

/// Top-level keys v1 reads. Anything else is ignored (forward-compat).
const KNOWN_KEYS: &[&str] = &["schema_version", "site", "php", "db", "preset"];

fn malformed(line: usize) -> ConfigError {
    ConfigError::OrckerYml {
        reason: OrckerYmlErrorReason::Malformed { line },
    }
}

fn invalid(key: &'static str, value: &str) -> ConfigError {
    ConfigError::OrckerYml {
        reason: OrckerYmlErrorReason::InvalidValue {
            key,
            value: value.to_owned(),
        },
    }
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value)
}

fn required<'a>(
    found: &'a BTreeMap<String, String>,
    key: &'static str,
) -> Result<&'a str, ConfigError> {
    found
        .get(key)
        .map(String::as_str)
        .ok_or(ConfigError::OrckerYml {
            reason: OrckerYmlErrorReason::MissingKey { key },
        })
}

fn one_of(key: &'static str, value: &str, allowed: &[&str]) -> Result<String, ConfigError> {
    if allowed.contains(&value) {
        Ok(value.to_owned())
    } else {
        Err(invalid(key, value))
    }
}

fn validated_site(site: &str) -> Result<String, ConfigError> {
    orcker_core::normalize_site_name(site).ok_or_else(|| invalid("site", site))
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

    fn php(v: &str) -> PhpVersion {
        v.parse().unwrap()
    }

    const CANONICAL: &str = "\
schema_version: 1
site: spike
php: \"8.4\"
db: postgres
preset: reference
";

    #[test]
    fn v1_roundtrip_and_forward_compat() {
        let parsed = OrckerYml::parse(CANONICAL).unwrap();
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
        assert_eq!(parsed.site, "spike");
        assert_eq!(parsed.php, php("8.4"));
        assert_eq!(parsed.db, "postgres");
        assert_eq!(parsed.preset, "reference");

        assert_eq!(parsed.render(), CANONICAL, "render is canonical");
        assert_eq!(
            OrckerYml::parse(&parsed.render()).unwrap(),
            parsed,
            "render then parse round-trips"
        );

        let from_the_future = "\
# written by a newer orcker
schema_version: 1
site: spike
php: \"8.4\"
db: postgres
preset: reference
services:
  - redis
  - meilisearch
supervisor:
  horizon: true
telemetry: off
";
        assert_eq!(
            OrckerYml::parse(from_the_future).unwrap(),
            parsed,
            "unknown top-level keys and their nested blocks are ignored"
        );
    }

    #[test]
    fn invalid_values_are_rejected() {
        let cases: &[(&str, &str, OrckerYmlErrorReason)] = &[
            (
                "unsupported schema version",
                "schema_version: 2\nsite: a\nphp: \"8.4\"\ndb: postgres\npreset: reference\n",
                OrckerYmlErrorReason::UnsupportedSchemaVersion { found: 2 },
            ),
            (
                "unknown database engine",
                "schema_version: 1\nsite: a\nphp: \"8.4\"\ndb: sqlite\npreset: reference\n",
                OrckerYmlErrorReason::InvalidValue {
                    key: "db",
                    value: "sqlite".to_owned(),
                },
            ),
            (
                "unknown preset",
                "schema_version: 1\nsite: a\nphp: \"8.4\"\ndb: postgres\npreset: bespoke\n",
                OrckerYmlErrorReason::InvalidValue {
                    key: "preset",
                    value: "bespoke".to_owned(),
                },
            ),
            (
                "missing required key",
                "schema_version: 1\nphp: \"8.4\"\ndb: postgres\npreset: reference\n",
                OrckerYmlErrorReason::MissingKey { key: "site" },
            ),
            (
                "duplicate key",
                "schema_version: 1\nsite: a\nsite: b\nphp: \"8.4\"\ndb: postgres\npreset: reference\n",
                OrckerYmlErrorReason::DuplicateKey {
                    key: "site".to_owned(),
                },
            ),
            (
                "malformed line",
                "schema_version: 1\nsite\nphp: \"8.4\"\ndb: postgres\npreset: reference\n",
                OrckerYmlErrorReason::Malformed { line: 2 },
            ),
        ];

        for (label, input, expected) in cases {
            match OrckerYml::parse(input) {
                Err(ConfigError::OrckerYml { reason }) => assert_eq!(&reason, expected, "{label}"),
                other => panic!("{label}: expected OrckerYml error, got {other:?}"),
            }
        }
    }

    #[test]
    fn new_validates_and_renders() {
        let yml = OrckerYml::new("spike", php("8.4"), "postgres", "reference").unwrap();
        assert_eq!(yml.render(), CANONICAL);
        assert!(OrckerYml::new("spike", php("8.4"), "sqlite", "reference").is_err());
    }
}
