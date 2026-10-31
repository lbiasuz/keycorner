#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
case "${1:-settings}" in
  bootstrap|install)
    action="$1"
    shift
    if [[ $(uname -m) != x86_64 ]]; then
      printf '%s\n' '{"ok":false,"error":"This release supports x86_64 Linux only."}'
      exit 1
    fi
    if [[ ! -x "$project_dir/bin/keycorner" ]]; then
      printf '%s\n' '{"ok":false,"error":"Native executable missing. Reinstall the complete Keycorner release."}'
      exit 1
    fi
    if [[ "$action" == bootstrap ]]; then
      exec "$project_dir/bin/keycorner" install --setup-only --source "$project_dir"
    fi
    exec "$project_dir/bin/keycorner" install --source "$project_dir" "$@" ;;
  build) exec "$project_dir/scripts/prepare-native.sh" ;;
  start|enable) exec omarchy plugin enable lbiasuz.keycorner ;;
  stop|disable) exec omarchy plugin disable lbiasuz.keycorner ;;
  settings|toggleSettings|toggle|pause|auto|passthrough|state|reload) exec omarchy-shell keycorner "$1" ;;
  layer|corner|scale|monitor|opacity|configure) exec omarchy-shell keycorner "$1" "${2:?Missing value}" ;;
  test) cd "$project_dir"; cargo test --locked; exec node tests/js/test_config.cjs ;;
  *) echo 'Usage: keycorner.sh [install|build|enable|disable|settings|toggleSettings|toggle|pause|state|auto|passthrough|layer N|corner bottom-left|scale 34|opacity 0.78|monitor NAME|configure JSON|test]'; exit 2 ;;
esac
