# Keyboard compatibility

Compatibility is capability-based, not a vendor allowlist. A single Vial candidate is inspected automatically; multiple candidates require selection. Standard keyboards use manually selected US ANSI, Brazilian ABNT2 or regional ISO presets. Permission grants are explicit and device-scoped. Exact live layers require the optional firmware extension.

Current limitations include rotated layouts, unsupported report fields, limited standard presets and USB-port-dependent saved identities. An explicit selection is never silently replaced. Hidden, paused and locked sessions must not begin capture during setup.

Before public publication, validate fresh-account permissions, several independently configured Vial keyboards, ordinary keyboards, hotplug/reconnect, unsupported-device fallback and capture privacy. Keep personal hardware records and test captures outside the repository. Automated tests cover discovery policy, descriptor decoding, configuration, installation and protocol behavior; they do not prove universal hardware compatibility.

See the [standard layout catalog](STANDARD_LAYOUTS.md) for all 18 presets and physical-validation status. Regional presets are available with automated validation even when hardware testing is unavailable; compatibility issues and pull requests are welcome.
