# Rust runtime

The production runtime combines device discovery, input capture, theme loading, settings, shortcuts, permission setup and installation in one native executable. The interface uses QML and JavaScript inside Omarchy's existing shell.

Cargo.lock pins the build dependencies. The runtime dynamically links libc, libgcc_s and liblzma. Rust/Cargo is needed for source builds, not for using a compatible native archive. The runtime does not need Python or Node.js.

The historical Python comparison implementation is retained on the python-reference branch. Synthetic golden fixtures support Rust regression tests without importing the old runtime. Local benchmark results and machine details are deliberately not distributed.
