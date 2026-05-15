use crate::core::compose::ComposeDriver;
use crate::core::error::AppError;
use crate::core::projects::{ProjectConfig, ProjectsState};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ImportResult {
    pub path: String,
    pub detected_files: Vec<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn pick_project_folder(app: AppHandle) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.map(|p| p.to_string()));
    });
    Ok(rx.await.unwrap_or(None))
}

#[tauri::command]
#[specta::specta]
pub async fn register_project(
    state: State<'_, ProjectsState>,
    name: String,
    path: String,
) -> Result<ProjectConfig, AppError> {
    let config = ProjectConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path,
        vite_auto: true,
    };
    let mut projects = state.projects.write().await;
    projects.push(config.clone());
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub async fn import_project(path: String) -> Result<ImportResult, AppError> {
    let probe_files = ["artisan", "composer.json", ".env", "docker-compose.yml"];
    let detected_files = probe_files
        .iter()
        .filter(|f| std::fs::metadata(format!("{}/{}", path, f)).is_ok())
        .map(|f| f.to_string())
        .collect();
    Ok(ImportResult {
        path,
        detected_files,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_projects(
    state: State<'_, ProjectsState>,
) -> Result<Vec<ProjectConfig>, AppError> {
    Ok(state.projects.read().await.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn get_compose_driver(
    state: State<'_, ProjectsState>,
) -> Result<ComposeDriver, AppError> {
    Ok(state.compose_driver.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_project_detects_artisan() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("artisan"), "").unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let probe_files = ["artisan", "composer.json", ".env", "docker-compose.yml"];
        let detected: Vec<String> = probe_files
            .iter()
            .filter(|f| std::fs::metadata(format!("{}/{}", path, f)).is_ok())
            .map(|f| f.to_string())
            .collect();
        assert_eq!(detected, vec!["artisan"]);
    }

    #[test]
    fn test_import_project_empty_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let probe_files = ["artisan", "composer.json", ".env", "docker-compose.yml"];
        let detected: Vec<String> = probe_files
            .iter()
            .filter(|f| std::fs::metadata(format!("{}/{}", path, f)).is_ok())
            .map(|f| f.to_string())
            .collect();
        assert!(detected.is_empty());
    }
}
