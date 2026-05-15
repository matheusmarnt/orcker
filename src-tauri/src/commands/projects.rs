use crate::core::compose::ComposeDriver;
use crate::core::error::AppError;
use crate::core::projects::{ProjectConfig, ProjectStatus, ProjectStatusEvent, ProjectsState};
use crate::core::state::AppState;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

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
pub async fn open_project_folder(
    project_id: String,
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let path = {
        let projects = state.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| AppError::NotFound("Project not found".into()))?
    };
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Internal(e.to_string()))
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

            let status = child
                .wait()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
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
        [
            "create-project",
            "laravel/laravel",
            &project_name,
            "--prefer-dist"
        ],
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
                [
                    "install",
                    "-D",
                    "tailwindcss",
                    "alpinejs",
                    "@alpinejs/collapse"
                ],
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub is_comment: bool,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct EnvFile {
    pub entries: Vec<EnvEntry>,
}

pub fn parse_env(content: &str) -> Vec<EnvEntry> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed.starts_with('#') {
                return Some(EnvEntry {
                    key: trimmed.to_string(),
                    value: String::new(),
                    is_comment: true,
                });
            }
            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim().to_string();
                let value = trimmed[pos + 1..].to_string();
                Some(EnvEntry {
                    key,
                    value,
                    is_comment: false,
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct EnvReadResult {
    pub env: EnvFile,
    pub example: EnvFile,
}

#[tauri::command]
#[specta::specta]
pub async fn read_env_file(
    project_id: String,
    state: State<'_, ProjectsState>,
) -> Result<EnvReadResult, AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::Internal(format!("Project {} not found", project_id)))?;

    let env_content = std::fs::read_to_string(format!("{}/.env", project.path)).unwrap_or_default();
    let example_content =
        std::fs::read_to_string(format!("{}/.env.example", project.path)).unwrap_or_default();

    Ok(EnvReadResult {
        env: EnvFile {
            entries: parse_env(&env_content),
        },
        example: EnvFile {
            entries: parse_env(&example_content),
        },
    })
}

#[tauri::command]
#[specta::specta]
pub async fn save_env_file(
    project_id: String,
    entries: Vec<EnvEntry>,
    state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::Internal(format!("Project {} not found", project_id)))?;

    let content: String = entries
        .iter()
        .map(|e| {
            if e.is_comment {
                format!("{}\n", e.key)
            } else {
                format!("{}={}\n", e.key, e.value)
            }
        })
        .collect();

    std::fs::write(format!("{}/.env", project.path), content)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_vite_auto(
    project_id: String,
    vite_auto: bool,
    state: State<'_, ProjectsState>,
) -> Result<ProjectConfig, AppError> {
    let mut projects = state.projects.write().await;
    let project = projects
        .iter_mut()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::Internal(format!("Project {} not found", project_id)))?;
    project.vite_auto = vite_auto;
    Ok(project.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn generate_xdebug_config(
    project_id: String,
    state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::Internal(format!("Project {} not found", project_id)))?;

    let vscode_dir = format!("{}/.vscode", project.path);
    let idea_dir = format!("{}/.idea", project.path);
    std::fs::create_dir_all(&vscode_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create .vscode: {}", e)))?;
    std::fs::create_dir_all(&idea_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create .idea: {}", e)))?;

    let launch_json = r#"{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Listen for Xdebug",
      "type": "php",
      "request": "launch",
      "port": 9003,
      "pathMappings": {
        "/var/www/html": "${workspaceFolder}"
      },
      "log": false
    }
  ]
}"#;

    let phpstorm_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="PhpProjectSharedConfiguration" php_language_level="8.3" />
  <component name="PhpDebugGeneral">
    <option name="listenerPort" value="9003" />
  </component>
</project>"#;

    std::fs::write(format!("{}/.vscode/launch.json", project.path), launch_json)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    std::fs::write(format!("{}/.idea/php.xml", project.path), phpstorm_xml)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

