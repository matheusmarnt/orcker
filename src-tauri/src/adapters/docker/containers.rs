#[allow(deprecated)]
use bollard::container::ListContainersOptions;
#[allow(deprecated)]
use bollard::system::EventsOptions;
use futures_util::StreamExt;
use std::collections::HashMap;
use tauri::Emitter;

/// Preferred app service names for Laravel projects (Sail, plain Compose, etc.)
const APP_SERVICES: &[&str] = &["laravel.test", "app", "web", "php", "application"];

/// Find the running app container for a project by querying Docker Compose labels.
/// Uses the project folder name as the compose project name (default Compose v2 behavior).
/// Returns `None` if no container found or Docker query fails.
#[allow(deprecated)]
pub async fn find_app_container(docker: &bollard::Docker, project_path: &str) -> Option<String> {
    let project_name = std::path::Path::new(project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if project_name.is_empty() {
        return None;
    }

    let mut filters: HashMap<String, Vec<String>> = HashMap::new();
    filters.insert(
        "label".to_string(),
        vec![format!("com.docker.compose.project={}", project_name)],
    );

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: false,
            filters,
            ..Default::default()
        }))
        .await
        .ok()?;

    // Preferred services first (Sail = laravel.test)
    for pref in APP_SERVICES {
        for c in &containers {
            let service = c
                .labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.service"))
                .map(|s| s.as_str())
                .unwrap_or("");
            if service == *pref {
                if let Some(name) = c.names.as_ref().and_then(|n| n.first()) {
                    return Some(name.trim_start_matches('/').to_string());
                }
            }
        }
    }

    // No preferred app service found — don't fall back to non-app containers
    // (mysql/redis/mailpit don't have php; exec-ing into them produces misleading errors)
    None
}

#[allow(deprecated)]
pub async fn subscribe_events(docker: bollard::Docker, app_handle: tauri::AppHandle) {
    let mut stream = docker.events(Some(EventsOptions::<String> {
        filters: std::collections::HashMap::from([(
            "type".to_string(),
            vec!["container".to_string()],
        )]),
        ..Default::default()
    }));

    while let Some(event) = stream.next().await {
        match event {
            Ok(evt) => {
                app_handle.emit("docker://container-event", &evt).ok();
            }
            Err(e) => {
                tracing::warn!("Docker event stream error: {e}");
                break;
            }
        }
    }
}
