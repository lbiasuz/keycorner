# Keycorner

![Checks](https://github.com/lbiasuz/keycorner/actions/workflows/checks.yml/badge.svg)

**v0.6.0 · MIT desktop plugin · Omarchy 4 / Hyprland Lua**

A configurable, translucent overlay for **standard Linux keyboards**, with layouts read from compatible **Vial keyboards**. Runs inside Omarchy's Quickshell host as `lbiasuz.keycorner`, with a bar button and settings window. The Rust/Cargo runtime replaces the Python backend and setup helpers with one native executable. The QML interface is shared with the Python reference.

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
| Layout | Preview layer, conservative layer suggestions, automatic firmware layers when available |
| Modifiers | Modifier badges; Vial modifier-hold coloring |

Omarchy's current palette is applied automatically, including while HID capture is paused, hidden or disconnected. The keyboard floats and is pinned across workspaces, without taking initial focus. Settings also float. A narrow user-owned Hyprland rule is installed in `~/.config/hypr/keycorner.lua`; its include is added to `hyprland.lua` with a backup.

## Compatibility

Targets x86_64 Omarchy Linux.
Rust mode requires libc, libgcc_s and liblzma; it does not require Python. Building
from source needs Rust/Cargo 1.88+, a C toolchain, pkg-config and liblzma development
files. Python 3.11+ and Node.js are reference/test dependencies only.

**Standard mode supports ordinary Linux keyboards without Vial or QMK.** Select
one keyboard in Settings and choose a US ANSI full-size, TKL or 60% preset.
These are US QWERTY label presets; automatic XKB labels and ISO/ABNT2/custom
physical layouts are not implemented. Broader physical-keyboard coverage remains a release requirement.

**Vial layout mode uses capabilities, not a vendor/product allowlist.** Setup finds
a compatible raw HID interface, verifies the Vial response, and reads the physical
layout and keymaps. Keyboard, consumer and mouse-button input reports are decoded from their
HID descriptors, including report IDs. Unsupported protocols, rotated layouts and
unsupported report fields produce a setup error; Standard mode remains available.
QMK/VIA alone does not imply Vial layout discovery. Exact layer reporting requires the optional firmware integration. See [keyboard setup](docs/KEYBOARD_ACCESS.md).

## Install / update

Install from source, or download the native x86_64 archive from [GitHub Releases](https://github.com/lbiasuz/keycorner/releases/latest). The archive includes the executable and needs no Rust/Cargo installation: extract it, enter `keycorner-rust`, and run `./keycorner.sh install`. It targets current Omarchy/Arch Linux, not arbitrary older distributions.

```sh
git clone https://github.com/lbiasuz/keycorner.git
cd keycorner
./keycorner.sh install
# Only if Omarchy retains the previous QML in its cache:
omarchy restart shell
```

Setup creates missing settings and installs narrow floating-window rules in
`~/.config/hypr/keycorner.lua`. It adds one include to `hyprland.lua`, backs up the
original file, validates the configuration and rolls back its edits if validation
fails. Existing settings and customized rules are preserved. No root access is
needed by the plugin or installer.

`./keycorner.sh install` builds with Cargo, copies the native runtime into the
plugin directory and enables it. `install.py` is retained as historical Python
reference; do not use it to install the Rust runtime. A packaged build can use the same
shell installer without Cargo. See [Rust architecture and comparison](docs/RUST.md).

For source-built installations, pull changes and rerun `./keycorner.sh install`.
A Git-only plugin update does not rebuild the native executable.
If Omarchy retains cached QML, use `omarchy restart shell` after saving your work.
Enabled plugins start with Omarchy; visibility and pause settings govern capture.
The current dragged position and resized dimensions are session-only.

In **Settings → Keyboard setup**, select a connected keyboard and mode, then click
**Add keyboard**. Vial reads its own layout; Standard uses your selected ANSI preset.
Successful setup saves a separate profile for that keyboard. Selecting a saved
keyboard restores its mode and preset; **Read layout & save** updates it and
**Forget** removes only its profile. Disconnected saved keyboards stay listed.
A first install opens Settings, and an empty overlay shows its error/setup action.

If necessary, **Enable access** previews and
installs a device-specific Keycorner udev rule after administrator authentication.
Vial Desktop is not required. Standard mode reads keyboard event interfaces;
Vial mode uses the selected board's compatible hidraw interfaces. The normal plugin
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

In Vial mode, at each input-process startup, the bridge downloads the actual physical layout and available keymaps. It highlights standard HID key output, modifiers, and common media keys, including the board's NKRO reports and mouse buttons 1–8. Pointer movement is ignored. Dual-role labels show tap and hold functions using the keyboard’s reported Vial protocol, including legacy home-row modifiers. Disconnects clear highlights and trigger reconnection. Pause and resume after remapping in Vial; close Vial first to avoid competing raw-HID transactions.

**Exact live layers are available with the experimental [CSTC40 firmware integration](firmware/cstc40/README.md).** The Rust bridge discovers support automatically and follows firmware layer masks, including transparent-key fallthrough. While capturing, live state takes precedence over the manual preview; a solid dot replaces the approximation button. Other keyboards retain manual layer previews and optional conservative suggestions. Firmware reports contain layer masks only, not physical switch positions; key highlighting still uses ordinary keyboard output reports. Firmware installation is separate from plugin installation. An optional [Crosses firmware integration](firmware/crosses/README.md) is available.

HID reports output keycodes rather than physical switches. Duplicate mappings may highlight multiple positions. Modifier badges show active output modifiers. Optional modifier-hold coloring infers the source of unambiguous mod-tap assignments; duplicate bindings remain ambiguous. Shifted and unshifted mappings may share an output usage. Modifier-only reports never trigger layer suggestions.

Only the selected keyboard's interfaces are opened. Legacy Vial configurations with no selection can adopt a single compatible device. The Vial client permits only read commands; it neither unlocks Vial nor bypasses matrix-test protection. Key state stays in memory and is piped to the UI, with no typing logs or network transmission. **Password-field keys are visible too: use Super + Ctrl + Shift + K before sensitive typing. Screen locking automatically stops capture; unlocking resumes it if it was enabled.** The overlay has no pressed-key text line, but still highlights keys.

Device permissions must be configured on each machine. The runtime stays unprivileged. Optional permission setup authenticates separately; no input-group additions, firmware flashing, or keyboard remapping is performed. Rotated KLE layouts and HID fields outside the supported keyboard/consumer formats need additional support.

## Development

- `manifest.json`: Omarchy overlay and bar-widget registration.
- `Overlay.qml`: overlay, bridge lifecycle, settings persistence and IPC.
- `SettingsWindow.qml`: settings controls.
- `BarWidget.qml`: bar access.
- `Config.js`: shared configuration validation/defaults.
- `Backend.js`: native/Python command routing for comparison installs.
- `src/devices.rs`: scoped discovery and stable device selection.
- `src/core.rs`, `data/`: layouts, labels, reports and layer suggestions.
- `src/stream.rs`: Vial read queries, evdev decoding and blocking input loop.
- `src/permissions.rs`: permission preview and authenticated setup.
- `src/shortcuts.rs`, `src/setup.rs`: shortcut and installation helpers.
- `keycorner/`, `install.py`: unchanged Python reference implementation.
- `tests/`: native tests, synthetic Python parity fixtures and UI/config tests.

Run `./keycorner.sh build` to populate the local `bin/keycorner` first.
`shell.qml` is a standalone development harness (run only with the plugin disabled to avoid duplicate capture). It uses the same persistent settings.

## Releases

See [publication checklist](docs/PUBLISHING.md), [changelog](CHANGELOG.md), and [contribution guide](CONTRIBUTING.md). No firmware images or personal layout snapshots are bundled.

## Resource use and security

Recorder key-mapping checks (Qt 6):

```bash
QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qmltestrunner -input tests/qml -o -,txt
```

See the [dependency and deployment audit](docs/DEPENDENCIES.md) for runtime, default-Omarchy, build/test and device-permission requirements.