// ─── php.ini types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct IniEntry {
    pub key: String,
    pub value: String,
    pub is_comment: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct IniSection {
    pub name: String,
    pub entries: Vec<IniEntry>,
}

pub fn parse_php_ini(content: &str) -> Vec<IniSection> {
    let mut sections: Vec<IniSection> = vec![IniSection {
        name: "Global".into(),
        entries: vec![],
    }];
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = trimmed[1..trimmed.len() - 1].to_string();
            sections.push(IniSection {
                name,
                entries: vec![],
            });
        } else if trimmed.starts_with(';') {
            sections.last_mut().unwrap().entries.push(IniEntry {
                key: trimmed.to_string(),
                value: String::new(),
                is_comment: true,
            });
        } else if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim().to_string();
            let value = trimmed[pos + 1..].trim().to_string();
            sections.last_mut().unwrap().entries.push(IniEntry {
                key,
                value,
                is_comment: false,
            });
        }
    }
    // Drop empty sections (e.g. Global with no entries)
    sections.retain(|s| !s.entries.is_empty());
    sections
}

#[tauri::command]
#[specta::specta]
pub async fn read_php_ini(
    project_id: String,
    state: State<'_, ProjectsState>,
) -> Result<Vec<IniSection>, AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("Project {} not found", project_id)))?;

    // Try docker/php.ini first, then root php.ini
    let docker_path = format!("{}/docker/php.ini", project.path);
    let root_path = format!("{}/php.ini", project.path);
    let path = if std::fs::metadata(&docker_path).is_ok() {
        docker_path
    } else {
        root_path
    };

    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Internal(format!("Cannot read {}: {}", path, e)))?;
    Ok(parse_php_ini(&content))
}

#[tauri::command]
#[specta::specta]
pub async fn save_php_ini(
    project_id: String,
    sections: Vec<IniSection>,
    state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("Project {} not found", project_id)))?;

    let content: String = sections
        .iter()
        .map(|section| {
            let header = format!("[{}]\n", section.name);
            let entries: String = section
                .entries
                .iter()
                .map(|e| {
                    if e.is_comment {
                        format!("{}\n", e.key)
                    } else {
                        format!("{} = {}\n", e.key, e.value)
                    }
                })
                .collect();
            format!("{}{}", header, entries)
        })
        .collect();

    let docker_path = format!("{}/docker/php.ini", project.path);
    let root_path = format!("{}/php.ini", project.path);
    let path = if std::fs::metadata(&docker_path).is_ok() {
        docker_path
    } else {
        root_path
    };

    std::fs::write(path, content).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

// ─── Supervisor types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct SupervisorWorker {
    pub name: String,
    pub status: String,
}

