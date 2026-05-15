#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum CommandKind {
    /// php artisan {args} inside app container
    Artisan(String),
    /// Raw shell command inside app container
    ShellInContainer(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ArtisanCommand {
    pub id: String,
    pub label: String,
    pub kind: CommandKind,
    pub destructive: bool,
}

pub fn artisan_commands() -> Vec<ArtisanCommand> {
    vec![
        ArtisanCommand {
            id: "migrate".into(),
            label: "migrate".into(),
            kind: CommandKind::Artisan("migrate".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "migrate-fresh-seed".into(),
            label: "migrate:fresh --seed".into(),
            kind: CommandKind::Artisan("migrate:fresh --seed".into()),
            destructive: true,
        },
        ArtisanCommand {
            id: "tinker".into(),
            label: "tinker".into(),
            kind: CommandKind::Artisan("tinker".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "cache-clear".into(),
            label: "cache:clear".into(),
            kind: CommandKind::Artisan("cache:clear".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "config-clear".into(),
            label: "config:clear".into(),
            kind: CommandKind::Artisan("config:clear".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "route-clear".into(),
            label: "route:clear".into(),
            kind: CommandKind::Artisan("route:clear".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "view-clear".into(),
            label: "view:clear".into(),
            kind: CommandKind::Artisan("view:clear".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "queue-restart".into(),
            label: "queue:restart".into(),
            kind: CommandKind::Artisan("queue:restart".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "npm-dev".into(),
            label: "npm run dev".into(),
            kind: CommandKind::ShellInContainer("npm run dev".into()),
            destructive: false,
        },
        ArtisanCommand {
            id: "pest".into(),
            label: "pest".into(),
            kind: CommandKind::ShellInContainer("./vendor/bin/pest".into()),
            destructive: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_minimum_entries() {
        assert!(artisan_commands().len() >= 9);
    }

    #[test]
    fn test_migrate_fresh_is_destructive() {
        let cmd = artisan_commands()
            .into_iter()
            .find(|c| c.id == "migrate-fresh-seed")
            .unwrap();
        assert!(cmd.destructive);
    }

    #[test]
    fn test_migrate_is_not_destructive() {
        let cmd = artisan_commands()
            .into_iter()
            .find(|c| c.id == "migrate")
            .unwrap();
        assert!(!cmd.destructive);
    }

    #[test]
    fn test_pest_command_present() {
        let cmd = artisan_commands().into_iter().find(|c| c.id == "pest");
        assert!(cmd.is_some());
    }
}
