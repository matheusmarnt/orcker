use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

pub type ProjectId = String; // UUID4 string at creation time

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ProjectConfig {
    pub id: ProjectId,
    pub name: String,
    pub path: String,
    pub vite_auto: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectStatus {
    Running,
    PartiallyRunning,
    Unhealthy,
    Stopped,
}

#[derive(Clone, serde::Serialize, specta::Type)]
pub struct ProjectStatusEvent {
    pub project_id: String,
    pub status: ProjectStatus,
}

pub struct ProjectsState {
    pub projects: tokio::sync::RwLock<Vec<ProjectConfig>>,
    pub compose_driver: crate::core::compose::ComposeDriver,
    pub cancel_tokens: tokio::sync::RwLock<HashMap<ProjectId, CancellationToken>>,
    pub monitors: tokio::sync::RwLock<HashMap<ProjectId, tauri::async_runtime::JoinHandle<()>>>,
}

impl ProjectsState {
    pub fn new(compose_driver: crate::core::compose::ComposeDriver) -> Self {
        Self {
            projects: tokio::sync::RwLock::new(Vec::new()),
            compose_driver,
            cancel_tokens: tokio::sync::RwLock::new(HashMap::new()),
            monitors: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_serializes() {
        let cfg = ProjectConfig {
            id: "test-id".into(),
            name: "my-app".into(),
            path: "/home/user/my-app".into(),
            vite_auto: true,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("\"vite_auto\":true"));
        assert!(json.contains("\"path\":\"/home/user/my-app\""));
    }

    #[test]
    fn test_project_status_running_serializes() {
        let status = ProjectStatus::Running;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["kind"], "running");
    }

    #[test]
    fn test_project_status_partially_running_serializes() {
        let status = ProjectStatus::PartiallyRunning;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["kind"], "partially_running");
    }
}
