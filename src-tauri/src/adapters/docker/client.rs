use crate::core::error::AppError;
use bollard::Docker;

pub struct DockerAdapter {
    pub client: Docker,
}

impl DockerAdapter {
    pub async fn connect() -> Result<(Self, String), AppError> {
        // 1. DOCKER_HOST env var takes priority
        if let Ok(host) = std::env::var("DOCKER_HOST") {
            let client = Docker::connect_with_socket(&host, 30, bollard::API_DEFAULT_VERSION)
                .map_err(|e| AppError::DockerUnavailable(e.to_string()))?;
            client
                .ping()
                .await
                .map_err(|e| AppError::DockerUnavailable(e.to_string()))?;
            return Ok((Self { client }, host));
        }

        // 2. Socket path cascade (unix)
        #[cfg(unix)]
        {
            let home = std::env::var("HOME").unwrap_or_default();
            let candidates = vec![
                format!("{home}/.orbstack/run/docker.sock"),
                format!("{home}/.colima/default/docker.sock"),
                format!("{home}/.docker/run/docker.sock"),
                {
                    let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
                    format!("{xdg}/docker.sock")
                },
                "/var/run/docker.sock".to_string(),
            ];

            for path in &candidates {
                if path.is_empty() || path.ends_with('/') {
                    continue;
                }
                if std::path::Path::new(path).exists() {
                    if let Ok(client) =
                        Docker::connect_with_socket(path, 30, bollard::API_DEFAULT_VERSION)
                    {
                        if client.ping().await.is_ok() {
                            return Ok((Self { client }, path.clone()));
                        }
                    }
                }
            }

            Err(AppError::DockerUnavailable(
                "No reachable Docker socket found".to_string(),
            ))
        }

        // 3. Windows named pipe
        #[cfg(windows)]
        {
            let client = Docker::connect_with_named_pipe_defaults()
                .map_err(|e| AppError::DockerUnavailable(e.to_string()))?;
            client
                .ping()
                .await
                .map_err(|e| AppError::DockerUnavailable(e.to_string()))?;
            Ok((Self { client }, "\\\\.\\pipe\\docker_engine".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn docker_host_nonexistent_returns_err() {
        std::env::set_var("DOCKER_HOST", "/nonexistent/docker.sock");
        let result = DockerAdapter::connect().await;
        std::env::remove_var("DOCKER_HOST");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn all_invalid_paths_returns_docker_unavailable() {
        std::env::remove_var("DOCKER_HOST");
        // Unset HOME and XDG_RUNTIME_DIR so no valid sockets are found in CI
        let orig_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/nonexistent_home_for_test");
        std::env::set_var("XDG_RUNTIME_DIR", "/nonexistent_xdg_for_test");

        // /var/run/docker.sock may exist in CI — test only validates error shape when unavailable
        let result = DockerAdapter::connect().await;

        if let Some(h) = orig_home {
            std::env::set_var("HOME", h);
        }
        std::env::remove_var("XDG_RUNTIME_DIR");

        // Either succeeds (Docker running in CI) or returns DockerUnavailable — both valid
        if let Err(e) = result {
            assert!(matches!(e, AppError::DockerUnavailable(_)));
        }
    }
}
