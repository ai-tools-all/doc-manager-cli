# Rename Command Design

## CLI

```
docs-manager-cli rename [OPTIONS]

Options:
  -d, --dir <PATH>       Docs directory (default: docs/)
  -x, --execute          Actually rename (default: dry-run)
  -c, --config <PATH>    Config path (default: .doc-manager-cli/config.toml)
```

## Config (.doc-manager-cli/config.toml)

```toml
docs_dir = "docs/"
format = "%Y-%m-%d-%H-%M-%S"
extensions = ["md"]
```

All optional. CLI args > config > defaults.

## Rename Logic

1. Scan docs_dir for files matching configured extensions
2. Check if filename already matches `<date-format>-<title>.<ext>`
3. If not → get created() time (fallback modified()) → build `{date}-{title}.{ext}`
4. Title = existing filename minus ext, lowercased, spaces→hyphens
5. Dry-run: print planned renames | --execute: fs::rename
6. Skip already-correct files

## Error Handling

- Missing docs dir → clear error
- Permission errors → skip file, report
- Name collision → skip, warn

## Dependencies

- clap (derive), serde + toml, chrono

## Modules

- main.rs — CLI, orchestration
- config.rs — TOML loading, defaults, merge
- renamer.rs — scan, detect, rename
