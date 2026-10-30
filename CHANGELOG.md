# Changelog

## 0.5.0 — Keyboard setup

- Remove vendor/product restrictions from the Rust Vial runtime and device-scoped rules.
- Decode keyboard/consumer input from HID descriptors rather than fixed report IDs.
- Decode key labels and tap outputs using the reported Vial protocol; fix legacy home-row modifiers and layer-toggle labels.
- Add/read/save/forget per-keyboard profiles in Settings, with first-run setup and empty-overlay error feedback.
- Read and validate Vial layouts through a one-shot setup probe while capture is stopped.

## 0.4.0 — Rust comparison branch

- One Cargo-built executable replaces Python input, theme, device, shortcut, permission and installation helpers.
- Shared QML interface and install-time Python/Rust comparison switch.
- Python source remains available as a reference; firmware is unchanged.

## Unreleased

- Add a read-only evdev backend for standard keyboards with ANSI 104/TKL/60% presets.
- Select one physical keyboard; keep Crosses Vial layout support optional.
- Bundle scoped udev setup with rule preview, administrator authentication and capture shutdown.
- Handle partial input events, repeat events, reconnects and SYN_DROPPED resynchronization.

- Add a persistent position lock for overlay dragging and resizing.
- Make full-window click-through explicit and clarify its settings label.

- Record shortcut combinations with focused-window shortcut inhibition, cancel and clear.
- Simplify settings labels and rename Reference layer to Preview layer.
- Load Omarchy colors independently of HID capture, fixing stale settings themes.

- Configure all three shortcuts in settings with conflict checks and rollback.
- Add Super+Ctrl+Shift+P to pause/resume HID input without hiding the keyboard.

- Stop HID capture on Hyprland focus-clear events, including screen locking;
  require an unlocked-session check before resuming.
- Add configurable Super+Ctrl+Shift+K privacy and Super+Ctrl+Shift+O settings shortcuts.
- Validate live lock/unlock shutdown; retain documented asynchronous timing limits.

- Block on keyboard/control events instead of polling while connected.
- Update themes only when files change; suppress duplicate input state.
- Treat device labels as plain text and bound layout decompression/geometry.
- Remove unused human-readable key strings and disable bridge core dumps.
- Stop capture when minimized; document password-field privacy risks.

## 0.3.1 — 2026-10-29

Initial repository release candidate; private preparation, not yet published.

- Omarchy overlay and bar-widget integration with persistent settings.
- Movable, resizable, translucent keyboard window and monochrome controls.
- Crosses 54 Rev2 Vial layout/keymap readout and transient HID key highlights.
- Conservative layer suggestions, with exact firmware layer telemetry deferred.
- Fresh-install configuration initialization and repeatable setup.
- User-owned floating-window rules with validation and rollback on failure.
- MIT license, automated checks and publication instructions.

Known limits: only Crosses 54 Rev2 is supported; generic QMK/Vial discovery,
exact active layers and persistent dragged window geometry are not implemented.
