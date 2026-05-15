use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// ServiceId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ServiceId {
    Redis,
    Postgres,
    Mysql,
    Mailpit,
    Minio,
    Soketi,
    Meilisearch,
}

impl ServiceId {
    pub fn all() -> [ServiceId; 7] {
        [
            ServiceId::Redis,
            ServiceId::Postgres,
            ServiceId::Mysql,
            ServiceId::Mailpit,
            ServiceId::Minio,
            ServiceId::Soketi,
            ServiceId::Meilisearch,
        ]
    }

    pub fn container_name(&self) -> &'static str {
        match self {
            ServiceId::Redis => "orcker-redis",
            ServiceId::Postgres => "orcker-postgres",
            ServiceId::Mysql => "orcker-mysql",
            ServiceId::Mailpit => "orcker-mailpit",
            ServiceId::Minio => "orcker-minio",
            ServiceId::Soketi => "orcker-soketi",
            ServiceId::Meilisearch => "orcker-meilisearch",
        }
    }

    /// Internal Docker port string (e.g. "6379/tcp").
    pub fn internal_port(&self) -> &'static str {
        match self {
            ServiceId::Redis => "6379/tcp",
            ServiceId::Postgres => "5432/tcp",
            ServiceId::Mysql => "3306/tcp",
            ServiceId::Mailpit => "8025/tcp",
            ServiceId::Minio => "9000/tcp",
            ServiceId::Soketi => "6001/tcp",
            ServiceId::Meilisearch => "7700/tcp",
        }
    }

    /// Environment variables required to start this service.
    pub fn default_env_vars(&self) -> Vec<String> {
        match self {
            ServiceId::Postgres => vec![
                "POSTGRES_PASSWORD=postgres".to_string(),
                "POSTGRES_USER=postgres".to_string(),
                "POSTGRES_DB=postgres".to_string(),
            ],
            ServiceId::Mysql => vec![
                "MYSQL_ROOT_PASSWORD=root".to_string(),
                "MYSQL_DATABASE=orcker".to_string(),
                "MYSQL_USER=orcker".to_string(),
                "MYSQL_PASSWORD=orcker".to_string(),
            ],
            ServiceId::Minio => vec![
                "MINIO_ROOT_USER=orcker".to_string(),
                "MINIO_ROOT_PASSWORD=orckerpassword".to_string(),
            ],
            ServiceId::Soketi => vec![
                "SOKETI_DEFAULT_APP_ID=orcker".to_string(),
                "SOKETI_DEFAULT_APP_KEY=orckerkey".to_string(),
                "SOKETI_DEFAULT_APP_SECRET=orckersecret".to_string(),
            ],
            ServiceId::Meilisearch => vec!["MEILI_MASTER_KEY=orckermaster".to_string()],
            ServiceId::Redis | ServiceId::Mailpit => vec![],
        }
    }

    /// Optional command override (e.g. MinIO needs server args).
    pub fn command(&self) -> Option<Vec<&'static str>> {
        match self {
            ServiceId::Minio => Some(vec!["server", "/data", "--console-address", ":9001"]),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// ServiceConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ServiceConfig {
    pub image_tag: String,
    pub port: u16,
}

