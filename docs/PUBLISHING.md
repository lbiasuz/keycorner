# Publishing Keycorner

The repository is private; v0.6.0 is a draft with native archive and checksum attached. GitHub publication and marketplace submission are deferred until a later explicit instruction.

The v0.6.0 release candidate uses the Rust runtime on the default `main` branch. The desktop plugin is MIT; optional firmware integration sources are GPL-2.0-or-later and remain separate from the native plugin archive.

## Release procedure

1. Run `./keycorner.sh test`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, the Qt tests documented in README, and the firmware protocol test in CI.
2. Match Cargo, manifest and changelog versions; validate the Omarchy manifest.
3. Run `./scripts/prepare-native.sh`, commit `bin/keycorner` and `packaging/native.json` with the matching sources, then run `./scripts/package.sh`, inspect the archive and verify the packaged executable. Publish its SHA256 checksum alongside it.
4. Fast-forward `main` only after checks pass; tag the same commit and publish the GitHub release.
5. Keep personal layouts, firmware backups, images, temporary build trees and desktop captures out of Git history and release assets.

## Distribution and compatibility

Git installations include the native executable; first enable runs idempotent desktop setup. `./keycorner.sh install` also uses this executable without a build toolchain. Maintainers rebuild through `./scripts/prepare-native.sh`. The native x86_64 archive contains the runtime and installer and needs no Cargo or Python. It is built for current Omarchy/Arch; older distributions are not guaranteed compatible.

## Optional marketplace listing

GitHub publication does not mean marketplace acceptance. The [Omarchy publishing guide](https://plugins.omarchy.org/publish.html) describes a separate issue-form submission and maintainer review. No listing is claimed until accepted.

- Name: Keycorner
- Repository: https://github.com/lbiasuz/keycorner
- Category: System
- Tags: keyboard, overlay, vial
- Summary: A movable, translucent keyboard overlay with standard keyboard presets, Vial layout discovery, live key and mouse-button highlights, modifier indicators and configurable privacy shortcuts.
- Limits: exact layers require optional firmware; physical switch identity is unavailable from ordinary HID output; standard presets use US ANSI labels.

See [marketplace installation](MARKETPLACE_INSTALL.md) for deployment and remaining validation.
