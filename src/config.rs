use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    pub docs_dir: Option<String>,
    pub format: Option<String>,
    pub extensions: Option<Vec<String>>,
    #[serde(alias = "allow_dirs")]
    pub allow: Option<Vec<String>>,
    #[serde(alias = "deny_dirs")]
    pub deny: Option<Vec<String>>,
    pub depth: Option<usize>,
}

#[derive(Debug)]
pub struct Config {
    pub docs_dir: PathBuf,
    pub format: String,
    pub extensions: Vec<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub depth: usize,
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

        let allow = file_cfg.allow.unwrap_or_default();
        let deny = file_cfg.deny.unwrap_or_default();
        let depth = file_cfg.depth.unwrap_or(1);

        Config {
            docs_dir,
            format,
            extensions,
            allow,
            deny,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config_has_empty_allow_deny() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.toml");
        std::fs::write(&cfg_path, "").unwrap();
        let cfg = Config::load(&cfg_path, None);
        assert!(cfg.allow.is_empty());
        assert!(cfg.deny.is_empty());
        assert_eq!(cfg.depth, 1);
    }

    #[test]
    fn test_config_parses_allow_deny_depth() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        f.write_all(b"allow = [\"running-*\", \"notes\"]\ndeny = [\"archive\"]\ndepth = 2\n")
            .unwrap();
        let cfg = Config::load(&cfg_path, None);
        assert_eq!(cfg.allow, vec!["running-*", "notes"]);
        assert_eq!(cfg.deny, vec!["archive"]);
        assert_eq!(cfg.depth, 2);
    }

    #[test]
    fn test_config_backward_compat_allow_dirs_deny_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        f.write_all(b"allow_dirs = [\"notes\"]\ndeny_dirs = [\"archive\"]\n")
            .unwrap();
        let cfg = Config::load(&cfg_path, None);
        assert_eq!(cfg.allow, vec!["notes"]);
        assert_eq!(cfg.deny, vec!["archive"]);
    }
}
