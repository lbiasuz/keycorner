#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
case "${1:-settings}" in
  install) exec python "$project_dir/install.py" ;;
  start|enable) exec omarchy plugin enable lbiasuz.keycorner ;;
  stop|disable) exec omarchy plugin disable lbiasuz.keycorner ;;
  settings|toggle|pause|auto|passthrough|state|reload) exec omarchy-shell keycorner "$1" ;;
  layer|corner|scale|monitor|opacity|configure) exec omarchy-shell keycorner "$1" "${2:?Missing value}" ;;
  test) cd "$project_dir"; python -m unittest discover -s tests -v; exec node tests/test_config.cjs ;;
  *) echo 'Usage: keycorner.sh [install|enable|disable|settings|toggle|pause|state|auto|passthrough|layer N|corner bottom-left|scale 34|opacity 0.78|monitor NAME|configure JSON|test]'; exit 2 ;;
esac
