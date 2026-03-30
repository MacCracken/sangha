#!/usr/bin/env bash
set -euo pipefail

NEW_VERSION="${1:?Usage: $0 <new-version>}"

echo "$NEW_VERSION" > VERSION

sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

cargo check --quiet 2>/dev/null || true

echo "Bumped to $NEW_VERSION"
