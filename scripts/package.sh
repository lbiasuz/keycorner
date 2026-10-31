#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
cargo build --release --locked
mkdir -p "$project_dir/dist"
staging_dir="$(mktemp -d "$project_dir/dist/.package-XXXXXX")"
trap 'rm -rf -- "$staging_dir"' EXIT
package_dir="$staging_dir/keycorner-rust"
mkdir -p "$package_dir/bin"
for name in manifest.json Overlay.qml SettingsWindow.qml BarWidget.qml Config.js Highlight.js Backend.js ShortcutRecorder.qml ShortcutKeys.js README.md LICENSE SECURITY.md CHANGELOG.md CONTRIBUTING.md hyprland.lua shortcuts.lua keycorner.sh; do
  cp -- "$name" "$package_dir/$name"
done
cp -- target/release/keycorner "$package_dir/bin/keycorner"
mkdir -p "$package_dir/docs"
cp -- docs/*.md "$package_dir/docs/"
tar -C "$staging_dir" -czf dist/keycorner-rust-linux-"$(uname -m)".tar.gz keycorner-rust
python3 scripts/check_package.py "dist/keycorner-rust-linux-$(uname -m).tar.gz"
printf 'Package: %s\n' "$project_dir/dist/keycorner-rust-linux-$(uname -m).tar.gz"
