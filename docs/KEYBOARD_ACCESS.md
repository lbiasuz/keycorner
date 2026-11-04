# Keyboard selection and access

Keycorner now separates ordinary keyboard input from Vial layout discovery.
Vial Desktop, QMK build tools and Vial firmware are not prerequisites for
Standard mode. The Rust runtime does not require Python.

## Add and save a keyboard

First-run setup opens Settings. Select a connected keyboard under **Keyboard setup**,
choose a mode/preset, enable access if needed, and click **Add keyboard**. Setup
validates metadata/access and saves its name, mode, preset and layout counts in
`keycorner.json` under `keyboards[deviceId]`. It does not save typing or keymaps.
Reading a Vial layout is a one-shot operation; capture is stopped first and remains
stopped if the session locks. Permission setup stays separately authenticated.

Saved profiles appear alongside connected keyboards. Selecting one restores its
own mode/preset; a disconnected profile remains visible. **Read layout & save**
refreshes the profile, and **Forget** removes the profile and clears selection.
It does not remove system permission rules. Connection errors appear in Settings
and in an otherwise empty overlay instead of leaving an unexplained blank window.
Profiles do not automatically switch capture to an arbitrary newly attached device.

## Choose a mode

Open Settings, select a physical keyboard and choose a mode:

- **Standard:** read-only Linux evdev input. Choose US ANSI 104, TKL (87), or
  60% (61 keys), Brazilian ABNT2 (107 keys), or one of the regional ISO full-size/TKL presets in the [layout catalog](STANDARD_LAYOUTS.md). This works with ordinary USB keyboards, laptop keyboards and
  other Linux keyboard devices with stable physical identity. Bluetooth devices
  require a stable `phys` identity; device variations still need hardware testing.
  Labels follow the selected fixed preset, not an automatic readout of the desktop's XKB
  layout or firmware remapping. Other ISO and custom physical presets are not
  implemented yet. Fn is usually internal to firmware and is not reported.
- **Vial layout:** read the keyboard's physical layout and keymaps. Discovery
  checks a 32-byte raw HID interface on usage page `0xff60`, usage `0x61`;
  setup verifies the protocol/definition rather than matching a list of products.
  Keyboard/consumer report descriptors supply report IDs, offsets and key fields.
  Supported input forms include boot key arrays, NKRO bitmaps, modifier bitmaps,
  and consumer arrays/bitmaps. Unknown fields fail closed or use Standard mode.
  This is capability-based support, not a promise that every Vial firmware and
  layout variant works. Rotated KLE layouts and physical layout-option selection
  remain unsupported. VIA/QMK without Vial cannot supply a Vial definition.

Multiple keyboard-capable event interfaces belonging to the same USB keyboard
are combined, so boot and NKRO output can both be handled. Consumer-only, mouse
and joystick interfaces are excluded. Capture is scoped to one selected device,
not every keyboard. Input is never grabbed or injected. Held keys are initialized
from EVIOCGKEY; SYN_DROPPED overruns are resynchronized according to the Linux ABI.

USB selection includes port, vendor/product and serial identity. Moving the
keyboard to another USB port requires selecting it again, and possibly enabling
access for that port. It does not silently switch to a different keyboard after
an unplug. Internal/non-USB selection uses name, physical path and unique ID.
These identifiers are matching criteria, not cryptographic device authentication.

Existing configurations retain their selected device and mode. Opening settings
can adopt a legacy empty Vial selection if exactly one compatible device is attached. Fresh
installs begin in Standard mode and require selecting a keyboard.

## Labels and firmware versions

The keyboard's reported Vial protocol selects its keycode encoding automatically.
Protocol 0–5 and protocol 6 use different ranges for modifier-tap and layer actions.
The CSTC40 reports protocol 5: its home-row modifier keys now display their tap
letters plus hold modifiers (for example `A / Super`), and tap output usages map
back to those keys for highlighting. Layer tap-toggle keys display `TT n`.
Left/right modifiers remain distinct. No firmware writes or new query types are
needed for this: the protocol version is already returned with the keyboard ID.

True firmware-defined user actions use the definition's `shortName`, `name` or
`title` when available. An unnamed opaque action remains `User n` or a numeric
code; its behavior cannot safely be inferred from a number alone. Tap-dance and
macro execution details are not synthesized. Holding a modifier does not uniquely
identify which physical dual-role key produced it.

