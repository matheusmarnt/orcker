#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("Docker engine unreachable: {0}")]
    DockerUnavailable(String),
    #[error("Docker API error: {0}")]
    DockerApi(String),
    #[error("Permission denied on Docker socket: {0}")]
    DockerPermission(String),
    #[error("Container not found: {0}")]
    ContainerNotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<bollard::errors::Error> for AppError {
    fn from(e: bollard::errors::Error) -> Self {
        let msg = e.to_string();
        if msg.contains("Permission denied") || msg.contains("EACCES") {
            AppError::DockerPermission(msg)
        } else {
            AppError::DockerApi(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_serializes_docker_unavailable() {
        let err = AppError::DockerUnavailable("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"DockerUnavailable\""));
        assert!(json.contains("\"message\":\"test\""));
    }

    #[test]
    fn app_error_serializes_docker_api() {
        let err = AppError::DockerApi("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"DockerApi\""));
        assert!(json.contains("\"message\":\"test\""));
    }
}
