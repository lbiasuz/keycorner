# Dependencies

The native runtime uses libc, libgcc_s and liblzma alongside Omarchy, Quickshell/Qt and Hyprland. Source builds require Rust/Cargo, a C toolchain, pkg-config and liblzma headers. Release checking uses Python; configuration tests use Node.js; UI tests use Qt's test runner. These development tools are not installed-runtime requirements.

Device discovery uses sysfs and udevadm. Optional permission setup uses administrator authentication through pkexec. Vial Desktop is not required. Standard mode does not require Vial firmware. No machine package inventory is distributed.
