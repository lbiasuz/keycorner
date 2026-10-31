# Repository structure

```text
keycorner/
├── manifest.json              # Omarchy plugin registration (must stay at root)
├── Cargo.toml / Cargo.lock    # Rust package and pinned dependencies
├── keycorner.sh               # Stable install/build/control/test entry point
├── src/                      # Production Rust runtime and installer
├── ui/                       # QML windows, bar widget and JavaScript helpers
│   └── shell.qml             # Standalone development harness
├── config/                   # Hyprland rule and shortcut templates
├── data/                     # Synthetic ANSI presets and labels embedded in Rust
├── firmware/                 # Optional QMK/Vial integrations; GPL license
│   ├── common/               # Shared leased layer reporter
│   ├── crosses/              # Crosses preparation recipe and validation notes
│   └── cstc40/               # CSTC40 build recipe and protocol documentation
├── reference/python/         # Python comparison implementation and old installer
├── tests/
│   ├── native_cli.rs         # Cargo integration tests (standard Cargo location)
│   ├── python/               # Python reference and historical installer tests
│   ├── js/                   # Configuration and onboarding policy tests
│   ├── qml/                  # Qt shortcut/highlight tests
│   └── firmware/             # Native C protocol tests with fake QMK headers
├── packaging/
│   └── runtime-files.txt     # Shared native installer/archive file inventory
├── scripts/                  # Packaging, archive checks, profiling and parity tools
├── docs/                     # Architecture, compatibility, security and release guides
└── .github/workflows/        # CI
```

Root README, changelog, license, security and contribution documents remain easy to find. Rust unit tests live with `src/`, alongside the modules they exercise. `tests/rust-golden.json` contains synthetic parity fixtures, not a personal keyboard layout.

## Runtime and installation

The root manifest points to `ui/Overlay.qml` and `ui/BarWidget.qml`. UI components resolve commands relative to the plugin root, where the installed native executable lives at `bin/keycorner`. The standalone `ui/shell.qml` harness follows the same layout; never run it against the same keyboard as the installed plugin.

`packaging/runtime-files.txt` is compiled into the Rust installer and read by the packaging script. Add a production UI file or desktop template there so source installation and release archives stay consistent. The archive also includes the executable and documentation. Python reference sources and firmware are not included in the native archive. Explicit Python comparison installs copy the reference runtime separately.

`config/` contains templates, not live user configuration. Installation still preserves settings at `~/.config/omarchy/keycorner.json` and managed desktop files under `~/.config/hypr/`.

## Generated and private material

`target/`, `bin/`, `dist/` and Python bytecode caches are generated and ignored by Git. Native release archives are created in `dist/`. Local firmware backups, personal Vial layouts, screenshots and diagnostic files stay outside this repository and must never enter release assets.

## Common commands

- `./keycorner.sh build`: build the runtime into `bin/`.
- `./keycorner.sh install`: build and install the native plugin.
- `./keycorner.sh test`: Rust, Python-reference, parity and JavaScript tests.
- `QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qmltestrunner -input tests/qml -o -,txt`: Qt UI tests.
- `./scripts/package.sh`: build and verify a native archive.
