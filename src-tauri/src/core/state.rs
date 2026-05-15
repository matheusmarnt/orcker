use crate::adapters::docker::client::DockerAdapter;
use crate::core::global_stack::GlobalStackState;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub docker: Arc<RwLock<Option<DockerAdapter>>>,
    pub docker_socket: Arc<RwLock<Option<String>>>,
    pub global_stack: GlobalStackState,
}

impl AppState {
    pub fn disconnected() -> Self {
        Self {
            docker: Arc::new(RwLock::new(None)),
            docker_socket: Arc::new(RwLock::new(None)),
            global_stack: GlobalStackState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disconnected_state_has_no_docker() {
        let state = AppState::disconnected();
        assert!(state.docker.read().await.is_none());
        assert!(state.docker_socket.read().await.is_none());
    }
}
