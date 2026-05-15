use crate::core::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct TemplateEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub compose_url: String,
}

#[derive(Debug, Deserialize)]
struct TemplateManifest {
    templates: Vec<TemplateEntry>,
}

/// Fetch template manifest from GitHub raw URL.
/// Done in Rust to avoid CSP issues — frontend never contacts external HTTPS.
#[tauri::command]
#[specta::specta]
pub async fn fetch_template_manifest() -> Result<Vec<TemplateEntry>, AppError> {
    let url = "https://raw.githubusercontent.com/matheusmarnt/orcker-templates/main/manifest.json";
    let resp = reqwest::get(url)
        .await
        .map_err(|e| AppError::Internal(format!("manifest fetch failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "manifest returned HTTP {}",
            resp.status()
        )));
    }
    let manifest: TemplateManifest = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("manifest parse failed: {e}")))?;
    Ok(manifest.templates)
}

/// Download a template's docker-compose.yml to a user-chosen directory.
#[tauri::command]
#[specta::specta]
pub async fn install_template(
    app: tauri::AppHandle,
    compose_url: String,
) -> Result<String, AppError> {
    use tauri_plugin_dialog::DialogExt;

    // Open save dialog for destination path
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<std::path::PathBuf>>();
    app.dialog()
        .file()
        .set_title("Save docker-compose.yml as")
        .add_filter("YAML", &["yml", "yaml"])
        .save_file(move |path| {
            let _ = tx.send(path.map(|p| std::path::PathBuf::from(p.to_string())));
        });
    let Some(dest) = rx
        .await
        .map_err(|_| AppError::Internal("dialog cancelled".into()))?
    else {
        return Err(AppError::Internal("no path selected".into()));
    };

    // Download compose file content
    let content = reqwest::get(&compose_url)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tokio::fs::write(&dest, content)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(dest.display().to_string())
}
