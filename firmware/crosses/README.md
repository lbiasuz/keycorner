# Crosses 54 Rev2 layer reporting

Manufacturer source: `https://github.com/Good-Great-Grand-Wonderful/crosses-vial-qmk`, pinned at `6bfefe2057b7e2b4ea084ece7e64520aeb6b108c` with its pinned submodules.

Target: `gggw/crosses/4x6/rev2:keycorner`. Confirm the trackball arrangement before preparing:

```sh
python firmware/crosses/prepare.py /path/to/crosses-vial-qmk --trackballs right
# Or --trackballs left / --trackballs both, matching the hardware.
make -C /path/to/crosses-vial-qmk gggw/crosses/4x6/rev2:keycorner -j2
```

The recipe retains the manufacturer's seven-layer Vial keymap, UID, custom keycodes, pointing-device logic, OLED support, right-master selection and RP2040 double-reset recovery. It adds the shared layer reporter and a guarded user-housekeeping call before the manufacturer's split housekeeping logic. Both halves need matching firmware for the split protocol. The USB-connected right master serves layer reports; the secondary has no subscriber.

See [the layer protocol](../cstc40/README.md) for packet format, lease expiry and privacy behavior. Firmware integration is GPL-2.0-or-later, as is the upstream firmware; the desktop plugin remains MIT. The firmware source license is included at `../cstc40/LICENSE`.

Preparation and compilation do not flash. Back up each half separately and export current Vial settings first. Match each physical controller's flash ID against its backup before writing. UF2 firmware files must contain only the application range, not the entire flash or settings region. Verify writes before rebooting. Restoration of Vial settings and functional checks of keys, split communication, trackballs, OLEDs and live layers follow flashing.
