# Contributing

Run `./keycorner.sh test` before submitting changes. On an Omarchy desktop, also
run `omarchy plugin validate .` and exercise the UI. Never run the standalone
harness and installed plugin at the same time against the keyboard.

Please include the hardware model and USB VID/PID when proposing a device
adapter. Do not assume that a QMK device exposes Vial, a physical layout, or the
same HID report format. Tests should use synthetic data. Do not submit personal
keymaps, recorded keystrokes, firmware dumps, screenshots of other applications,
access tokens or local system configuration.

Optional firmware integrations live in `firmware/` under their own GPL license. Keep them separate from the MIT desktop runtime; document hardware identity, recovery, protocol changes and tests. The plugin installer never flashes firmware.

Use `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
and `./keycorner.sh test`. The test command runs Rust and JavaScript tests. Historical
Python implementation/tests and the synthetic parity generator live on the
[`python-reference` branch](https://github.com/lbiasuz/keycorner/tree/python-reference).
The committed golden fixtures remain useful for Rust regression tests; review changes
rather than regenerating them blindly.

See [repository structure](docs/STRUCTURE.md). Add production UI/templates to
`packaging/runtime-files.txt`; both the native installer and release packager consume
that inventory. `main` supports only the Rust runtime.

After changing Rust sources, embedded data, Cargo files or the runtime inventory, run `./scripts/prepare-native.sh` and commit `bin/keycorner` plus `packaging/native.json` with the sources. CI rejects stale bundles. Checksums detect drift, not malicious maintainers; review binaries and source provenance before distribution.
