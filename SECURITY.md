# Security and privacy

Keycorner reads raw keyboard output to highlight pressed keys. That access works
across applications, including password fields. Hide or pause it before sensitive
typing or locking the screen; automatic lock detection is not implemented yet.

There is no typing history, analytics, network client or firmware-writing feature.
Current key state is transient in the Python bridge and shared Omarchy UI process.
The plugin is not a sandbox, and these properties do not protect against malicious
code already running as your desktop user.

See [the resource and security review](docs/SECURITY_AND_PERFORMANCE.md) for the
measured behavior, fixes and remaining limitations. Please never include personal
keystrokes, credentials, firmware backups or live desktop captures in a bug report.