#[tauri::command]
#[specta::specta]
pub async fn list_supervisor_workers(
    supervisor_container: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<SupervisorWorker>, AppError> {
    use futures_util::StreamExt;

    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?;

    let exec = docker
        .client
        .create_exec(
            &supervisor_container,
            bollard::exec::CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec!["supervisorctl".to_string(), "status".to_string()]),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    let mut workers = Vec::new();
    if let bollard::exec::StartExecResults::Attached { mut output, .. } = docker
        .client
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?
    {
        while let Some(Ok(chunk)) = output.next().await {
            let text = chunk.to_string();
            for line in text.lines() {
                // supervisorctl status output: "worker_name   RUNNING   pid 123, uptime 0:00:01"
                let parts: Vec<&str> = line
                    .splitn(3, char::is_whitespace)
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() >= 2 {
                    workers.push(SupervisorWorker {
                        name: parts[0].to_string(),
                        status: parts[1].to_string(),
                    });
                }
            }
        }
    }
    Ok(workers)
}

#[tauri::command]
#[specta::specta]
pub async fn restart_supervisor_worker(
    supervisor_container: String,
    worker_name: String,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    use futures_util::StreamExt;

    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?;

    let exec = docker
        .client
        .create_exec(
            &supervisor_container,
            bollard::exec::CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec![
                    "supervisorctl".to_string(),
                    "restart".to_string(),
                    worker_name,
                ]),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    if let bollard::exec::StartExecResults::Attached { mut output, .. } = docker
        .client
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?
    {
        while output.next().await.is_some() {} // drain output
    }
    Ok(())
}

// ── Compose lifecycle ──────────────────────────────────────────────────────

/// Detect whether the project uses Laravel Sail.
fn is_sail(project_path: &str) -> bool {
    std::path::Path::new(project_path)
        .join("vendor/bin/sail")
        .exists()
}

/// Build the compose command (program + args) for Sail / Plugin / Legacy.
fn build_compose_cmd(
    project_path: &str,
    driver: &ComposeDriver,
    args: &[&str],
) -> Result<(String, Vec<String>), AppError> {
    let (program, cmd_args): (String, Vec<String>) = if is_sail(project_path) {
        let sail = std::path::Path::new(project_path)
            .join("vendor/bin/sail")
            .to_string_lossy()
            .to_string();
        (sail, args.iter().map(|s| s.to_string()).collect())
    } else {
        match driver {
            ComposeDriver::Plugin => {
                let mut v = vec!["compose".to_string()];
                v.extend(args.iter().map(|s| s.to_string()));
                ("docker".to_string(), v)
            }
            ComposeDriver::Legacy => (
                "docker-compose".to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ),
            ComposeDriver::None => {
                return Err(AppError::Internal(
                    "No Docker Compose driver found on this system.".into(),
                ))
            }
        }
    };
    Ok((program, cmd_args))
}

/// Run compose command, return (combined_output, exit_success) without failing on non-zero exit.
async fn exec_compose_output(
    project_path: &str,
    driver: &ComposeDriver,
    args: &[&str],
) -> Result<(String, bool), AppError> {
    use tokio::process::Command;
    let (program, cmd_args) = build_compose_cmd(project_path, driver, args)?;
    let out = Command::new(&program)
        .args(&cmd_args)
        .current_dir(project_path)
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to run '{}': {}", program, e)))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
    .trim()
    .to_string();
    Ok((combined, out.status.success()))
}

/// Run a docker compose command (up/down/etc.) in the project directory.
/// Auto-selects Sail > Compose Plugin > Compose Legacy.
async fn run_compose_command(
    project_path: &str,
    driver: &ComposeDriver,
    args: &[&str],
) -> Result<String, AppError> {
    let (combined, success) = exec_compose_output(project_path, driver, args).await?;
    if !success {
        // Return last non-empty line — compose puts the root cause last
        let summary = combined
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| "Command failed".to_string());
        return Err(AppError::Internal(summary));
    }
    Ok(combined)
}

