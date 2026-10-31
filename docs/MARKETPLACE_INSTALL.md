# Marketplace installation

## Deployment choice

Omarchy's Git installer clones a plugin repository, validates its manifest and optionally enables it. Updates fast-forward the checkout and rescan plugins. Neither operation invokes Cargo or a custom installer hook.

Keycorner therefore tracks a prebuilt x86_64 executable at `bin/keycorner`. Git updates deliver matching UI and native files together. There is no runtime download, network bootstrap, privileged daemon, or compiler requirement. The runtime still depends on system libc, libgcc_s and liblzma. The current artifact requires glibc 2.39 or newer; other architectures and older systems are not supported by this bundle.

## First enable

The UI runs `keycorner.sh bootstrap` before loading settings or starting capture. It checks architecture and executable presence, then invokes the native installer's setup-only path. This initializes missing settings and installs scoped Hyprland templates/includes with backups, validation and rollback on failure. Existing configuration and customized rules/shortcuts are preserved. A repeat run does not reload Hyprland when no changes are needed.

Failures appear in Settings with a retry action. A failed startup does not enable capture. Input permissions remain a separate explicit, authenticated step. Firmware is never flashed by this flow.

## Maintaining the executable

1. Change and test sources normally.
2. Run `scripts/prepare-native.sh` on the supported build architecture. It strips personal build paths, builds with Cargo.lock, and updates the bundled executable and `packaging/native.json`.
3. Commit the binary, metadata and sources together. CI checks checksum, ELF architecture, version, private-path absence and a fingerprint of compiled source inputs. The fingerprint is a consistency check, not independent reproducible-build attestation.
4. Run `scripts/package.sh` to verify and package that same binary. Rebuilds do not happen during user installation or update.

## Validation scope

Automated tests cover absent executable, unsupported architecture, first setup without Cargo, repeated setup without extra compositor reloads, and preserved hidden/paused settings. Rust tests cover setup rollback. Package checks cover entry points and JS dependencies.

A real fresh-account install/update/remove test remains the next preparation step. Marketplace publication and submission remain explicitly deferred.
