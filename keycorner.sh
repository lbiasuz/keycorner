#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
case "${1:-settings}" in
  install)
    shift
    if [[ -f "$project_dir/Cargo.toml" ]]; then
      cargo build --release --locked --manifest-path "$project_dir/Cargo.toml"
      exec "$project_dir/target/release/keycorner" install --source "$project_dir" "$@"
    fi
    exec "$project_dir/bin/keycorner" install --source "$project_dir" "$@" ;;
  build)
    cargo build --release --locked --manifest-path "$project_dir/Cargo.toml"
    mkdir -p "$project_dir/bin"
    install -m 755 "$project_dir/target/release/keycorner" "$project_dir/bin/keycorner.new"
    mv -- "$project_dir/bin/keycorner.new" "$project_dir/bin/keycorner" ;;
  start|enable) exec omarchy plugin enable lbiasuz.keycorner ;;
  stop|disable) exec omarchy plugin disable lbiasuz.keycorner ;;
  settings|toggleSettings|toggle|pause|auto|passthrough|state|reload) exec omarchy-shell keycorner "$1" ;;
  layer|corner|scale|monitor|opacity|configure) exec omarchy-shell keycorner "$1" "${2:?Missing value}" ;;
  test) cd "$project_dir"; cargo test --locked; python scripts/generate_parity.py --check; python -m unittest discover -s tests/python -v; exec node tests/js/test_config.cjs ;;
  *) echo 'Usage: keycorner.sh [install [--engine rust|python]|build|enable|disable|settings|toggleSettings|toggle|pause|state|auto|passthrough|layer N|corner bottom-left|scale 34|opacity 0.78|monitor NAME|configure JSON|test]'; exit 2 ;;
esac
