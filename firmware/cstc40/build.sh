#!/usr/bin/env bash
set -euo pipefail
source_root="$(realpath "${1:?Usage: build.sh /path/to/vial-qmk}")"
firmware_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
expected=dd43959ae5c08d8a28d38a1acf7b04e86b14a344
[[ "$(git -C "$source_root" rev-parse HEAD)" == "$expected" ]] || { echo 'Unexpected Vial-QMK commit' >&2; exit 1; }
keymaps="$source_root/keyboards/kprepublic/cstc40/single_pcb/keymaps"
mkdir -p "$keymaps/keycorner"
cp "$keymaps/vial/"{config.h,keymap.c,rules.mk,vial.json} "$keymaps/keycorner/"
cp "$firmware_dir/../common/keycorner_layers.c" "$keymaps/keycorner/"
printf '\n#define VIAL_KEY_OVERRIDE_ENTRIES 32\n' >> "$keymaps/keycorner/config.h"
printf '\nSRC += keycorner_layers.c\n' >> "$keymaps/keycorner/rules.mk"
make -C "$source_root" kprepublic/cstc40/single_pcb:keycorner -j2
