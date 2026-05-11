#[allow(deprecated)]
use bollard::system::EventsOptions;
use futures_util::StreamExt;
use tauri::Emitter;

#[allow(deprecated)]
pub async fn subscribe_events(
    docker: bollard::Docker,
    app_handle: tauri::AppHandle,
) {
    let mut stream = docker.events(Some(EventsOptions::<String> {
        filters: std::collections::HashMap::from([
            ("type".to_string(), vec!["container".to_string()]),
        ]),
        ..Default::default()
    }));

    while let Some(event) = stream.next().await {
        match event {
            Ok(evt) => {
                app_handle.emit("docker://container-event", &evt).ok();
            }
            Err(e) => {
                tracing::warn!("Docker event stream error: {e}");
                break;
            }
        }
    }
}
