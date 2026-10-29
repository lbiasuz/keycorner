# Changelog

## Unreleased

- Block on keyboard/control events instead of polling while connected.
- Update themes only when files change; suppress duplicate input state.
- Treat device labels as plain text and bound layout decompression/geometry.
- Remove unused human-readable key strings and disable bridge core dumps.
- Stop capture when minimized; document unresolved screen-lock privacy risk.

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
