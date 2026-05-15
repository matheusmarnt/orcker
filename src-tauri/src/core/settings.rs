use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Application settings — persisted via tauri-plugin-store.
#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct AppSettingsData {
    pub theme: String,
    pub locale: String,
    pub tray_enabled: bool,
    pub autostart_enabled: bool,
    pub docker_socket: Option<String>,
}

impl Default for AppSettingsData {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            locale: "en".into(),
            tray_enabled: false,
            autostart_enabled: false,
            docker_socket: None,
        }
    }
}

/// Managed state — tray_enabled as AtomicBool for sync window event handler.
pub struct AppSettings {
    pub tray_enabled: Arc<AtomicBool>,
    pub data: tokio::sync::Mutex<AppSettingsData>,
}

impl AppSettings {
    pub fn new(data: AppSettingsData) -> Self {
        let tray = data.tray_enabled;
        Self {
            tray_enabled: Arc::new(AtomicBool::new(tray)),
            data: tokio::sync::Mutex::new(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_settings_data_default_theme_is_system() {
        let d = AppSettingsData::default();
        assert_eq!(d.theme, "system");
    }

    #[test]
    fn app_settings_data_default_locale_is_en() {
        let d = AppSettingsData::default();
        assert_eq!(d.locale, "en");
    }

    #[test]
    fn app_settings_data_default_tray_is_false() {
        let d = AppSettingsData::default();
        assert!(!d.tray_enabled);
    }

    #[test]
    fn app_settings_new_mirrors_tray_enabled_into_atomic() {
        use std::sync::atomic::Ordering;
        let mut d = AppSettingsData::default();
        d.tray_enabled = true;
        let s = AppSettings::new(d);
        assert!(s.tray_enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn app_settings_data_serializes_to_json() {
        let d = AppSettingsData::default();
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("\"theme\":\"system\""));
        assert!(json.contains("\"locale\":\"en\""));
    }
}
