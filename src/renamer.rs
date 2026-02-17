use chrono::{DateTime, Local};
use glob::Pattern;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn plan_renames(config: &Config) -> Vec<RenameOp> {
    let dir = &config.docs_dir;
    if !dir.is_dir() {
        eprintln!("error: directory '{}' does not exist", dir.display());
        return vec![];
    }

    let max_depth = config.depth + 1;
    let mut ops = Vec::new();

    for entry in WalkDir::new(dir).max_depth(max_depth).into_iter().flatten() {
        let path = entry.path().to_path_buf();
        if !path.is_file() {
            continue;
        }

        let parent = match path.parent() {
            Some(p) => p,
            None => continue,
        };

        let is_root = parent == dir;
        if !is_root {
            let rel = match parent.strip_prefix(dir) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => continue,
            };
            if !is_subfolder_allowed(&rel, config) {
                continue;
            }
        }

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) if config.extensions.iter().any(|x| x == e) => e.to_string(),
            _ => continue,
        };

        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };

        if already_formatted(stem, &config.format) {
            continue;
        }

        let timestamp = get_file_time(&path);
        let date_str = timestamp.format(&config.format).to_string();
        let stripped = strip_date_prefix(stem);
        let title = slugify(stripped);
        let new_name = format!("{date_str}-{title}.{ext}");
        let new_path = parent.join(&new_name);

        if new_path.exists() {
            eprintln!("warning: '{}' already exists, skipping '{}'", new_name, path.display());
            continue;
        }

        ops.push(RenameOp { from: path, to: new_path });
    }

    ops
}

pub fn execute_renames(ops: &[RenameOp]) -> usize {
    let mut count = 0;
    for op in ops {
        match fs::rename(&op.from, &op.to) {
            Ok(()) => {
                println!("renamed: {} -> {}", op.from.display(), op.to.display());
                count += 1;
            }
            Err(e) => {
                eprintln!(
                    "error: failed to rename '{}' -> '{}': {e}",
                    op.from.display(),
                    op.to.display()
                );
            }
        }
    }
    count
}

fn already_formatted(stem: &str, format: &str) -> bool {
    let prefix_len = estimate_format_len(format);
    if stem.len() < prefix_len + 1 {
        return false;
    }

    let (date_part, rest) = stem.split_at(prefix_len);
    if !rest.starts_with('-') {
        return false;
    }

    DateTime::parse_from_str(
        &format!("{date_part} +0000"),
        &format!("{format} %z"),
    )
    .is_ok()
}

fn estimate_format_len(format: &str) -> usize {
    format
        .replace("%Y", "2026")
        .replace("%m", "02")
        .replace("%d", "17")
        .replace("%H", "10")
        .replace("%M", "30")
        .replace("%S", "00")
        .len()
}

fn get_file_time(path: &Path) -> DateTime<Local> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok().or_else(|| m.modified().ok()))
        .map(DateTime::<Local>::from)
        .unwrap_or_else(Local::now)
}

fn strip_date_prefix(name: &str) -> &str {
    use regex_lite::Regex;
    static PATTERNS: &[&str] = &[
        r"^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}[-_]?",
        r"^\d{4}-\d{2}-\d{2}[-_]?",
    ];
    for pat in PATTERNS {
        let re = Regex::new(pat).unwrap();
        if let Some(m) = re.find(name) {
            let rest = &name[m.end()..];
            if !rest.is_empty() {
                return rest;
            }
        }
    }
    name
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

fn is_subfolder_allowed(subfolder: &str, config: &Config) -> bool {
    if config.allow_dirs.is_empty() {
        return false;
    }
    for pat in &config.deny_dirs {
        if Pattern::new(pat).map_or(false, |p| p.matches(subfolder)) {
            return false;
        }
    }
    for pat in &config.allow_dirs {
        if Pattern::new(pat).map_or(false, |p| p.matches(subfolder)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_config(allow: Vec<&str>, deny: Vec<&str>, depth: usize) -> Config {
        Config {
            docs_dir: PathBuf::from("/tmp/test-docs"),
            format: "%Y-%m-%d-%H-%M-%S".to_string(),
            extensions: vec!["md".to_string()],
            allow_dirs: allow.into_iter().map(String::from).collect(),
            deny_dirs: deny.into_iter().map(String::from).collect(),
            depth,
        }
    }

    #[test]
    fn test_is_subfolder_allowed_empty_allow() {
        let cfg = make_config(vec![], vec![], 1);
        assert!(!is_subfolder_allowed("notes", &cfg));
    }

    #[test]
    fn test_is_subfolder_allowed_wildcard() {
        let cfg = make_config(vec!["*"], vec![], 1);
        assert!(is_subfolder_allowed("anything", &cfg));
    }

    #[test]
    fn test_is_subfolder_allowed_specific() {
        let cfg = make_config(vec!["running-knowledge"], vec![], 1);
        assert!(is_subfolder_allowed("running-knowledge", &cfg));
        assert!(!is_subfolder_allowed("other", &cfg));
    }

    #[test]
    fn test_is_subfolder_allowed_glob() {
        let cfg = make_config(vec!["running-*"], vec![], 1);
        assert!(is_subfolder_allowed("running-knowledge", &cfg));
        assert!(is_subfolder_allowed("running-notes", &cfg));
        assert!(!is_subfolder_allowed("archive", &cfg));
    }

    #[test]
    fn test_deny_overrides_allow() {
        let cfg = make_config(vec!["*"], vec!["archive"], 1);
        assert!(is_subfolder_allowed("notes", &cfg));
        assert!(!is_subfolder_allowed("archive", &cfg));
    }

    #[test]
    fn test_plan_renames_skips_subfolder_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("notes");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("root-file.md"), "root").unwrap();
        std::fs::write(sub.join("nested-file.md"), "nested").unwrap();

        let cfg = Config {
            docs_dir: dir.path().to_path_buf(),
            format: "%Y-%m-%d-%H-%M-%S".to_string(),
            extensions: vec!["md".to_string()],
            allow_dirs: vec![],
            deny_dirs: vec![],
            depth: 1,
        };
        let ops = plan_renames(&cfg);
        assert_eq!(ops.len(), 1);
        assert!(ops[0].from.file_name().unwrap().to_str().unwrap().contains("root-file"));
    }

    #[test]
    fn test_plan_renames_includes_allowed_subfolder() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("notes");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("root-file.md"), "root").unwrap();
        std::fs::write(sub.join("nested-file.md"), "nested").unwrap();

        let cfg = Config {
            docs_dir: dir.path().to_path_buf(),
            format: "%Y-%m-%d-%H-%M-%S".to_string(),
            extensions: vec!["md".to_string()],
            allow_dirs: vec!["notes".to_string()],
            deny_dirs: vec![],
            depth: 1,
        };
        let ops = plan_renames(&cfg);
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_plan_renames_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("notes");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("my-doc.md"), "content").unwrap();

        let cfg = Config {
            docs_dir: dir.path().to_path_buf(),
            format: "%Y-%m-%d-%H-%M-%S".to_string(),
            extensions: vec!["md".to_string()],
            allow_dirs: vec!["notes".to_string()],
            deny_dirs: vec![],
            depth: 1,
        };
        let ops = plan_renames(&cfg);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].to.parent().unwrap(), sub.as_path());
    }
}
