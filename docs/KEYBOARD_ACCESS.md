# Keyboard selection and access

Keycorner now separates ordinary keyboard input from Vial layout discovery.
Vial Desktop, QMK build tools and Vial firmware are not prerequisites for
Standard mode. The Rust runtime does not require Python.

## Choose a keyboard

Open Settings, select a physical keyboard and choose a mode:

- **Standard:** read-only Linux evdev input. Choose US ANSI 104, TKL (87), or
  60% (61 keys). This works with ordinary USB keyboards, laptop keyboards and
  other Linux keyboard devices with stable physical identity. Bluetooth devices
  require a stable `phys` identity; device variations still need hardware testing.
  Labels are US QWERTY presets, not an automatic readout of the desktop's XKB
  layout or firmware remapping. ISO, ABNT2 and custom physical presets are not
  implemented yet. Fn is usually internal to firmware and is not reported.
- **Vial layout:** preserve the exact physical layout and keymaps from the
  supported Crosses 54 Rev2. Other QMK/Vial keyboards can use Standard mode;
  generic Vial layout/report compatibility is not claimed.

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

Existing Crosses configurations migrate to Vial mode. Opening settings adopts
an explicit selection if exactly one supported Crosses is attached. Fresh
installs begin in Standard mode and require selecting a keyboard.

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
match that supported keyboard's hidraw interfaces. USB rules include port and
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

Use `vial` instead of `standard` only for the supported Vial path. `install`
requests administrator authentication; the normal plugin installer does not.
If access remains unavailable after setup, reconnect the keyboard and Refresh.

## Removal

Stop capture first. Removing the plugin intentionally does not silently elevate
to remove system files. Delete only the exact Keycorner rule(s) shown in the
preview, with administrator privileges, then reload udev and reconnect that
keyboard. Never remove vendor rules or unrelated device rules as part of this
cleanup. Existing Vial-provided permissions may still grant access independently.

## Validation and limits

The standard path was exercised against real Crosses evdev nodes without opening
any hidraw descriptor or making a Vial query. Switching back restored all 54 keys
and 7 Vial layers. Pause terminated the reader; resume created a new connection.
The original Python permission installation was exercised through pkexec; the Rust
port was checked with identical rule previews and validation but did not reinstall
existing system rules. The generated standard
and Vial rules passed `udevadm verify`. Synthetic event tests cover partial records,
repeat suppression, releases and overrun resynchronization. A separate physical
ordinary keyboard and Bluetooth/ISO/ABNT2 hardware have not yet been tested here.

References: [Linux event protocol](https://docs.kernel.org/input/event-codes.html),
[udev rules](https://www.freedesktop.org/software/systemd/man/latest/udev.html),
[Vial permission setup](https://get.vial.today/manual/linux-udev.html).