/// Start project containers — strict 2-stage recovery.
///
/// Stage 1 — full `up -d` + readiness poll (30 s): waits until ALL containers
///            are running+healthy. Returns Ok only when all pass.
/// Stage 2 — Docker restart loop: restarts exited/unhealthy containers, re-runs
///            `up -d`, polls again (25 s). Returns Ok only when all pass.
///
/// On success, spawns a 5 s health monitor that emits `project://status` events.
/// On failure, returns the names of the services that did not reach healthy state.
#[tauri::command]
#[specta::specta]
#[allow(deprecated)]
pub async fn start_project(
    project_id: String,
    state: State<'_, ProjectsState>,
    app_state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, AppError> {
    let (path, driver) = {
        let projects = state.projects.read().await;
        let p = projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| AppError::NotFound("Project not found".into()))?;
        (p.path.clone(), state.compose_driver.clone())
    };

    // Stage 1: full compose up (all services, dependency ordering intact)
    exec_compose_output(&path, &driver, &["up", "-d"]).await?;

    let docker_opt = {
        let g = app_state.docker.read().await;
        g.as_ref().map(|a| a.client.clone())
    };

    let Some(ref docker) = docker_opt else {
        return Ok("started".into());
    };

    let stage1 = poll_containers_ready(docker, &path, 30).await;

    if matches!(stage1, ReadinessResult::AllRunning) {
        spawn_project_monitor(app, docker.clone(), project_id, path, &state).await;
        return Ok("started".into());
    }

    // Stage 2: restart exited/unhealthy containers, re-run compose.
    let to_restart = match stage1 {
        ReadinessResult::SomeExited(ids) => ids,
        _ => list_exited_project_containers(docker, &path).await,
    };

    for id in &to_restart {
        let _ = docker
            .restart_container(id, None::<bollard::container::RestartContainerOptions>)
            .await;
    }
    if !to_restart.is_empty() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    exec_compose_output(&path, &driver, &["up", "-d"]).await?;

    if let ReadinessResult::AllRunning = poll_containers_ready(docker, &path, 25).await {
        spawn_project_monitor(app, docker.clone(), project_id, path, &state).await;
        return Ok("started".into());
    }

    // All stages exhausted — report which services failed.
    let failed = list_failed_service_names(docker, &path).await;
    let detail = if failed.is_empty() {
        "one or more services failed to reach healthy state".to_string()
    } else {
        format!("services failed to start: {}", failed.join(", "))
    };
    Err(AppError::Internal(detail))
}

/// Outcome of a container readiness poll.
enum ReadinessResult {
    /// Every project container is running and has passed its healthcheck (if any).
    AllRunning,
    /// One or more containers exited or became unhealthy. Carries their Docker IDs.
    SomeExited(Vec<String>),
    /// Timeout elapsed before containers stabilised.
    Timeout,
}

/// Returns (id, state, status) for every container in a compose project (all: true).
#[allow(deprecated)]
async fn list_all_project_containers(
    docker: &bollard::Docker,
    project_path: &str,
) -> Vec<(String, String, String)> {
    use bollard::container::ListContainersOptions;
    let mut filters = std::collections::HashMap::new();
    filters.insert(
        "label".to_string(),
        vec![format!(
            "com.docker.compose.project.working_dir={}",
            project_path
        )],
    );
    docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        }))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            let id = c.id.unwrap_or_default();
            let state = c
                .state
                .map(|s| format!("{:?}", s).to_lowercase())
                .unwrap_or_default();
            let status = c.status.unwrap_or_default();
            (id, state, status)
        })
        .collect()
}

/// Polls container states until all are running+healthy or a failure/timeout occurs.
///
/// Docker status strings observed in the wild:
///   "Up 3 minutes"                  — running, no healthcheck
///   "Up 3 minutes (healthy)"        — healthcheck passing ✓
///   "Up 3 minutes (unhealthy)"      — healthcheck failing → treated as failure
///   "Up 2 seconds (health: starting)" — healthcheck not yet concluded → wait
///   "Exited (1) 5 seconds ago"      — state == "exited"             → failure
async fn poll_containers_ready(
    docker: &bollard::Docker,
    project_path: &str,
    timeout_secs: u64,
) -> ReadinessResult {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let containers = list_all_project_containers(docker, project_path).await;

        if !containers.is_empty() {
            let bad_ids: Vec<String> = containers
                .iter()
                .filter(|(_, state, status)| {
                    state == "exited" || state == "dead" || status.contains("(unhealthy)")
                })
                .map(|(id, _, _)| id.clone())
                .collect();

            if !bad_ids.is_empty() {
                return ReadinessResult::SomeExited(bad_ids);
            }

            let all_ready = containers.iter().all(|(_, state, status)| {
                state == "running" && !status.contains("(health: starting)")
            });

            if all_ready {
                return ReadinessResult::AllRunning;
            }
        }

        if std::time::Instant::now() >= deadline {
            return ReadinessResult::Timeout;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
    }
}

