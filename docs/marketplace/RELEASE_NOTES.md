# Keycorner v0.6.0

A floating keyboard overlay for Omarchy with a native Rust runtime, configurable privacy controls and support for ordinary and compatible Vial keyboards.

## Highlights

- 18 standard presets: US ANSI full-size/TKL/60%, Brazilian ABNT2, and full-size/TKL layouts for UK, German, French, Spanish, Latin American Spanish, Italian and Portuguese keyboards.
- Per-user Vial layout discovery and profiles. No personal keymaps or saved devices ship with the plugin.
- Key and modifier highlighting; Vial mouse-button output and optional modifier-hold coloring.
- Movable, resizable, translucent overlay with position lock, click-through and Omarchy theme support.
- Recordable shortcuts for show/hide, Settings and pause without hiding. Hiding, minimizing and screen locking stop the input reader.
- Optional firmware integration for exact live layers; firmware flashing is separate from plugin installation.

## Install

```sh
omarchy plugin add https://github.com/lbiasuz/keycorner --enable --yes
```

Alternatively, download `keycorner-rust-linux-x86_64.tar.gz` and `SHA256SUMS` into the same directory, then run:

```sh
sha256sum -c SHA256SUMS
tar -xzf keycorner-rust-linux-x86_64.tar.gz
cd keycorner-rust
./keycorner.sh install
```

Targets x86_64 Omarchy with glibc 2.39+, libgcc_s and liblzma. Cargo, Python and Vial Desktop are not runtime requirements. Device access is a separate, explicit, authenticated setup step. Existing settings are preserved on update; restart the Omarchy shell if old QML remains cached.

## Validation and limits

Physical tests cover one ordinary ABNT2 keyboard plus Crosses and CSTC40 in Vial mode. Other standard presets have automated validation only. Clean local reinstall/setup, isolated-profile add/update/remove, pause/hide and same-port reconnect were exercised. Fresh-account permission revocation and broader hardware coverage remain unverified.

Presets have fixed legends; desktop remapping, arbitrary physical shapes and all Vial report formats are not covered. Exact layers require compatible firmware. Password fields are not detected: pause input before sensitive typing. Keycorner does not log or transmit typing.

The desktop plugin is MIT; optional firmware sources have their own GPL license and are not included as firmware images. See the repository README, feature tour and compatibility guide. Layout requests and pull requests are welcome through the Keyboard compatibility / layout issue form.
