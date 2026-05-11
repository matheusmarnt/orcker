#![allow(deprecated)]
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};
use bollard::Docker;
use crate::core::error::AppError;
use std::collections::HashMap;

/// Lazily creates the shared bridge network "orcker-global".
/// No-ops if the network already exists (404 → create, any other status → Err).
pub async fn ensure_global_network(docker: &Docker) -> Result<(), AppError> {
    match docker
        .inspect_network("orcker-global", None::<InspectNetworkOptions<&str>>)
        .await
    {
        Ok(_) => Ok(()),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            let mut labels = HashMap::new();
            labels.insert("managed-by", "orcker");
            docker
                .create_network(CreateNetworkOptions {
                    name: "orcker-global",
                    driver: "bridge",
                    check_duplicate: true,
                    labels,
                    ..Default::default()
                })
                .await
                .map_err(|e| AppError::DockerApi(e.to_string()))?;
            Ok(())
        }
        Err(e) => Err(AppError::DockerApi(e.to_string())),
    }
}

/// Removes "orcker-global" network. Ignores 404 (network already gone).
pub async fn remove_global_network(docker: &Docker) -> Result<(), AppError> {
    match docker.remove_network("orcker-global").await {
        Ok(_) => Ok(()),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(()),
        Err(e) => Err(AppError::DockerApi(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    /// Unit tests for network adapter logic are integration tests requiring a live
    /// Docker socket. Compile-time verification is sufficient for the unit test suite.
    /// See: test_ensure_global_network_compiles
    #[test]
    fn test_ensure_global_network_compiles() {
        // Verifies the function signatures and types compile correctly.
        // Integration tests require a running Docker daemon.
        let _: fn(&bollard::Docker) -> _ = super::ensure_global_network;
        let _: fn(&bollard::Docker) -> _ = super::remove_global_network;
    }
}
