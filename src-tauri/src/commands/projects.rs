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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ScaffoldTemplate {
    Tall,
    InertiaVue3,
    InertiaReact,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ScaffoldChunk {
    pub text: String,
    pub done: bool,
    pub error: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn scaffold_project(
    template: ScaffoldTemplate,
    project_name: String,
    parent_dir: String,
    on_chunk: tauri::ipc::Channel<ScaffoldChunk>,
) -> Result<(), AppError> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    // Stream both stdout and stderr from a child process, then wait for exit.
    // Returns Err if the exit code is non-zero, sending a done+error chunk first.
    macro_rules! run_step {
        ($program:expr, $args:expr, $dir:expr) => {{
            let mut child = Command::new($program)
                .args($args)
                .current_dir($dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| AppError::Internal(format!("Failed to spawn {}: {}", $program, e)))?;

            // Stream stdout
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = on_chunk.send(ScaffoldChunk {
                        text: line,
                        done: false,
                        error: false,
                    });
                }
            }
            // Stream stderr
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = on_chunk.send(ScaffoldChunk {
                        text: line,
                        done: false,
                        error: false,
                    });
                }
            }

            let status = child.wait().await.map_err(|e| AppError::Internal(e.to_string()))?;
            if !status.success() {
                let _ = on_chunk.send(ScaffoldChunk {
                    text: format!("{} step failed", $program),
                    done: true,
                    error: true,
                });
                return Err(AppError::Internal(format!(
                    "{} failed with status {:?}",
                    $program,
                    status.code()
                )));
            }
        }};
    }

    let project_dir = format!("{}/{}", parent_dir, project_name);

    // Step 1: base Laravel install — all templates start here
    let _ = on_chunk.send(ScaffoldChunk {
        text: "Installing Laravel...".into(),
        done: false,
        error: false,
    });
    run_step!(
        "composer",
        ["create-project", "laravel/laravel", &project_name, "--prefer-dist"],
        &parent_dir
    );

    // Step 2: template-specific additions
    match template {
        ScaffoldTemplate::Tall => {
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Livewire...".into(),
                done: false,
                error: false,
            });
            run_step!("composer", ["require", "livewire/livewire"], &project_dir);
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Alpine.js + Tailwind...".into(),
                done: false,
                error: false,
            });
            run_step!(
                "npm",
                ["install", "-D", "tailwindcss", "alpinejs", "@alpinejs/collapse"],
                &project_dir
            );
        }
        ScaffoldTemplate::InertiaVue3 => {
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Inertia Laravel adapter...".into(),
                done: false,
                error: false,
            });
            run_step!(
                "composer",
                ["require", "inertiajs/inertia-laravel"],
                &project_dir
            );
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Inertia Vue 3...".into(),
                done: false,
                error: false,
            });
            run_step!("npm", ["install", "@inertiajs/vue3", "vue"], &project_dir);
        }
        ScaffoldTemplate::InertiaReact => {
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Inertia Laravel adapter...".into(),
                done: false,
                error: false,
            });
            run_step!(
                "composer",
                ["require", "inertiajs/inertia-laravel"],
                &project_dir
            );
            let _ = on_chunk.send(ScaffoldChunk {
                text: "Installing Inertia React...".into(),
                done: false,
                error: false,
            });
            run_step!(
                "npm",
                ["install", "@inertiajs/react", "react", "react-dom"],
                &project_dir
            );
        }
    }

    let _ = on_chunk.send(ScaffoldChunk {
        text: "Scaffold complete!".into(),
        done: true,
        error: false,
    });
    Ok(())
}

#[cfg(test)]
mod scaffold_tests {
    use super::*;

    #[test]
    fn test_scaffold_template_tall_serializes() {
        let json = serde_json::to_value(ScaffoldTemplate::Tall).unwrap();
        assert_eq!(json, "Tall");
    }

    #[test]
    fn test_scaffold_chunk_done_serializes() {
        let chunk = ScaffoldChunk {
            text: "done".into(),
            done: true,
            error: false,
        };
        let json = serde_json::to_value(&chunk).unwrap();
        assert_eq!(json["done"], true);
        assert_eq!(json["error"], false);
    }
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
