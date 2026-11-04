# Standard keyboard layouts

Choose a preset in **Settings → Keyboard setup**, then **Add keyboard** (or **Read layout & save**). The selection is saved per keyboard and restored when that profile is selected again. Standard keyboards do not reliably report their printed legends or physical geometry; selection is explicit.

| Region / arrangement | Full-size | TKL | Compact | Physical validation |
| --- | --- | --- | --- | --- |
| US ANSI QWERTY | 104 | 87 | 61 (60%) | Automated only |
| Brazilian ABNT2 | 107 | — | — | Tested on one ordinary keyboard |
| UK ISO QWERTY | 105 | 88 | — | Automated only |
| German ISO QWERTZ | 105 | 88 | — | Automated only |
| French ISO AZERTY (traditional) | 105 | 88 | — | Automated only |
| Spanish ISO (Spain) | 105 | 88 | — | Automated only |
| Spanish ISO (Latin America) | 105 | 88 | — | Automated only |
| Italian ISO QWERTY | 105 | 88 | — | Automated only |
| Portuguese ISO (Portugal) | 105 | 88 | — | Automated only |

All presets are available in the first release. Automated-only presets are not blocked from use, but their physical compatibility has not been confirmed. Reports and pull requests are welcome.

## Built-in presets versus personal devices

The release ships only standard keyboard presets. A fresh installation has no saved devices, device IDs, Vial keymaps or user profiles. Vial layouts are discovered from the user’s connected keyboard and configured locally; saved Vial profile metadata belongs only to that user’s settings and is never bundled into the release. Firmware target source code is not a saved keyboard profile.

## Meaning and limits

Presets describe fixed key positions and base legends. Linux evdev codes identify the physical key independently of its printed character; German Z/Y and French A/Q, for example, retain their position's event code. Selecting a preset does not change the desktop's input language. Labels do not dynamically follow XKB remapping, Shift/AltGr output, dead-key composition or Num Lock state. Modifier indicators show held modifiers separately.

ISO/ABNT2 Enter is drawn as a tall rectangle rather than an exact stepped keycap. Compact and laptop variants may have different bottom rows or key spacing. Traditional French AZERTY is distinct from the newer standardized AZERTY arrangement. Japanese JIS, Korean-specific keys, other scripts, ergonomic shapes and custom layouts are not implied by this catalog. Vial layout discovery remains a separate mode.

Regional legends follow the basic variants in [xkeyboard-config](https://gitlab.freedesktop.org/xkeyboard-config/xkeyboard-config), especially `symbols/{gb,de,fr,es,latam,it,pt}`, `symbols/latin` and `keycodes/evdev`. The generator records the explicit mapping and runs only during development; installation has no XKB-generation dependency.

## Contributing a layout

1. Open a **Keyboard compatibility / layout** issue, or propose a pull request. Identify the language, arrangement, size and the nearest existing preset. State whether the problem concerns position, label, missing key or highlighting.
2. Add a synthetic `data/<preset>.json` using the existing schema. Keep Linux evdev codes in both `codes` and `usages`; do not substitute Unicode or QMK keycodes. For these regional ISO families, update `scripts/generate-regional-layouts.py` and regenerate instead of editing generated JSON alone.
3. Register the preset in `src/core.rs` and the shared `standardPresets` catalog in `ui/Config.js`. The settings selector and saved-profile validation use that catalog.
4. Add a native registration/representative-mapping test. The JavaScript suite checks all selectable presets for unique codes, bounds, overlaps and profile round trips. Describe physical tests separately; untested contributions are accepted for review with that limitation stated.
5. Run `./keycorner.sh test`, `cargo fmt --check`, and `cargo clippy --locked --all-targets -- -D warnings`. Rebuild the native bundle with `scripts/prepare-native.sh` and verify `scripts/package.sh`.

Share a layout diagram or synthetic key list if useful. Do not upload typed input, firmware backups, personal keymaps, serial numbers, passwords or desktop configuration.
