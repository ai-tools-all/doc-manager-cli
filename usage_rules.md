# docs-manager-cli cheatsheet

```bash
docs-manager-cli rename              # dry-run (preview)
docs-manager-cli rename -x           # execute renames
docs-manager-cli rename -d notes/    # custom directory
docs-manager-cli rename -c cfg.toml  # custom config
```

**Flags:** `-d <dir>` target dir · `-x` execute · `-c <path>` config file

**Config** (`.doc-manager-cli/config.toml`, all optional):
```toml
docs_dir = "docs/"
format = "%Y-%m-%d-%H-%M-%S"
extensions = ["md"]
allow_dirs = ["running-knowledge"]   # glob patterns, default: [] (no subfolders)
deny_dirs = []                        # deny overrides allow
depth = 1                             # subfolder depth, default: 1
```

**Subfolder rules:** denied by default · `allow_dirs = ["*"]` allows all · deny overrides allow · depth=1 means immediate children

**Rules:** dry-run by default · uses file creation time · idempotent · renames in-place · CLI > config > defaults
