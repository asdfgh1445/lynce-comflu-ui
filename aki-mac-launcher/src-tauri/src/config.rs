use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub swarm_dir: String,
    pub swarm_port: u16,
    pub comfy_dir: String,
    pub comfy_port: u16,
    pub python_bin: String,
    pub extra_model_dirs: Vec<String>,
    pub extra_env: String,
    pub swarm_args: String,
    pub comfy_args: String,
    pub auto_open: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            swarm_dir: home().join("SwarmUI").to_string_lossy().into_owned(),
            swarm_port: 7801,
            comfy_dir: home().join("ComfyUI").to_string_lossy().into_owned(),
            comfy_port: 8188,
            python_bin: "python3.13".into(),
            extra_model_dirs: vec![],
            extra_env: String::new(),
            swarm_args: String::new(),
            comfy_args: String::new(),
            auto_open: true,
        }
    }
}

pub fn app_support_dir() -> PathBuf {
    home().join("Library/Application Support/Lynce")
}

fn legacy_support_dir() -> PathBuf {
    home().join("Library/Application Support/AkiHuishi")
}

pub fn config_path() -> PathBuf {
    app_support_dir().join("config.json")
}

pub fn logs_dir() -> PathBuf {
    app_support_dir().join("logs")
}

pub fn load() -> AppConfig {
    let data = fs::read_to_string(config_path())
        .ok()
        .or_else(|| fs::read_to_string(legacy_support_dir().join("config.json")).ok());
    data.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &AppConfig) -> Result<(), String> {
    let p = config_path();
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&p, body).map_err(|e| e.to_string())
}

pub fn expand_tilde(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        home().join(rest).to_string_lossy().into_owned()
    } else {
        s.to_string()
    }
}

pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
