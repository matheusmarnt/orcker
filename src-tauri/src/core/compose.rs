#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ComposeDriver {
    Plugin,
    Legacy,
    None,
}

pub fn detect_compose_driver() -> ComposeDriver {
    // Plugin form (Docker Desktop, Docker CLI v2+)
    if std::process::Command::new("docker")
        .args(["compose", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return ComposeDriver::Plugin;
    }
    // Legacy standalone binary (older Ubuntu, Homebrew)
    if std::process::Command::new("docker-compose")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return ComposeDriver::Legacy;
    }
    ComposeDriver::None
}

pub fn compose_command(driver: &ComposeDriver) -> std::process::Command {
    match driver {
        ComposeDriver::Plugin => {
            let mut cmd = std::process::Command::new("docker");
            cmd.arg("compose");
            cmd
        }
        ComposeDriver::Legacy => std::process::Command::new("docker-compose"),
        ComposeDriver::None => panic!("No compose driver available"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_compose_driver_does_not_panic() {
        // Should return Plugin, Legacy, or None — never panic
        let driver = detect_compose_driver();
        // On dev machine with Docker Desktop, expects Plugin
        matches!(
            driver,
            ComposeDriver::Plugin | ComposeDriver::Legacy | ComposeDriver::None
        );
    }
}
