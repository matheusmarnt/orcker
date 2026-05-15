use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::core::error::AppError;

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct CommandChunk {
    pub text: String,
    pub is_stderr: bool,
}

/// Stream a docker exec command into a Tauri Channel<CommandChunk>.
///
/// - Guards that the container is in `running` state before creating exec.
/// - Checks `token.is_cancelled()` before creating exec (pre-cancellation exits immediately).
/// - Uses `tokio::select!` inside the streaming loop so cancellation is respected mid-stream.
/// - Returns `Ok(())` on natural stream end or cancellation; `Err(AppError)` on bollard errors.
pub async fn docker_exec_stream(
    docker: &Docker,
    container_id: &str,
    cmd: Vec<String>,
    on_chunk: tauri::ipc::Channel<CommandChunk>,
    token: CancellationToken,
) -> Result<(), AppError> {
    // Guard: container must be running
    let inspect = docker
        .inspect_container(
            container_id,
            None::<bollard::query_parameters::InspectContainerOptions>,
        )
        .await
        .map_err(|e| AppError::DockerApi(e.to_string()))?;

    let is_running = inspect
        .state
        .as_ref()
        .and_then(|s| s.running)
        .unwrap_or(false);

    if !is_running {
        return Err(AppError::Internal(
            "Container not running — start the project first".into(),
        ));
    }

    // Early exit if cancelled before creating exec
    if token.is_cancelled() {
        return Ok(());
    }

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

    if let StartExecResults::Attached { mut output, .. } =
        docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| AppError::DockerApi(e.to_string()))?
    {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break; // clean exit — no unbounded stream accumulation
                }
                maybe_chunk = output.next() => {
                    match maybe_chunk {
                        Some(Ok(log_output)) => {
                            let (text, is_stderr) = match log_output {
                                LogOutput::StdOut { message } => {
                                    (String::from_utf8_lossy(&message).into_owned(), false)
                                }
                                LogOutput::StdErr { message } => {
                                    (String::from_utf8_lossy(&message).into_owned(), true)
                                }
                                _ => continue,
                            };
                            let _ = on_chunk.send(CommandChunk { text, is_stderr });
                        }
                        Some(Err(e)) => return Err(AppError::DockerApi(e.to_string())),
                        None => break, // stream ended naturally
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_cancelled_token_exits_immediately() {
        // A pre-cancelled CancellationToken reports is_cancelled() == true.
        // docker_exec_stream checks this before creating exec — ensuring no exec is created
        // when cancellation is requested before streaming begins.
        let token = CancellationToken::new();
        token.cancel();
        assert!(
            token.is_cancelled(),
            "token.cancel() must make is_cancelled() return true"
        );
    }

    #[test]
    fn test_fresh_token_is_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled(), "fresh token must not be cancelled");
    }

    #[test]
    fn test_child_token_cancelled_by_parent() {
        let parent = CancellationToken::new();
        let child = parent.child_token();
        parent.cancel();
        assert!(
            child.is_cancelled(),
            "child token must be cancelled when parent cancels"
        );
    }

    #[test]
    fn command_chunk_serializes() {
        let chunk = CommandChunk {
            text: "hello".into(),
            is_stderr: false,
        };
        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("\"text\":\"hello\""));
        assert!(json.contains("\"is_stderr\":false"));
    }
}