/// List Docker IDs of all exited containers belonging to a compose project.
#[allow(deprecated)]
async fn list_exited_project_containers(
    docker: &bollard::Docker,
    project_path: &str,
) -> Vec<String> {
    use bollard::container::ListContainersOptions;
    let mut filters = std::collections::HashMap::new();
    filters.insert(
        "label".to_string(),
        vec![format!(
            "com.docker.compose.project.working_dir={}",
            project_path
        )],
    );
    filters.insert("status".to_string(), vec!["exited".to_string()]);
    docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        }))
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| c.id)
        .collect()
}

/// Stop project containers. Aborts health monitor, runs compose down, emits final Stopped event.
#[tauri::command]
#[specta::specta]
pub async fn stop_project(
    project_id: String,
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<String, AppError> {
    let (path, driver) = {
        let projects = state.projects.read().await;
        let p = projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| AppError::NotFound("Project not found".into()))?;
        (p.path.clone(), state.compose_driver.clone())
    };

    {
        let mut monitors = state.monitors.write().await;
        if let Some(handle) = monitors.remove(&project_id) {
            handle.abort();
        }
    }

    let result = run_compose_command(&path, &driver, &["down"]).await;

    let _ = app.emit(
        "project://status",
        ProjectStatusEvent {
            project_id,
            status: ProjectStatus::Stopped,
        },
    );

    result
}

