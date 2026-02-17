# Subfolder Renaming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add allow/deny list subfolder renaming with configurable depth to docs-manager-cli.

**Architecture:** Extend config with `allow_dirs`, `deny_dirs`, `depth`. Replace `fs::read_dir` with `walkdir` traversal. Filter subfolder files through glob pattern matching against allow/deny lists. Deny wins over allow. Empty allow = no subfolders (backward compat).

**Tech Stack:** Rust, walkdir 2, glob 0.3, existing clap/serde/toml/chrono/regex-lite

---

### Task 1: Add dependencies

**Files:**
- Modify: `Cargo.toml:6-11`

**Step 1: Add walkdir and glob to Cargo.toml**

In `Cargo.toml` under `[dependencies]`, add:
```toml
walkdir = "2"
glob = "0.3"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add walkdir and glob for subfolder support"
```

---

### Task 2: Extend Config struct

**Files:**
- Modify: `src/config.rs`

**Step 1: Write failing test for new config fields**

Add at bottom of `src/config.rs`:
```rust
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
        assert!(cfg.allow_dirs.is_empty());
        assert!(cfg.deny_dirs.is_empty());
        assert_eq!(cfg.depth, 1);
    }

    #[test]
    fn test_config_parses_allow_deny_depth() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        f.write_all(b"allow_dirs = [\"running-*\", \"notes\"]\ndeny_dirs = [\"archive\"]\ndepth = 2\n").unwrap();
        let cfg = Config::load(&cfg_path, None);
        assert_eq!(cfg.allow_dirs, vec!["running-*", "notes"]);
        assert_eq!(cfg.deny_dirs, vec!["archive"]);
        assert_eq!(cfg.depth, 2);
    }
}
```

Also add `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml`.

**Step 2: Run test to verify it fails**

Run: `cargo test -- test_default_config`
Expected: FAIL — `allow_dirs` field doesn't exist on Config

**Step 3: Add fields to FileConfig and Config**

In `src/config.rs`, update `FileConfig`:
```rust
#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    pub docs_dir: Option<String>,
    pub format: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub allow_dirs: Option<Vec<String>>,
    pub deny_dirs: Option<Vec<String>>,
    pub depth: Option<usize>,
}
```

Update `Config`:
```rust
#[derive(Debug)]
pub struct Config {
    pub docs_dir: PathBuf,
    pub format: String,
    pub extensions: Vec<String>,
    pub allow_dirs: Vec<String>,
    pub deny_dirs: Vec<String>,
    pub depth: usize,
}
```

Update `Config::load` to populate the new fields after the existing `extensions` block:
```rust
let allow_dirs = file_cfg.allow_dirs.unwrap_or_default();
let deny_dirs = file_cfg.deny_dirs.unwrap_or_default();
let depth = file_cfg.depth.unwrap_or(1);
```

And add them to the `Config { ... }` return struct.

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: both tests PASS

**Step 5: Commit**

```bash
git add src/config.rs Cargo.toml Cargo.lock
git commit -m "feat: extend config with allow_dirs, deny_dirs, depth"
```

---

### Task 3: Add subfolder filtering logic to renamer

**Files:**
- Modify: `src/renamer.rs`

**Step 1: Write failing test for subfolder filtering**

Add at bottom of `src/renamer.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
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
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -- test_is_subfolder`
Expected: FAIL — `is_subfolder_allowed` doesn't exist

**Step 3: Implement `is_subfolder_allowed`**

Add to `src/renamer.rs` before the tests module:
```rust
use glob::Pattern;

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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -- test_is_subfolder`
Expected: all 5 tests PASS

**Step 5: Commit**

```bash
git add src/renamer.rs
git commit -m "feat: add is_subfolder_allowed with glob matching"
```

---

### Task 4: Replace read_dir with WalkDir traversal

**Files:**
- Modify: `src/renamer.rs` — `plan_renames` function

**Step 1: Write integration test with temp directories**

Add to `tests` module in `src/renamer.rs`:
```rust
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
```

Also add `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml` if not already there (done in Task 2).

**Step 2: Run tests to verify they fail**

Run: `cargo test -- test_plan_renames`
Expected: `test_plan_renames_includes_allowed_subfolder` FAILS (only gets 1 file, not 2)

**Step 3: Rewrite `plan_renames` to use WalkDir**

Replace `plan_renames` in `src/renamer.rs`:
```rust
use walkdir::WalkDir;

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
```

Key change: `new_path = parent.join(...)` instead of `dir.join(...)` for in-place renaming.

**Step 4: Run all tests**

Run: `cargo test`
Expected: all tests PASS

**Step 5: Commit**

```bash
git add src/renamer.rs
git commit -m "feat: walkdir traversal with allow/deny subfolder filtering"
```

---

### Task 5: Update docs

**Files:**
- Modify: `usage_rules.md`

**Step 1: Update usage_rules.md with new config keys**

Add the new config keys to the config example:
```toml
docs_dir = "docs/"
format = "%Y-%m-%d-%H-%M-%S"
extensions = ["md"]
allow_dirs = ["running-knowledge"]   # glob patterns, default: [] (no subfolders)
deny_dirs = []                        # deny overrides allow
depth = 1                             # subfolder depth, default: 1
```

**Step 2: Commit**

```bash
git add usage_rules.md
git commit -m "docs: add subfolder config keys to usage rules"
```

---

### Task 6: Build and manual smoke test

**Step 1: Build release**

Run: `cargo build --release`
Expected: compiles clean

**Step 2: Smoke test with a temp directory**

Create a temp test structure and run dry-run to verify:
```bash
mkdir -p /tmp/test-docs/running-knowledge
echo "test" > /tmp/test-docs/hello.md
echo "test" > /tmp/test-docs/running-knowledge/usage-rules-transitive-deps.md
```

Create a temp config:
```toml
docs_dir = "/tmp/test-docs"
allow_dirs = ["running-knowledge"]
depth = 1
```

Run: `./target/release/docs-manager-cli rename -c /tmp/test-config.toml`
Expected: both files listed in dry-run output

**Step 3: Commit version bump if desired**
