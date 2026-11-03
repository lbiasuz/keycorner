#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
python3 scripts/native.py
mkdir -p "$project_dir/dist"
staging_dir="$(mktemp -d "$project_dir/dist/.package-XXXXXX")"
trap 'rm -rf -- "$staging_dir"' EXIT
package_dir="$staging_dir/keycorner-rust"
mkdir -p "$package_dir/bin"
while IFS= read -r name; do
  mkdir -p -- "$package_dir/$(dirname -- "$name")"
  cp -- "$name" "$package_dir/$name"
done < packaging/runtime-files.txt
cp -- bin/keycorner "$package_dir/bin/keycorner"
mkdir -p "$package_dir/docs"
cp -- docs/*.md "$package_dir/docs/"
cp -r -- docs/marketplace "$package_dir/docs/marketplace"
tar --owner=0 --group=0 --numeric-owner -C "$staging_dir" -czf dist/keycorner-rust-linux-"$(uname -m)".tar.gz keycorner-rust
python3 scripts/check_package.py "dist/keycorner-rust-linux-$(uname -m).tar.gz"
printf 'Package: %s\n' "$project_dir/dist/keycorner-rust-linux-$(uname -m).tar.gz"
