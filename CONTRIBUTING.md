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
