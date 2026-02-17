use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    pub docs_dir: Option<String>,
    pub format: Option<String>,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Config {
    pub docs_dir: PathBuf,
    pub format: String,
    pub extensions: Vec<String>,
}

impl Config {
    pub fn load(config_path: &Path, cli_dir: Option<&str>) -> Self {
        let file_cfg = std::fs::read_to_string(config_path)
            .ok()
            .and_then(|s| toml::from_str::<FileConfig>(&s).ok())
            .unwrap_or_default();

        let docs_dir = cli_dir
            .map(PathBuf::from)
            .or(file_cfg.docs_dir.map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("docs"));

        let format = file_cfg
            .format
            .unwrap_or_else(|| "%Y-%m-%d-%H-%M-%S".to_string());

        let extensions = file_cfg
            .extensions
            .unwrap_or_else(|| vec!["md".to_string()]);

        Config {
            docs_dir,
            format,
            extensions,
        }
    }
}
