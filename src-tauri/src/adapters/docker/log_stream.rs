#![allow(deprecated)]
use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::core::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum LogSource {
    Docker,
    Laravel,
    Nginx,
    Supervisor,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct LogLine {
    pub text: String,
    pub source: LogSource,
    pub timestamp: Option<String>,
}

/// Stream docker container logs with cancellation.
/// Tails the last 100 lines then follows new output until token is cancelled.
pub async fn docker_log_stream(
    docker: &Docker,
    container_id: &str,
    source: LogSource,
    on_line: tauri::ipc::Channel<LogLine>,
    token: CancellationToken,
) -> Result<(), AppError> {
    let mut stream = docker.logs(
        container_id,
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            timestamps: true,
            ..Default::default()
        }),
    );

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            maybe_line = stream.next() => {
                match maybe_line {
                    Some(Ok(output)) => {
                        let _ = on_line.send(LogLine {
                            text: output.to_string(),
                            source: source.clone(),
                            timestamp: None, // timestamps embedded in bollard output string
                        });
                    }
                    Some(Err(e)) => return Err(AppError::DockerApi(e.to_string())),
                    None => break,
                }
            }
        }
    }
    Ok(())
}

/// Tail a host filesystem file (e.g. storage/logs/laravel.log) with 1-second polling.
/// Seeks to end of file on open (tail behaviour — only new lines are emitted).
pub async fn file_tail_stream(
    path: String,
    source: LogSource,
    on_line: tauri::ipc::Channel<LogLine>,
    token: CancellationToken,
) -> Result<(), AppError> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};

    // Open and seek to last ~100KB so existing content is shown on attach
    let mut file = File::open(&path)
        .await
        .map_err(|e| AppError::Internal(format!("Cannot open {}: {}", path, e)))?;
    let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(102_400);
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut reader = BufReader::new(file);

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                let mut line = String::new();
                loop {
                    let n = reader.read_line(&mut line).await.unwrap_or(0);
                    if n == 0 {
                        break; // no new content this tick
                    }
                    let trimmed = line.trim_end().to_string();
                    if !trimmed.is_empty() {
                        let _ = on_line.send(LogLine {
                            text: trimmed,
                            source: source.clone(),
                            timestamp: None,
                        });
                    }
                    line.clear();
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
    fn test_log_line_serializes_with_source() {
        let line = LogLine {
            text: "hello".into(),
            source: LogSource::Docker,
            timestamp: None,
        };
        let json = serde_json::to_value(&line).unwrap();
        assert_eq!(json["source"], "Docker");
    }

    #[test]
    fn test_log_source_laravel_serializes() {
        let json = serde_json::to_value(LogSource::Laravel).unwrap();
        assert_eq!(json, "Laravel");
    }
}
