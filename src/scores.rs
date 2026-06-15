use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;

const MAX_PER_MISSION: usize = 5;

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreEntry {
    pub player_name: String,
    pub score: i32,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct ScoreFile {
    // mission filename → sorted list of top scores (descending)
    #[serde(default)]
    missions: HashMap<String, Vec<ScoreEntry>>,
}

fn score_path() -> std::path::PathBuf {
    config_dir().join("scores.toml")
}

fn read_file() -> ScoreFile {
    let content = match std::fs::read_to_string(score_path()) {
        Ok(s) => s,
        Err(_) => return ScoreFile::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

fn write_file(sf: &ScoreFile) {
    let _ = std::fs::create_dir_all(config_dir());
    if let Ok(content) = toml::to_string_pretty(sf) {
        let _ = std::fs::write(score_path(), content);
    }
}

/// Return the top-N scores for a mission, highest first.
pub fn top_scores(mission: &str) -> Vec<ScoreEntry> {
    read_file()
        .missions
        .get(mission)
        .cloned()
        .unwrap_or_default()
}

/// Record a score for a mission, keeping only the top MAX_PER_MISSION.
pub fn record_score(mission: &str, player_name: &str, score: i32) {
    if score <= 0 {
        return;
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut sf = read_file();
    let entries = sf.missions.entry(mission.to_string()).or_default();
    entries.push(ScoreEntry {
        player_name: player_name.to_string(),
        score,
        timestamp: ts,
    });
    entries.sort_by(|a, b| b.score.cmp(&a.score));
    entries.truncate(MAX_PER_MISSION);
    write_file(&sf);
}
