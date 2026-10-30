# Contributing

Run `./keycorner.sh test` before submitting changes. On an Omarchy desktop, also
run `omarchy plugin validate .` and exercise the UI. Never run the standalone
harness and installed plugin at the same time against the keyboard.

Please include the hardware model and USB VID/PID when proposing a device
adapter. Do not assume that a QMK device exposes Vial, a physical layout, or the
same HID report format. Tests should use synthetic data. Do not submit personal
keymaps, recorded keystrokes, firmware dumps, screenshots of other applications,
access tokens or local system configuration.

Firmware modifications and flashing are outside the current release scope.

On `rust-backend`, use `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
and `./keycorner.sh test`. Cargo tests run without Python; the combined test command
also runs the reference Python and JavaScript tests. `scripts/generate_parity.py`
refreshes synthetic golden fixtures from the reference. It must not connect to a
keyboard. Review fixture changes rather than accepting regenerated expectations blindly.

Use `./keycorner.sh install --engine python` or `--engine rust` from this checkout
for a comparison install. Restart the Omarchy shell if it caches the previous
Backend.js. Never run two bridges against the same Vial interface concurrently.
