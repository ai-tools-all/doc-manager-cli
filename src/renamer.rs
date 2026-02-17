use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};

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

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot read '{}': {e}", dir.display());
            return vec![];
        }
    };

    let mut ops = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
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
        let title = slugify(stem);
        let new_name = format!("{date_str}-{title}.{ext}");
        let new_path = dir.join(&new_name);

        if new_path.exists() {
            eprintln!("warning: '{}' already exists, skipping '{}'", new_name, path.display());
            continue;
        }

        ops.push(RenameOp {
            from: path,
            to: new_path,
        });
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

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}
