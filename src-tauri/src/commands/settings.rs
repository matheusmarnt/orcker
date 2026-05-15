use crate::core::error::AppError;
use crate::core::settings::{AppSettings, AppSettingsData};
use tauri::State;
use tauri_plugin_store::StoreExt;

#[tauri::command]
#[specta::specta]
pub async fn get_settings(settings: State<'_, AppSettings>) -> Result<AppSettingsData, AppError> {
    let data = settings.data.lock().await;
    Ok(data.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn save_settings(
    app: tauri::AppHandle,
    settings: State<'_, AppSettings>,
    new_data: AppSettingsData,
) -> Result<(), AppError> {
    // Update AtomicBool for sync window event handler
    settings
        .tray_enabled
        .store(new_data.tray_enabled, std::sync::atomic::Ordering::Relaxed);
    // Persist to store
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    store.set(
        "settings",
        serde_json::to_value(&new_data).unwrap_or_default(),
    );
    store
        .save()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    // Update in-memory
    let mut data = settings.data.lock().await;
    *data = new_data;
    Ok(())
}
