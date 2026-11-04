# Publishing Keycorner

The repository is private; v0.6.0 is a draft with native archive and checksum attached. GitHub publication and marketplace submission are deferred until a later explicit instruction.

The v0.6.0 release candidate uses the Rust runtime on the default `main` branch. The desktop plugin is MIT; optional firmware integration sources are GPL-2.0-or-later and remain separate from the native plugin archive.

## Release procedure

1. Run `./keycorner.sh test`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, the Qt tests documented in README, and the firmware protocol test in CI.
2. Match Cargo, manifest and changelog versions; validate the Omarchy manifest.
3. Run `./scripts/prepare-native.sh`, commit `bin/keycorner` and `packaging/native.json` with the matching sources, then run `./scripts/package.sh`, inspect the archive and verify the packaged executable. Publish its SHA256 checksum alongside it.
4. Fast-forward `main` only after checks pass; tag the same commit and publish the GitHub release.
5. Keep personal layouts, firmware backups, unreviewed images, temporary build trees and personal desktop captures out of Git history and release assets.

## Distribution and compatibility

Git installations include the native executable; first enable runs idempotent desktop setup. `./keycorner.sh install` also uses this executable without a build toolchain. Maintainers rebuild through `./scripts/prepare-native.sh`. The native x86_64 archive contains the runtime and installer and needs no Cargo or Python. It is built for current Omarchy/Arch; older distributions are not guaranteed compatible.

## Optional marketplace listing

GitHub publication does not mean marketplace acceptance. The [Omarchy publishing guide](https://plugins.omarchy.org/publish.html) describes a separate issue-form submission and maintainer review. No listing is claimed until accepted.

- Name: Keycorner
- Repository: https://github.com/lbiasuz/keycorner
- Category: System
- Tags: keyboard, overlay, vial
- Summary: A movable, translucent keyboard overlay with standard keyboard presets, Vial layout discovery, live key and mouse-button highlights, modifier indicators and configurable privacy shortcuts.
- Limits: exact layers require optional firmware; physical switch identity is unavailable from ordinary HID output; standard presets have fixed regional legends and documented validation levels.

See [marketplace installation](MARKETPLACE_INSTALL.md) for deployment and remaining validation.

## Prepared submission materials

- [Draft listing and preview](marketplace/LISTING.md)
- [Validation evidence and remaining acceptance checks](marketplace/READINESS.md)

These materials are private preparation only. Review the documented validation limits and obtain explicit publication authorization before making the repository public, publishing the draft release, or contacting marketplace maintainers.

## Final publication handoff

Public-facing copy and [release notes](marketplace/RELEASE_NOTES.md) are prepared without claiming a marketplace listing. The actual repository remains private and the GitHub release remains a draft until explicitly published.

1. Confirm `main` has a successful Checks run and no additional changes to include.
2. Confirm the draft `v0.6.0` targets that exact commit and contains the matching archive and `SHA256SUMS`. If changes were made after preparation, rebuild and refresh both before continuing.
3. Make the repository public in GitHub Settings → General → Danger Zone → Change visibility. Visibility applies to all repository branches, including historical reference branches.
4. Publish the prepared GitHub draft release. Confirm the resulting `v0.6.0` tag resolves to the tested commit.
5. Verify an unauthenticated clone and download/checksum verification. A normal Git install must contain executable `bin/keycorner` and the root manifest. Existing users need not reset their settings.
6. Open **Submit a plugin** from https://plugins.omarchy.org/publish.html. Use the repository URL, System category, keyboard/overlay/vial tags, prepared [listing](marketplace/LISTING.md), and `docs/marketplace/preview.png`. The feature tour links the additional screenshots.
7. Wait for marketplace validation and maintainer review. Do not describe submission as acceptance.

Do not publish personal recovery directories or temporary capture/build work. No marketplace issue or public message has been sent during preparation.
