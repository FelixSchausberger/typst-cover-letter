use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output: Option<Output>,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    pub dir: Option<String>,
}

pub fn load() -> Result<Config> {
    let path = find_config()
        .context("No config file found (checked XDG_CONFIG_HOME/coverletter/defaults.toml)")?;
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config from {}", path.display()))?;
    Ok(config)
}

fn find_config() -> Option<PathBuf> {
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    let candidate = xdg.join("coverletter").join("defaults.toml");
    if candidate.is_file() {
        return Some(candidate);
    }
    None
}
