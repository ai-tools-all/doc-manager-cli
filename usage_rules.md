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
```

**Rules:** dry-run by default · uses file creation time · idempotent · CLI > config > defaults
