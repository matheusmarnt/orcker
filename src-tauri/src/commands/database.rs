//! M6 — Database commands: auto-create testing DB, dump, restore, open CLI

use bollard::exec::{CreateExecOptions, StartExecResults};
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::adapters::docker::exec::{docker_exec_stream, CommandChunk};
use crate::core::error::AppError;
use crate::core::state::AppState;

/// Sanitise a project name into a valid PostgreSQL identifier fragment.
fn sanitise_db_name(project_name: &str) -> String {
    project_name.replace(['-', ' '], "_")
}

/// Find the global postgres container by Docker Compose project label.
///
/// Looks for a container with label `com.docker.compose.service=postgres`
/// belonging to the `orcker-global` compose project.
#[allow(deprecated)]
async fn find_global_postgres(docker: &bollard::Docker) -> Result<String, AppError> {
    use bollard::container::ListContainersOptions;

    let mut filters = std::collections::HashMap::new();
    filters.insert(
        "label".to_string(),
        vec!["com.docker.compose.service=postgres".to_string()],
    );

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: false,
            filters,
            ..Default::default()
        }))
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    containers
        .into_iter()
        .find(|c| {
            c.labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.project"))
                .map(|p| p == "orcker-global")
                .unwrap_or(false)
        })
        .and_then(|c| c.id)
        .ok_or_else(|| AppError::NotFound("Global postgres container not found".into()))
}

/// Execute a one-shot command in a container and return combined stdout+stderr output.
async fn exec_one_shot(
    docker: &bollard::Docker,
    container_id: &str,
    cmd: Vec<String>,
) -> Result<String, AppError> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(cmd),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    let mut output_text = String::new();

    if let StartExecResults::Attached { mut output, .. } =
        docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| AppError::DockerApi(e.to_string()))?
    {
        while let Some(Ok(chunk)) = output.next().await {
            output_text.push_str(&chunk.to_string());
        }
    }

    Ok(output_text)
}

/// Auto-create `{project_name}_testing` DB in the global PostgreSQL container.
///
/// Idempotent: "already exists" errors are silently ignored.
/// Called as a fire-and-forget from register_project and scaffold_project.
pub async fn create_testing_db_inner(
    docker: &bollard::Docker,
    project_name: &str,
) -> Result<(), AppError> {
    let db_name = format!("{}_testing", sanitise_db_name(project_name));
    let container_id = find_global_postgres(docker).await?;

    let sql = format!("CREATE DATABASE \"{}\"", db_name);
    let cmd = vec![
        "psql".to_string(),
        "-U".to_string(),
        "postgres".to_string(),
        "-c".to_string(),
        sql,
    ];

    let output = exec_one_shot(docker, &container_id, cmd).await?;

    // "already exists" is not an error — idempotent
    if output.contains("ERROR") && !output.contains("already exists") {
        return Err(AppError::Internal(format!(
            "psql error creating {}: {}",
            db_name, output
        )));
    }

    Ok(())
}

/// Tauri command wrapper for create_testing_db_inner.
#[tauri::command]
#[specta::specta]
pub async fn create_testing_db(
    project_name: String,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?;
    create_testing_db_inner(&docker.client, &project_name).await
}

/// Dump project database via pg_dump to a user-chosen file path via native save dialog.
#[tauri::command]
#[specta::specta]
pub async fn dump_db(
    project_name: String,
    app_state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri_plugin_dialog::DialogExt;

    let db_name = format!("{}_testing", sanitise_db_name(&project_name));

    // Open native save dialog
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("SQL", &["sql"])
        .set_file_name(format!("{}.sql", db_name))
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let Some(dest_path) = rx
        .await
        .map_err(|_| AppError::Internal("Dialog channel error".into()))?
    else {
        return Ok(()); // user cancelled
    };

    let dest_str = dest_path.to_string();

    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?
        .client
        .clone();
    drop(docker_guard);

    let container_id = find_global_postgres(&docker).await?;

    // pg_dump via blocking process (shells out docker exec)
    let dest = dest_str.clone();
    let name = db_name.clone();
    let cid = container_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("docker")
            .args(["exec", "-t", &cid, "pg_dump", "-U", "postgres", &name])
            .output()
            .map_err(|e| AppError::Internal(format!("docker exec pg_dump failed: {}", e)))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(AppError::Internal(format!("pg_dump failed: {}", stderr)));
        }

        std::fs::write(&dest, &out.stdout)
            .map_err(|e| AppError::Internal(format!("Cannot write dump file: {}", e)))?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking error: {}", e)))??;

    Ok(())
}

/// Restore project database from a user-chosen file path via native open dialog.
#[tauri::command]
#[specta::specta]
pub async fn restore_db(
    project_name: String,
    app_state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    use tauri_plugin_dialog::DialogExt;

    let db_name = format!("{}_testing", sanitise_db_name(&project_name));

    // Open native file picker
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("SQL", &["sql"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });

    let Some(src_path) = rx
        .await
        .map_err(|_| AppError::Internal("Dialog channel error".into()))?
    else {
        return Ok(()); // user cancelled
    };

    let src_str = src_path.to_string();

    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?
        .client
        .clone();
    drop(docker_guard);

    let container_id = find_global_postgres(&docker).await?;

    let src = src_str.clone();
    let name = db_name.clone();
    let cid = container_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let sql_content = std::fs::read(&src)
            .map_err(|e| AppError::Internal(format!("Cannot read dump file: {}", e)))?;

        let mut child = std::process::Command::new("docker")
            .args(["exec", "-i", &cid, "psql", "-U", "postgres", &name])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Internal(format!("docker exec psql failed: {}", e)))?;

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin
                .write_all(&sql_content)
                .map_err(|e| AppError::Internal(format!("Failed to write SQL: {}", e)))?;
        }

        let out = child
            .wait_with_output()
            .map_err(|e| AppError::Internal(format!("psql wait error: {}", e)))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(AppError::Internal(format!(
                "psql restore failed: {}",
                stderr
            )));
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking error: {}", e)))??;

    Ok(())
}

/// Open an interactive psql CLI session in the global postgres container.
///
/// Streams output via the existing docker_exec_stream adapter.
#[tauri::command]
#[specta::specta]
pub async fn open_db_cli(
    project_name: String,
    on_chunk: Channel<CommandChunk>,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db_name = format!("{}_testing", sanitise_db_name(&project_name));

    let docker_guard = app_state.docker.read().await;
    let docker = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::Internal("Docker not connected".into()))?
        .client
        .clone();
    drop(docker_guard);

    let container_id = find_global_postgres(&docker).await?;

    let cmd = vec![
        "psql".to_string(),
        "-U".to_string(),
        "postgres".to_string(),
        db_name,
    ];

    let token = CancellationToken::new();
    docker_exec_stream(&docker, &container_id, cmd, on_chunk, token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_create_testing_db_runs_correct_sql() {
        // Verify that sanitise_db_name produces valid PostgreSQL identifiers
        let name = sanitise_db_name("my-project");
        assert_eq!(name, "my_project");
        let name2 = sanitise_db_name("my project");
        assert_eq!(name2, "my_project");
        let db = format!("{}_testing", sanitise_db_name("my-project"));
        assert_eq!(db, "my_project_testing");
    }

    #[test]
    fn sanitise_handles_spaces_and_dashes() {
        assert_eq!(sanitise_db_name("hello world"), "hello_world");
        assert_eq!(sanitise_db_name("hello-world"), "hello_world");
        assert_eq!(sanitise_db_name("hello_world"), "hello_world");
    }
}
