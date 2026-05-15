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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "kind", content = "message")]
pub enum ProjectStatus {
    Running,
    Stopped,
    Error(String),
}

pub struct ProjectsState {
    pub projects: tokio::sync::RwLock<Vec<ProjectConfig>>,
    pub compose_driver: crate::core::compose::ComposeDriver,
    pub cancel_tokens: tokio::sync::RwLock<HashMap<ProjectId, CancellationToken>>,
}

impl ProjectsState {
    pub fn new(compose_driver: crate::core::compose::ComposeDriver) -> Self {
        Self {
            projects: tokio::sync::RwLock::new(Vec::new()),
            compose_driver,
            cancel_tokens: tokio::sync::RwLock::new(HashMap::new()),
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
    fn test_project_status_error_serializes_with_tag() {
        let status = ProjectStatus::Error("container crashed".into());
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["kind"], "Error");
        assert_eq!(json["message"], "container crashed");
    }
}
