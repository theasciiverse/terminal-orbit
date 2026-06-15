use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub fn config_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join("terminal-orbit")
}

fn ensure_dir() {
    let _ = std::fs::create_dir_all(config_dir());
}

#[derive(Serialize, Deserialize)]
pub struct Prefs {
    pub player_name: String,
    pub gravity: bool,
    pub dense_stars: bool,
    pub vulnerable: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            player_name: "Ace".to_string(),
            gravity: false,
            dense_stars: false,
            vulnerable: false,
        }
    }
}

pub fn read_prefs() -> Prefs {
    let path = config_dir().join("config.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Prefs::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

pub fn write_prefs(prefs: &Prefs) {
    ensure_dir();
    if let Ok(content) = toml::to_string_pretty(prefs) {
        let _ = std::fs::write(config_dir().join("config.toml"), content);
    }
}