/// Service names of containers that are exited, dead, or (unhealthy).
#[allow(deprecated)]
async fn list_failed_service_names(docker: &bollard::Docker, project_path: &str) -> Vec<String> {
    use bollard::container::ListContainersOptions;
    let mut filters = std::collections::HashMap::new();
    filters.insert(
        "label".to_string(),
        vec![format!(
            "com.docker.compose.project.working_dir={}",
            project_path
        )],
    );
    let mut names: Vec<String> = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        }))
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            let state = c
                .state
                .as_ref()
                .map(|s| format!("{:?}", s).to_lowercase())
                .unwrap_or_default();
            let status = c.status.as_deref().unwrap_or("");
            state == "exited" || state == "dead" || status.contains("(unhealthy)")
        })
        .filter_map(|c| {
            c.labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.service"))
                .cloned()
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Derive `ProjectStatus` from live Docker state for a project.
async fn compute_project_status(docker: &bollard::Docker, project_path: &str) -> ProjectStatus {
    let containers = list_all_project_containers(docker, project_path).await;
    if containers.is_empty() {
        return ProjectStatus::Stopped;
    }
    if containers
        .iter()
        .any(|(_, _, status)| status.contains("(unhealthy)"))
    {
        return ProjectStatus::Unhealthy;
    }
    let running = containers
        .iter()
        .filter(|(_, state, _)| state == "running")
        .count();
    match (running, containers.len()) {
        (r, t) if r == t => ProjectStatus::Running,
        (r, _) if r > 0 => ProjectStatus::PartiallyRunning,
        _ => ProjectStatus::Stopped,
    }
}

/// Spawn a 5 s health monitor that emits `project://status` events until stopped.
async fn spawn_project_monitor(
    app: AppHandle,
    docker: bollard::Docker,
    project_id: String,
    project_path: String,
    state: &ProjectsState,
) {
    let handle = tauri::async_runtime::spawn({
        let app = app.clone();
        let pid = project_id.clone();
        async move {
            let mut last: Option<ProjectStatus> = None;
            loop {
                let s = compute_project_status(&docker, &project_path).await;
                if last.as_ref() != Some(&s) {
                    let _ = app.emit(
                        "project://status",
                        ProjectStatusEvent {
                            project_id: pid.clone(),
                            status: s.clone(),
                        },
                    );
                    last = Some(s.clone());
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    let mut monitors = state.monitors.write().await;
    if let Some(old) = monitors.insert(project_id, handle) {
        old.abort();
    }
}

/// Return current `ProjectStatus` for a project by querying Docker.
#[tauri::command]
#[specta::specta]
pub async fn get_project_status(
    project_id: String,
    state: State<'_, ProjectsState>,
    app_state: State<'_, AppState>,
) -> Result<ProjectStatus, AppError> {
    let path = {
        let projects = state.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| AppError::NotFound("Project not found".into()))?
    };
    let docker_opt = {
        let g = app_state.docker.read().await;
        g.as_ref().map(|a| a.client.clone())
    };
    let Some(docker) = docker_opt else {
        return Ok(ProjectStatus::Stopped);
    };
    Ok(compute_project_status(&docker, &path).await)
}

#[cfg(test)]
mod ini_tests {
    use super::parse_php_ini;

    #[test]
    fn test_parse_php_ini_two_sections() {
        let content = "[PHP]\nmemory_limit=128M\n; comment\n[OPcache]\nopcache.enable=1";
        let sections = parse_php_ini(content);
        assert_eq!(sections.len(), 2);
    }

    #[test]
    fn test_parse_php_ini_groups_entries_correctly() {
        let content = "[PHP]\nmemory_limit=128M\n[OPcache]\nopcache.enable=1";
        let sections = parse_php_ini(content);
        assert_eq!(sections[0].name, "PHP");
        assert_eq!(sections[0].entries[0].key, "memory_limit");
    }
}

#[cfg(test)]
mod xdebug_tests {
    #[test]
    fn test_launch_json_contains_port_9003() {
        let launch_json = r#"{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Listen for Xdebug",
      "type": "php",
      "request": "launch",
      "port": 9003,
      "pathMappings": {
        "/var/www/html": "${workspaceFolder}"
      },
      "log": false
    }
  ]
}"#;
        assert!(launch_json.contains("9003"));
        assert!(!launch_json.contains("9000"));
    }

    #[test]
    fn test_launch_json_contains_workspace_folder() {
        let content = r#""/var/www/html": "${workspaceFolder}""#;
        assert!(content.contains("${workspaceFolder}"));
    }

    #[test]
    fn test_generate_xdebug_writes_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let launch_json = r#"{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Listen for Xdebug",
      "type": "php",
      "request": "launch",
      "port": 9003,
      "pathMappings": {
        "/var/www/html": "${workspaceFolder}"
      },
      "log": false
    }
  ]
}"#;
        let phpstorm_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="PhpProjectSharedConfiguration" php_language_level="8.3" />
  <component name="PhpDebugGeneral">
    <option name="listenerPort" value="9003" />
  </component>
</project>"#;

        std::fs::create_dir_all(format!("{}/.vscode", path)).unwrap();
        std::fs::create_dir_all(format!("{}/.idea", path)).unwrap();
        std::fs::write(format!("{}/.vscode/launch.json", path), launch_json).unwrap();
        std::fs::write(format!("{}/.idea/php.xml", path), phpstorm_xml).unwrap();

        let written = std::fs::read_to_string(format!("{}/.vscode/launch.json", path)).unwrap();
        assert!(written.contains("9003"));
        assert!(!written.contains("9000"));
        assert!(written.contains("${workspaceFolder}"));

        let xml = std::fs::read_to_string(format!("{}/.idea/php.xml", path)).unwrap();
        assert!(xml.contains("9003"));
    }
}

#[cfg(test)]
mod env_tests {
    use super::parse_env;

    #[test]
    fn test_parse_env_basic() {
        let entries = parse_env("KEY=value\n# comment\nEMPTY=");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, "KEY");
        assert_eq!(entries[0].value, "value");
        assert!(!entries[0].is_comment);
        assert!(entries[1].is_comment);
        assert_eq!(entries[2].key, "EMPTY");
        assert_eq!(entries[2].value, "");
    }

    #[test]
    fn test_parse_env_splits_on_first_equals() {
        let entries = parse_env("KEY=value=with=equals");
        assert_eq!(entries[0].value, "value=with=equals");
    }
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
