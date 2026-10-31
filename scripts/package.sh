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
while IFS= read -r name; do
  mkdir -p -- "$package_dir/$(dirname -- "$name")"
  cp -- "$name" "$package_dir/$name"
done < packaging/runtime-files.txt
cp -- target/release/keycorner "$package_dir/bin/keycorner"
mkdir -p "$package_dir/docs"
cp -- docs/*.md "$package_dir/docs/"
tar -C "$staging_dir" -czf dist/keycorner-rust-linux-"$(uname -m)".tar.gz keycorner-rust
python3 scripts/check_package.py "dist/keycorner-rust-linux-$(uname -m).tar.gz"
printf 'Package: %s\n' "$project_dir/dist/keycorner-rust-linux-$(uname -m).tar.gz"
