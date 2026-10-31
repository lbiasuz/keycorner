# CSTC40 live layers (experimental)

Target: `kprepublic/cstc40/single_pcb`, STM32F402RCT6 board with integrated USB and a separate LED driver. QMK uses its STM32F401 processor target. This is **not** the daughterboard or FXTwink revision.

Build against Vial-QMK commit `dd43959ae5c08d8a28d38a1acf7b04e86b14a344` with its pinned submodules and QMK Python/ARM build tools installed, then run `./firmware/cstc40/build.sh /path/to/vial-qmk`. It copies the upstream Vial keymap, retains 10 layers and 32 tap-dance/combo entries, and sets 32 key overrides to match the test keyboard's capacity. It does not embed personal settings or flash a device.

The firmware and this integration source are GPL-2.0-or-later; the desktop plugin remains MIT. See the upstream Vial-QMK license. Build tools and firmware sources are not plugin runtime dependencies.

## Protocol 1

Existing 32-byte Vial raw HID interface, usage page `ff60`, usage `61`. Host writes include the usual leading zero report ID (33 bytes total).

| Bytes | Meaning |
| --- | --- |
| 0–4 | `fc 4b 43 4c 59` (`fc`, `KCLY`) |
| 5 | Version: 1 |
| 6 | Request: 0 discover, 1 subscribe/renew, 2 unsubscribe; replies: `80`, `81`, `82`; change event: `83` |
| 7 | Reply capability flag: 1 (layer masks); zero in requests |
| 8–11 | Client session token, little-endian u32, echoed in responses |
| 12–15 | QMK `layer_state`, little-endian u32 |
| 16–19 | QMK `default_layer_state`, little-endian u32 |
| 20–31 | Reserved, zero |

Discovery is read-only and does not subscribe. Subscribe/renew affects RAM only and returns current state. The firmware emits an event when either mask changes, sampled by `housekeeping_task_user`. No keycodes, matrix state, text or history are included. The client renews once per second; the firmware expires the lease after three seconds. Graceful shutdown unsubscribes; abrupt termination relies on the lease. Multiple clients are not supported: the most recent subscriber owns the lease. The random token correlates responses; it is not authentication. Normal Vial discovery and editing are preserved.

The Rust client probes once per connection. Unsupported keyboards retain existing behavior, without periodic requests. For supported keyboards it watches the raw interface alongside keyboard reports, validates version/token/mask bounds, and resolves transparent keys against the enabled layer stack. Live layers override manual previews during capture. Pause, hide and session-lock guards stop the bridge as before; reports contain no pressed keys even during lease expiry.

## Flashing and recovery

Identify the PCB, back up firmware and export a `.vil` file before flashing. Vial protocol 5 to 6 requires symbolic keycode migration, including macro/combination/tap-dance codes; do not copy old numeric keycodes directly. Recheck capacities before restoring. Lighting modes may differ between legacy QMK lighting and VialRGB.

Use only the identified STM32 DFU target and serial, with a verified image and recovery dump. Do not unprotect or mass-erase to bypass a failed backup. Flashing is a separate manual operation; `build.sh` never invokes it. Recovery restore is not proven until tested on hardware.