impl ServiceConfig {
    pub fn default_for(id: ServiceId) -> Self {
        match id {
            ServiceId::Redis => ServiceConfig {
                image_tag: "redis:7-alpine".to_string(),
                port: 6379,
            },
            ServiceId::Postgres => ServiceConfig {
                image_tag: "postgres:16-alpine".to_string(),
                port: 5432,
            },
            ServiceId::Mysql => ServiceConfig {
                image_tag: "mysql:8-oracle".to_string(),
                port: 3306,
            },
            ServiceId::Mailpit => ServiceConfig {
                image_tag: "axllent/mailpit:latest".to_string(),
                port: 8025,
            },
            ServiceId::Minio => ServiceConfig {
                image_tag: "minio/minio:latest".to_string(),
                port: 9000,
            },
            ServiceId::Soketi => ServiceConfig {
                image_tag: "quay.io/soketi/soketi:latest".to_string(),
                port: 6001,
            },
            ServiceId::Meilisearch => ServiceConfig {
                image_tag: "getmeili/meilisearch:latest".to_string(),
                port: 7700,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// ServiceStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase", tag = "kind", content = "message")]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Unhealthy,
    Error(String),
}

impl ServiceStatus {
    /// Returns `true` when the service is in a transitional state (Starting or Stopping).
    /// Used by the UI to disable toggle buttons during transitions.
    pub fn is_transitioning(&self) -> bool {
        matches!(self, ServiceStatus::Starting | ServiceStatus::Stopping)
    }
}

// ---------------------------------------------------------------------------
// GlobalStackState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GlobalStackState {
    pub statuses: Arc<RwLock<HashMap<ServiceId, ServiceStatus>>>,
    pub configs: Arc<RwLock<HashMap<ServiceId, ServiceConfig>>>,
}

impl GlobalStackState {
    /// Initialises all six services with Stopped status and default configs.
    pub fn new() -> Self {
        let mut statuses = HashMap::new();
        let mut configs = HashMap::new();

        for id in ServiceId::all() {
            statuses.insert(id, ServiceStatus::Stopped);
            configs.insert(id, ServiceConfig::default_for(id));
        }

        GlobalStackState {
            statuses: Arc::new(RwLock::new(statuses)),
            configs: Arc::new(RwLock::new(configs)),
        }
    }
}

impl Default for GlobalStackState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_defaults() {
        let redis = ServiceConfig::default_for(ServiceId::Redis);
        assert_eq!(redis.port, 6379);
        assert_eq!(redis.image_tag, "redis:7-alpine");

        let postgres = ServiceConfig::default_for(ServiceId::Postgres);
        assert_eq!(postgres.port, 5432);
        assert_eq!(postgres.image_tag, "postgres:16-alpine");

        let mailpit = ServiceConfig::default_for(ServiceId::Mailpit);
        assert_eq!(mailpit.port, 8025);
        assert_eq!(mailpit.image_tag, "axllent/mailpit:latest");
    }

    #[test]
    fn test_status_is_transitioning() {
        assert!(ServiceStatus::Starting.is_transitioning());
        assert!(ServiceStatus::Stopping.is_transitioning());
        assert!(!ServiceStatus::Stopped.is_transitioning());
        assert!(!ServiceStatus::Running.is_transitioning());
        assert!(!ServiceStatus::Unhealthy.is_transitioning());
        assert!(!ServiceStatus::Error("oops".to_string()).is_transitioning());
    }

    #[test]
    fn test_all_services() {
        assert_eq!(ServiceId::all().len(), 7);
    }

    #[test]
    fn test_container_names() {
        assert_eq!(ServiceId::Redis.container_name(), "orcker-redis");
        assert_eq!(ServiceId::Postgres.container_name(), "orcker-postgres");
        assert_eq!(ServiceId::Mailpit.container_name(), "orcker-mailpit");
        assert_eq!(ServiceId::Minio.container_name(), "orcker-minio");
        assert_eq!(ServiceId::Soketi.container_name(), "orcker-soketi");
        assert_eq!(
            ServiceId::Meilisearch.container_name(),
            "orcker-meilisearch"
        );
    }

    #[test]
    fn test_new_service_defaults() {
        let minio = ServiceConfig::default_for(ServiceId::Minio);
        assert_eq!(minio.port, 9000);
        assert_eq!(minio.image_tag, "minio/minio:latest");

        let soketi = ServiceConfig::default_for(ServiceId::Soketi);
        assert_eq!(soketi.port, 6001);
        assert_eq!(soketi.image_tag, "quay.io/soketi/soketi:latest");

        let meili = ServiceConfig::default_for(ServiceId::Meilisearch);
        assert_eq!(meili.port, 7700);
        assert_eq!(meili.image_tag, "getmeili/meilisearch:latest");
    }
}
