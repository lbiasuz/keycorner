# Keycorner

![Checks](https://github.com/lbiasuz/keycorner/actions/workflows/checks.yml/badge.svg)

**Private release preparation · MIT · Omarchy 4 / Hyprland Lua**

A configurable, translucent overlay for **standard Linux keyboards**, with optional exact Vial layouts for the **Crosses 54 Rev2** (`3034:5659`). Runs inside Omarchy's Quickshell host as `lbiasuz.keycorner`, with a bar button and settings window. The read-only Python bridge needs no third-party packages.

## Use

The keyboard icon in the bar provides:

- **Left-click:** show/hide the overlay.
- **Right-click:** open settings.
- **Middle-click:** pause/resume input.

**Super + Ctrl + Shift + K** hides/pauses or shows/resumes the keyboard overlay.
**Super + Ctrl + Shift + O** opens/closes the settings panel.
**Super + Ctrl + Shift + P** pauses/resumes HID listening while keeping the keyboard visible.

Click a shortcut button in **Settings → Keyboard shortcuts**, press the desired
combination, then click **Apply shortcuts**. Escape cancels recording; **Clear**
disables the action. **Defaults** fills the standard combinations; Apply saves them.
Recording temporarily inhibits compositor shortcuts only for the focused settings
window, and ends on focus loss, cancellation or after 15 seconds. Letters, numbers,
function keys, navigation keys and common punctuation with modifiers are supported.
Conflicts and duplicate combinations are rejected. Failed reloads restore the
previous file. These shortcuts are unavailable while the screen is locked.

Bindings live in `~/.config/hypr/keycorner-shortcuts.lua`, with the previous version
saved as `.lua.backup`. Existing combinations are loaded into settings, including
older installs with no pause shortcut; use Defaults or fill in that field to add it.
Custom Lua beyond the standard binding lines is preserved and must be edited
manually. The helper runs only when opening settings or applying changes.

Capture also stops while Hyprland has no focused application (including empty
workspaces and some launchers). Re-entry checks Omarchy's public lock status;
failed or unknown checks leave capture off. Lock handling is event-driven, with
no periodic polling. This is asynchronous privacy protection, not a security
boundary against other software running as your user.

The overlay has a monochrome **sliders icon** for settings, **pause/resume**, and **×** buttons. Drag any key or empty background to move the floating window. Drag its bottom-right grip to resize; keys scale proportionally. Hyprland’s Super + right-drag resize also works. The header and footer text have been removed; connection details are available in settings. The × button hides only Keycorner; it never exits Omarchy. Hiding, pausing or minimizing stops the input process.
**Lock position** disables dragging from the keyboard/background and hides its
resize grip; buttons remain usable. Hyprland's own window-management commands
(such as Super-drag) remain available.
**Click through overlay** passes all pointer input to windows underneath,
including clicks over the overlay controls. It does not stop HID listening.
Use the settings hotkey or right-click the bar icon to turn it off.
Both switches persist across restarts and can be used independently.

Settings save automatically to `~/.config/omarchy/keycorner.json` and persist across shell restarts and plugin updates. External edits reload automatically. Invalid settings keep the previous configuration and stop input access until repaired; the settings window displays the error.

| Setting | Options |
| --- | --- |
| Placement | Drag the floating window; it opens near the bottom left |
| Opacity | 15–100%; labels stay opaque |
| Key size | 28–60 px |
| Input | Show/hide, pause/resume |
| Mouse | Lock position, click through overlay |
| Layout | Preview layer, conservative layer suggestions |

Omarchy's current palette is applied automatically, including while HID capture is paused, hidden or disconnected. The keyboard floats and is pinned across workspaces, without taking initial focus. Settings also float. A narrow user-owned Hyprland rule is installed in `~/.config/hypr/keycorner.lua`; its include is added to `hyprland.lua` with a backup.

## Compatibility

Targets x86_64 Omarchy Linux.
Python 3.11+ is required (`tomllib`). Node.js is needed only for development tests.

**Standard mode supports ordinary Linux keyboards without Vial or QMK.** Select
one keyboard in Settings and choose a US ANSI full-size, TKL or 60% preset.
These are US QWERTY label presets; automatic XKB labels and ISO/ABNT2/custom
physical layouts are not implemented. Broader physical-keyboard coverage remains a release requirement.

**Vial layout mode remains specific to Crosses 54 Rev2** (`3034:5659`). Other
QMK/Vial boards can use Standard mode, but generic Vial layout/report support and
exact live layer telemetry are not claimed. See [keyboard selection and access](docs/KEYBOARD_ACCESS.md).

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

Select the keyboard in Settings. If necessary, **Enable access** previews and
installs a device-specific Keycorner udev rule after administrator authentication.
Vial Desktop is not required. Standard mode reads keyboard event interfaces;
Vial mode uses the supported board's hidraw interfaces. The normal plugin
installer does not alter device permissions. Capture stops during access setup.
See [keyboard access and rule removal](docs/KEYBOARD_ACCESS.md).

## Disable / remove

```sh
omarchy plugin disable lbiasuz.keycorner
omarchy plugin remove lbiasuz.keycorner
```

Disabling unloads the plugin and stops its bridge. The settings and separate
Hyprland rules and shortcuts are intentionally retained. Remove
`require("hypr.keycorner-shortcuts")` from `~/.config/hypr/bindings.lua` and delete
`~/.config/hypr/keycorner-shortcuts.lua` when uninstalling completely. For complete cleanup, remove the
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

In Vial mode, at each input-process startup, the bridge downloads the actual physical layout and all seven keymaps. It highlights standard HID key output, modifiers, and common media keys, including the board's NKRO reports. Dual-role labels show tap and hold functions. Disconnects clear highlights and trigger reconnection. Pause and resume after remapping in Vial; close Vial first to avoid competing raw-HID transactions.

**The current firmware does not report its active layer.** Layer chips select a saved layout. The `≈` button enables conservative layer suggestions, not firmware-reported state. Suggestions return to the reference preview when that evidence disappears. Silent layer switches, tap-toggle timing, automatic mouse layers, combos, and the complete stack of transparent layers cannot be reliably inferred. Exact live layers and physical-switch telemetry remain a future firmware integration, as requested.

HID reports output keycodes rather than physical switches. Duplicate mappings may highlight multiple positions. A held mod-tap can emit a modifier without revealing its position. Shifted and unshifted mappings may share an output usage. Modifier-only reports never trigger layer suggestions.

Only the selected keyboard's interfaces are opened (legacy Crosses configurations retain automatic discovery until selection). The Vial client permits only read commands; it neither unlocks Vial nor bypasses matrix-test protection. Key state stays in memory and is piped to the UI, with no typing logs or network transmission. **Password-field keys are visible too: use Super + Ctrl + Shift + K before sensitive typing. Screen locking automatically stops capture; unlocking resumes it if it was enabled.** The overlay has no pressed-key text line, but still highlights keys.

Device permissions must be configured on each machine. No root access, input-group additions, firmware flashing, or keyboard remapping is performed. Other keyboard models, rotated KLE layouts and alternative HID descriptors need additional support.

## Development

- `manifest.json`: Omarchy overlay and bar-widget registration.
- `Overlay.qml`: overlay, bridge lifecycle, settings persistence and IPC.
- `SettingsWindow.qml`: settings controls.
- `BarWidget.qml`: bar access.
- `Config.js`: shared configuration validation/defaults.
- `keycorner/keyboard_devices.py`: scoped discovery and stable device selection.
- `keycorner/standard.py`: evdev decoding and US ANSI layout presets.
- `keycorner/permissions.py`: permission preview and one-time authenticated setup.
- `keycorner/core.py`: read-only Vial transport, KLE, key labels, HID decoding and layer estimates.
- `keycorner/bridge.py`: transient input stream, reconnect and palette reading.
- `install.py`: repeatable installation preserving settings.
- `tests/`: HID parsing, read-only command guard, layout/inference and configuration validation tests. Tests require Node.js in addition to Python.

`shell.qml` is a standalone development harness (run only with the plugin disabled to avoid duplicate capture). It uses the same persistent settings.

## Release preparation

See [publication checklist](docs/PUBLISHING.md), [changelog](CHANGELOG.md), and [contribution guide](CONTRIBUTING.md). No firmware images or personal layout snapshots are bundled.

## Resource use and security

Recorder key-mapping checks (Qt 6):

```bash
QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qmltestrunner -input tests/qml -o -,txt
```

See the [dependency and deployment audit](docs/DEPENDENCIES.md) for runtime, default-Omarchy, build/test and device-permission requirements.
