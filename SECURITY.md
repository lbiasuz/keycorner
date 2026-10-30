# Security and privacy

Keycorner reads raw keyboard output to highlight pressed keys. That access works
across applications, including password fields. Use Super + Ctrl + Shift + K before sensitive typing to hide the overlay and
stop HID listening. Screen locking clears Hyprland focus and stops capture;
unlock resumes it if enabled. Empty workspaces and some launchers also suspend
capture. Re-entry requires a successful public Omarchy lock-status check.

Lock handling is asynchronous: a small interval can exist between the lock
request, its focus-cleared event, and bridge termination. It is not a hard
security boundary. Manually hide/pause first when immediate shutdown matters.
No password-field detection is attempted.

There is no typing history, analytics, network client or firmware-writing feature.
Current key state is transient in the Python bridge and shared Omarchy UI process.
The plugin is not a sandbox, and these properties do not protect against malicious
code already running as your desktop user.

See [the resource and security review](docs/SECURITY_AND_PERFORMANCE.md) for the
measured behavior, fixes and remaining limitations. Please never include personal
keystrokes, credentials, firmware backups or live desktop captures in a bug report.

Standard mode opens only the selected keyboard's Linux event nodes read-only;
Vial mode opens only the selected supported keyboard's HID interfaces. Device
selection persists, and switching backends terminates the previous reader.
The optional Enable access action previews a narrowly scoped udev rule and uses
administrator authentication for a one-time installation. It creates no privileged
daemon or permanent authorization policy. Capture stops before that preview and
authentication. Session ACLs grant access to the desktop user, not exclusively to
Keycorner; existing broader Vial rules are not removed. See
[keyboard access](docs/KEYBOARD_ACCESS.md) for scope and cleanup details.
