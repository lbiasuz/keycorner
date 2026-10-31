#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
[[ $(uname -m) == x86_64 ]] || { echo 'Prepare the supported x86_64 target on an x86_64 build host.' >&2; exit 1; }
remap_flags="--remap-path-prefix=$HOME=/build/user"$'\x1f'"--remap-path-prefix=$project_dir=/build/keycorner"
export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+${CARGO_ENCODED_RUSTFLAGS}$'\x1f'}$remap_flags"
cargo build --release --locked
python3 scripts/native.py prepare
