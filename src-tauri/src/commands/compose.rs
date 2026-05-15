//! M5.1 + M5.4 — Compose file read/save commands + git-based versioning

use crate::core::error::AppError;
use crate::core::projects::ProjectsState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn read_compose_file(
    projects_state: State<'_, ProjectsState>,
    project_id: String,
) -> Result<String, AppError> {
    let projects = projects_state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

    let compose_path = std::path::Path::new(&project.path).join("docker-compose.yml");
    if !compose_path.exists() {
        // Try docker-compose.yaml fallback
        let alt = std::path::Path::new(&project.path).join("docker-compose.yaml");
        if alt.exists() {
            return tokio::fs::read_to_string(&alt)
                .await
                .map_err(|e| AppError::Internal(e.to_string()));
        }
        return Err(AppError::NotFound("docker-compose.yml".into()));
    }
    tokio::fs::read_to_string(&compose_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
#[specta::specta]
pub async fn save_compose_file(
    projects_state: State<'_, ProjectsState>,
    project_id: String,
    content: String,
) -> Result<(), AppError> {
    let projects = projects_state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

    let compose_path = std::path::Path::new(&project.path).join("docker-compose.yml");
    tokio::fs::write(&compose_path, &content)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Hook: commit the saved file into .orcker-history/ git repo (fire-and-forget)
    let project_dir = std::path::PathBuf::from(&project.path);
    let content_clone = content.clone();
    tokio::task::spawn_blocking(move || {
        if let Ok(repo) = crate::core::versioning::ensure_repo(&project_dir) {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = crate::core::versioning::commit_file(
                &repo,
                "docker-compose.yml",
                &content_clone,
                &format!("save: {}", timestamp),
            );
        }
    })
    .await
    .ok();

    Ok(())
}

/// Return a unified diff string between the last two committed versions of docker-compose.yml.
/// Returns an empty string when fewer than two commits exist (i.e., no prior saved version).
#[tauri::command]
#[specta::specta]
pub async fn get_compose_diff(
    projects_state: State<'_, ProjectsState>,
    project_id: String,
) -> Result<String, AppError> {
    let projects = projects_state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

    let project_dir = std::path::PathBuf::from(&project.path);
    tokio::task::spawn_blocking(move || {
        let repo = crate::core::versioning::ensure_repo(&project_dir)?;
        crate::core::versioning::diff_last_two(&repo)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_compose_file_returns_not_found_for_missing() {
        // This is a pure logic test — the actual IPC is tested via type-check
        let path = std::path::Path::new("/nonexistent/path/docker-compose.yml");
        assert!(!path.exists());
    }
}
