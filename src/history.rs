//! Persisted conversion history (most recent first, capped).
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Done,
    Failed,
    Skipped,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub input: PathBuf,
    pub output: PathBuf,
    pub status: Status,
    pub secs: f64,
    pub size: u64,
    pub at: u64, // unix seconds
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("vidly").join("history.json"))
}

pub async fn load() -> Vec<Entry> {
    let Some(path) = path() else { return Vec::new() };
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            tracing::warn!("历史记录解析失败，忽略: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

pub async fn save(entries: &[Entry]) {
    let Some(path) = path() else { return };
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match serde_json::to_string(entries) {
        Ok(s) => {
            if let Err(e) = tokio::fs::write(&path, s).await {
                tracing::warn!("保存历史记录失败: {e}");
            }
        }
        Err(e) => tracing::warn!("序列化历史记录失败: {e}"),
    }
}

/// Inserts `entry` at the front and enforces the cap.
pub fn push(entries: &mut Vec<Entry>, entry: Entry) {
    entries.insert(0, entry);
    entries.truncate(MAX_ENTRIES);
}
