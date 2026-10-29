# Keycorner

![Checks](https://github.com/lbiasuz/keycorner/actions/workflows/checks.yml/badge.svg)

**Private release preparation · MIT · Omarchy 4 / Hyprland Lua**

A configurable, translucent keyboard overlay for the **Crosses 54 Rev2** (`3034:5659`). Runs inside Omarchy's Quickshell host as `lbiasuz.keycorner`, with a bar button and settings window. The read-only Python bridge needs no third-party packages.

## Use

The keyboard icon in the bar provides:

- **Left-click:** show/hide the overlay.
- **Right-click:** open settings.
- **Middle-click:** pause/resume input.

The overlay has a monochrome **sliders icon** for settings, **pause/resume**, and **×** buttons. Drag any key or empty background to move the floating window. Drag its bottom-right grip to resize; keys scale proportionally. Hyprland’s Super + right-drag resize also works. The header and footer text have been removed; connection details are available in settings. The × button hides only Keycorner; it never exits Omarchy. Hiding, pausing or minimizing stops the input process. Click-through mode leaves the bar button and CLI available.

Settings save automatically to `~/.config/omarchy/keycorner.json` and persist across shell restarts and plugin updates. External edits reload automatically. Invalid settings keep the previous configuration and stop input access until repaired; the settings window displays the error.

| Setting | Options |
| --- | --- |
| Placement | Drag the floating window; it opens near the bottom left |
| Opacity | 15–100%; labels stay opaque |
| Key size | 28–60 px |
| Input | Show/hide, pause/resume |
| Mouse | Click-through toggle |
| Layout | Reference layer, conservative layer suggestions |

Omarchy's current palette is applied automatically. The keyboard floats and is pinned across workspaces, without taking initial focus. Settings also float. A narrow user-owned Hyprland rule is installed in `~/.config/hypr/keycorner.lua`; its include is added to `hyprland.lua` with a backup.

## Compatibility

Targets x86_64 Omarchy Linux.
Python 3.11+ is required (`tomllib`). Node.js is needed only for development tests.

**An arbitrary QMK/Vial keyboard will not be recognized automatically.** Discovery
currently filters Crosses 54 Rev2 USB IDs (`3034:5659`) and understands that board's
boot, NKRO and consumer HID reports. Vial-capable keyboards may provide layout
metadata, but still need device detection and report-decoder support. Plain QMK
does not necessarily expose either Vial or physical layout metadata. Connect one
controller of the supported split keyboard directly; multiple identical controllers
connected to USB at once are not a supported setup.

## Install / update

This repository is currently private; only authorized GitHub accounts can clone it.
The same URL can be used for public installation after publication.

```sh
omarchy plugin add https://github.com/lbiasuz/keycorner.git
python ~/.config/omarchy/plugins/lbiasuz.keycorner/install.py --setup-only
omarchy plugin enable lbiasuz.keycorner
```

Setup creates missing settings and installs narrow floating-window rules in
`~/.config/hypr/keycorner.lua`. It adds one include to `hyprland.lua`, backs up the
original file, validates the configuration and rolls back its edits if validation
fails. Existing settings and customized rules are preserved. No root access is
needed by the plugin or installer.

For development from a separate source checkout, use `python install.py` to copy
the runtime to the user plugin directory and enable it. The installer also works
from an existing plugin checkout without copying files onto themselves.

For git-installed plugins, update with `omarchy plugin update lbiasuz.keycorner`.
If Omarchy retains cached QML, use `omarchy restart shell` after saving your work.
Enabled plugins start with Omarchy; visibility and pause settings govern capture.
The current dragged position and resized dimensions are session-only.

On a fresh machine, the user needs read/write permission to this keyboard's Vial
hidraw interface and read permission to its input HID interfaces. The installer
does not change device permissions. If access fails, verify the VID/PID and use
Vial's [Linux udev setup](https://get.vial.today/manual/linux-udev.html), scoped to
the actual keyboard; do not run this plugin as root or grant all input devices
world-readable access. Close Vial before starting capture.

## Disable / remove

```sh
omarchy plugin disable lbiasuz.keycorner
omarchy plugin remove lbiasuz.keycorner
```

Disabling unloads the plugin and stops its bridge. The settings and separate
Hyprland rules are intentionally retained. For complete cleanup, remove the
`require("hypr.keycorner")` line and its adjacent Keycorner comment from
`~/.config/hypr/hyprland.lua`, then delete `~/.config/hypr/keycorner.lua` and,
optionally, `~/.config/omarchy/keycorner.json`. Run `hyprctl reload` followed by
`hyprctl configerrors` to verify the configuration. Do not remove the Lua rules
file while its `require` remains in the main configuration.

## CLI

```sh
./keycorner.sh settings
./keycorner.sh toggle
./keycorner.sh pause
./keycorner.sh opacity 0.50
./keycorner.sh scale 38
./keycorner.sh monitor DP-2
./keycorner.sh passthrough
./keycorner.sh layer 1
./keycorner.sh auto
./keycorner.sh configure '{"suggestLayers":false}'
./keycorner.sh state
./keycorner.sh test
```

All configuration changes persist. Native IPC uses `omarchy-shell keycorner <method>`. To open through the shell's plugin interface: `omarchy-shell shell summon lbiasuz.keycorner '{"settings":true}'`.

## Keyboard behavior and limits

At each input-process startup, the bridge downloads the actual physical layout and all seven keymaps. It highlights standard HID key output, modifiers, and common media keys, including the board's NKRO reports. Dual-role labels show tap and hold functions. Disconnects clear highlights and trigger reconnection. Pause and resume after remapping in Vial; close Vial first to avoid competing raw-HID transactions.

**The current firmware does not report its active layer.** Layer chips select a saved layout. The `≈` button enables conservative layer suggestions, not firmware-reported state. Suggestions return to the reference preview when that evidence disappears. Silent layer switches, tap-toggle timing, automatic mouse layers, combos, and the complete stack of transparent layers cannot be reliably inferred. Exact live layers and physical-switch telemetry remain a future firmware integration, as requested.

HID reports output keycodes rather than physical switches. Duplicate mappings may highlight multiple positions. A held mod-tap can emit a modifier without revealing its position. Shifted and unshifted mappings may share an output usage. Modifier-only reports never trigger layer suggestions.

Only this model's keyboard interfaces are opened. The Vial client permits only read commands; it neither unlocks Vial nor bypasses matrix-test protection. Key state stays in memory and is piped to the UI, with no typing logs or network transmission. **Password-field keys are visible too. Automatic screen-lock pausing is not implemented; hide or pause before sensitive typing or locking.** The overlay has no pressed-key text line, but still highlights keys.

Device permissions must be configured on each machine. No root access, input-group additions, firmware flashing, or keyboard remapping is performed. Other keyboard models, rotated KLE layouts and alternative HID descriptors need additional support.

## Development

- `manifest.json`: Omarchy overlay and bar-widget registration.
- `Overlay.qml`: overlay, bridge lifecycle, settings persistence and IPC.
- `SettingsWindow.qml`: settings controls.
- `BarWidget.qml`: bar access.
- `Config.js`: shared configuration validation/defaults.
- `keycorner/core.py`: read-only Vial transport, KLE, key labels, HID decoding and layer estimates.
- `keycorner/bridge.py`: transient input stream, reconnect and palette reading.
- `install.py`: repeatable installation preserving settings.
- `tests/`: HID parsing, read-only command guard, layout/inference and configuration validation tests. Tests require Node.js in addition to Python.

`shell.qml` is a standalone development harness (run only with the plugin disabled to avoid duplicate capture). It uses the same persistent settings.

## Release preparation

See [publication checklist](docs/PUBLISHING.md), [changelog](CHANGELOG.md), and [contribution guide](CONTRIBUTING.md). No firmware images or personal layout snapshots are bundled.

## Resource use and security
