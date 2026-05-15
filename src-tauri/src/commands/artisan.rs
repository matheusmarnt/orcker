use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::adapters::docker::exec::{docker_exec_stream, CommandChunk};
use crate::core::artisan::{artisan_commands, ArtisanCommand, CommandKind};
use crate::core::error::AppError;
use crate::core::projects::ProjectsState;
use crate::core::state::AppState;

/// Return the full artisan command catalog.
#[tauri::command]
#[specta::specta]
pub async fn list_artisan_commands() -> Result<Vec<ArtisanCommand>, AppError> {
    Ok(artisan_commands())
}

/// Stream artisan/shell output to the frontend via Channel<CommandChunk>.
///
/// Stores a CancellationToken in ProjectsState keyed by project_id so that
/// cancel_artisan_command can stop the stream at any time.
#[tauri::command]
#[specta::specta]
pub async fn run_artisan_command(
    command_id: String,
    project_id: String,
    container_name: String,
    on_chunk: Channel<CommandChunk>,
    app_state: State<'_, AppState>,
    projects_state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    // Resolve command from catalog
    let catalog = artisan_commands();
    let cmd_def = catalog
        .iter()
        .find(|c| c.id == command_id)
        .ok_or_else(|| AppError::Internal(format!("Command '{}' not in catalog", command_id)))?;

    // Build shell command vector
    let cmd_vec: Vec<String> = match &cmd_def.kind {
        CommandKind::Artisan(args) => {
            let mut v = vec!["php".into(), "artisan".into()];
            v.extend(args.split_whitespace().map(String::from));
            v
        }
        CommandKind::ShellInContainer(raw) => {
            vec!["sh".into(), "-c".into(), raw.clone()]
        }
    };

    // Create and store cancellation token keyed by project_id
    let token = CancellationToken::new();
    {
        let mut tokens = projects_state.cancel_tokens.write().await;
        tokens.insert(project_id.clone(), token.clone());
    }

    // Acquire Docker client
    let docker_guard = app_state.docker.read().await;
    let adapter = docker_guard
        .as_ref()
        .ok_or_else(|| AppError::DockerUnavailable("Docker not connected".into()))?;
    let docker = adapter.client.clone();
    drop(docker_guard);

    // Stream exec output
    let result = docker_exec_stream(&docker, &container_name, cmd_vec, on_chunk, token).await;

    // Remove token after completion or cancellation
    {
        let mut tokens = projects_state.cancel_tokens.write().await;
        tokens.remove(&project_id);
    }

    result
}

/// Cancel a running artisan command for the given project.
#[tauri::command]
#[specta::specta]
pub async fn cancel_artisan_command(
    project_id: String,
    projects_state: State<'_, ProjectsState>,
) -> Result<(), AppError> {
    let tokens = projects_state.cancel_tokens.read().await;
    if let Some(token) = tokens.get(&project_id) {
        token.cancel();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artisan_cmd_vec_built_correctly() {
        // Verify Artisan variant builds [php, artisan, ...args]
        let kind = CommandKind::Artisan("migrate:fresh --seed".into());
        let cmd_vec: Vec<String> = match &kind {
            CommandKind::Artisan(args) => {
                let mut v = vec!["php".into(), "artisan".into()];
                v.extend(args.split_whitespace().map(String::from));
                v
            }
            CommandKind::ShellInContainer(raw) => {
                vec!["sh".into(), "-c".into(), raw.clone()]
            }
        };
        assert_eq!(cmd_vec, vec!["php", "artisan", "migrate:fresh", "--seed"]);
    }

    #[test]
    fn test_shell_cmd_vec_built_correctly() {
        let kind = CommandKind::ShellInContainer("npm run dev".into());
        let cmd_vec: Vec<String> = match &kind {
            CommandKind::Artisan(args) => {
                let mut v = vec!["php".into(), "artisan".into()];
                v.extend(args.split_whitespace().map(String::from));
                v
            }
            CommandKind::ShellInContainer(raw) => {
                vec!["sh".into(), "-c".into(), raw.clone()]
            }
        };
        assert_eq!(cmd_vec, vec!["sh", "-c", "npm run dev"]);
    }
}
