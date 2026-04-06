use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub pdb_path: String,
    #[serde(default = "default_view")]
    pub default_view: String,
}

fn default_view() -> String {
    "cpp".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            pdb_path: "./pdb".to_string(),
            default_view: default_view(),
        }
    }
}

impl AppConfig {
    /// Resolve `pdb_path` to an absolute path
    pub fn pdb_dir(&self) -> PathBuf {
        let raw = PathBuf::from(&self.pdb_path);
        if raw.is_absolute() {
            raw
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(raw)
        }
    }
}

/// Save config to `config.toml` next to the executable
pub fn save(config: &AppConfig) -> Result<(), String> {
    let contents = toml::to_string_pretty(config).map_err(|e| e.to_string())?;

    // Try exe dir first, then CWD
    let path = exe_dir()
        .map(|d| d.join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    std::fs::write(&path, contents).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Load config from `config.toml` next to the executable.
pub fn load() -> AppConfig {
    // Installed: config.toml sits next to the exe
    if let Some(path) = exe_dir().map(|d| d.join("config.toml")) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str(&contents) {
                return cfg;
            }
        }
    }

    // Dev / cargo run: try CWD
    if let Ok(contents) = std::fs::read_to_string("config.toml") {
        if let Ok(cfg) = toml::from_str(&contents) {
            return cfg;
        }
    }

    AppConfig::default()
}

/// Directory containing the running executable.
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}
