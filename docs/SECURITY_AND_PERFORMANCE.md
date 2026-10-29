# Resource use and security review

The bridge blocks on input/control events and suppresses duplicate state. Hidden, paused and locked sessions stop capture. Discovery uses device notifications instead of periodic scanning. UI costs are shared with Omarchy's shell.

Measure a deployment with `scripts/profile.py`; keep its output private. CPU counters have finite resolution and whole-shell memory does not measure the plugin in isolation. No local measurement data or machine inventory is distributed.

See SECURITY.md for password-field exposure, asynchronous lock handling, scoped device permissions and the limits of this design. Tests and source review are not an independent security audit.
