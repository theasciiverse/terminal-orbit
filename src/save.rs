use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;

const MAX_SAVES: usize = 10;

const SKIP_MISSIONS: &[&str] = &["lose.msn", "null.msn", "template.msn"];

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveSlot {
    pub mission: String,
    pub timestamp: u64,
}

impl SaveSlot {
    pub fn display_time(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.timestamp);
        let diff = now.saturating_sub(self.timestamp);
        if diff < 120 {
            return "just now".to_string();
        }
        let mins = diff / 60;
        if mins < 60 {
            return format!("{} min ago", mins);
        }
        let hours = mins / 60;
        if hours < 24 {
            return format!("{} hr ago", hours);
        }
        format!("{} days ago", hours / 24)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct SaveFile {
    #[serde(default)]
    slots: Vec<SaveSlot>,
}

fn save_path() -> std::path::PathBuf {
    config_dir().join("saves.toml")
}

pub fn read_saves() -> Vec<SaveSlot> {
    let content = match std::fs::read_to_string(save_path()) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    toml::from_str::<SaveFile>(&content)
        .unwrap_or_default()
        .slots
}

pub fn push_save(mission: &str) {
    if SKIP_MISSIONS.contains(&mission) {
        return;
    }

    let mut slots = read_saves();
    slots.retain(|s| s.mission != mission);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    slots.insert(0, SaveSlot { mission: mission.to_string(), timestamp: ts });
    slots.truncate(MAX_SAVES);

    let _ = std::fs::create_dir_all(config_dir());
    let sf = SaveFile { slots };
    if let Ok(content) = toml::to_string_pretty(&sf) {
        let _ = std::fs::write(save_path(), content);
    }
}