Protocol references: [Vial keycode version selection](https://github.com/vial-kb/vial-gui/blob/master/src/main/python/keycodes/keycodes.py),
[legacy constants](https://github.com/vial-kb/vial-gui/blob/master/src/main/python/keycodes/keycodes_v5.py),
[protocol 6 constants](https://github.com/vial-kb/vial-gui/blob/master/src/main/python/keycodes/keycodes_v6.py).

## Enable access

If needed, click **Enable access**. A preview shows the exact device-specific rule
and destination. Confirming starts the normal administrator authentication dialog.
Capture is stopped during the permission preview and authentication, including
when the plugin had been listening before setup. Cancellation makes no new rule.

The one-time helper writes only a Keycorner-owned rule:

```
/etc/udev/rules.d/69-keycorner-<device-id>-standard.rules
/etc/udev/rules.d/69-keycorner-<device-id>-vial.rules
```

Standard rules match keyboard event interfaces on the selected device. Vial rules
match that selected keyboard's hidraw interfaces. USB rules include port and
vendor/product plus serial when available. Non-USB standard rules match name,
physical path and unique ID when available. Unsupported pattern/substitution
characters are rejected rather than interpolated into udev syntax.

The helper uses `TAG+="uaccess"`, before systemd's seat-access rule, to grant the
active local session access. It does not add a group membership, change all input
permissions, install a daemon or policy granting permanent elevation, or edit
Vial's packaged rule. Rule validation runs before atomic installation; udev is
reloaded and only the selected device nodes are retriggered. Root work is limited
to that one setup process. Runtime capture continues as the ordinary desktop user.

`uaccess` grants the session user read/write device-node access, even though the
standard backend opens devices read-only. It is not permission exclusively for
Keycorner, and it does not revoke already-open file descriptors on screen lock.
The plugin's capture shutdown remains necessary. Existing broader rules from Vial
or other software remain unchanged; adding a narrow rule does not narrow them.

The preview/setup helper can also be used from the source checkout:

```sh
target/release/keycorner devices
target/release/keycorner permissions preview DEVICE_ID standard
target/release/keycorner permissions install DEVICE_ID standard
```

Use `vial` instead of `standard` only for a compatible Vial interface. `install`
requests administrator authentication; the normal plugin installer does not.
If access remains unavailable after setup, reconnect the keyboard and Refresh.

## Removal

Stop capture first. Removing the plugin intentionally does not silently elevate
to remove system files. Delete only the exact Keycorner rule(s) shown in the
preview, with administrator privileges, then reload udev and reconnect that
keyboard. Never remove vendor rules or unrelated device rules as part of this
cleanup. Existing Vial-provided permissions may still grant access independently.

## Current Rust validation

The capability-based path read the connected CSTC40 without product-specific code:
47 keys and 10 layers. A subsequent live read verified A/S/D/F tap/hold labels
and their tap-output usages with protocol 5. The Crosses was also re-added through
the same generic setup path (54 keys, 7 layers). The installed panel saved its profile, reread the layout
while capture was active without competing readers, and resumed capture; pause
and resume also passed. Synthetic descriptor tests compare existing boot/NKRO/media
reports against the Python reference, and cover different report IDs, bit offsets,
malformed descriptors, and arbitrary USB IDs in narrowly scoped permission rules.
The Python comparison backend remains a historical Crosses-specific reference;
this setup implementation belongs to the Rust runtime.

## Earlier validation and remaining limits

The standard path was exercised against real Crosses evdev nodes without opening
any hidraw descriptor or making a Vial query. Switching back restored all 54 keys
and 7 Vial layers. Pause terminated the reader; resume created a new connection.
The original Python permission installation was exercised through pkexec; the Rust
port was checked with identical rule previews and validation but did not reinstall
existing system rules. The generated standard
and Vial rules passed `udevadm verify`. Synthetic event tests cover partial records,
repeat suppression, releases and overrun resynchronization. A separate physical
ABNT2 keyboard passed physical key/modifier/numpad highlighting and release tests. Its previously inaccessible event node became accessible through the Rust permission flow. The independent pause shortcut kept the overlay visible and terminated the reader. Hiding also stopped the reader, and reconnecting to the same port resumed highlighting without re-adding the keyboard. Bluetooth and other ISO hardware remain untested.

References: [Linux event protocol](https://docs.kernel.org/input/event-codes.html),
[udev rules](https://www.freedesktop.org/software/systemd/man/latest/udev.html),
[Vial permission setup](https://get.vial.today/manual/linux-udev.html).

The ABNT2 preset uses Linux evdev codes and the Brazilian symbol/geometry conventions from xkeyboard-config (`symbols/br`, `keycodes/evdev`, `geometry/pc`). Enter is approximated by a tall rectangle. Presets do not follow runtime XKB remapping or Num Lock labels automatically.
