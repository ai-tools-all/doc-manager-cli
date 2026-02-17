#!/usr/bin/env bash
set -euo pipefail

NEW_VERSION="${1:?Usage: ./bump.sh <version>}"
BINARY="docs-manager-cli"
CARGO="Cargo.toml"

CURRENT=$(grep '^version' "$CARGO" | head -1 | cut -d'"' -f2)
echo "$BINARY: $CURRENT → $NEW_VERSION"

sed -i "0,/version = \"$CURRENT\"/s//version = \"$NEW_VERSION\"/" "$CARGO"
cargo build --release
rm -f ~/bin/$BINARY
cp target/release/$BINARY ~/bin/

INSTALLED=$(~/bin/$BINARY --version)
echo "installed: $INSTALLED"
