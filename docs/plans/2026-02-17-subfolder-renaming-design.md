# Subfolder Renaming Design

## Problem
Tool only renames files in root `docs_dir`. Subfolders ignored.

## Solution
Allow/deny list with glob patterns + configurable depth.

## Config
```toml
allow_dirs = ["running-knowledge", "notes/*"]  # glob patterns
deny_dirs = []                                  # glob patterns
depth = 1                                       # max subfolder depth
```

**Rules:**
- No `allow_dirs` or empty → subfolders denied (backward compat)
- `allow_dirs = ["*"]` → all subfolders allowed up to `depth`
- `deny_dirs` overrides `allow_dirs` (deny wins)
- `depth = 1` → immediate children; `depth = 2` → grandchildren
- Patterns match relative subfolder path from `docs_dir`

## Traversal
1. `WalkDir` with `max_depth(depth + 1)`
2. Root files → always process
3. Subfolder files → check allow/deny globs
4. Renamed files stay in-place

## New deps
- `walkdir = "2"` — recursive traversal
- `glob = "0.3"` — pattern matching

## Unchanged
- `already_formatted`, `slugify`, `strip_date_prefix`, `get_file_time`
- `execute_renames` — works with any path
- CLI args — no new flags
- Default behavior — identical when no allow/deny keys
