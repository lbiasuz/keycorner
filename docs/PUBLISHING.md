# Publication preparation

The repository remains private until its owner explicitly chooses to publish.
No marketplace submission or public release has been created.

## Current readiness

- Root manifest, MIT license, README and changelog are included.
- Automated Python tests, settings tests and source compilation run in GitHub Actions.
- The installed Omarchy validator accepts the manifest.
- Installation documents required setup, scoped USB permissions and removal.
- Device support is intentionally limited to Crosses 54 Rev2.
- Firmware backups, personal keymaps and desktop screenshots are excluded.

## Before publishing

1. Exercise a fresh install on another supported Omarchy desktop. Automated tests
   do not replace a real Wayland, Hyprland and USB smoke test.
2. Test hide/show, pause/resume, reconnect, settings persistence, dragging, resizing,
   click-through recovery via the bar, and removal. Confirm the shell stays alive.
3. Test the second keyboard before claiming generic support. Discovery, embedded
   geometry, keycode version and HID descriptors all need validation per device.
4. If a preview is desired, capture the plugin against a clean background with a
   synthetic/demo layout. The private desktop screenshot is not a release asset.
5. Confirm CI is green and the version in `manifest.json` matches the changelog.
6. With owner approval, make the repository public and create the corresponding
   tag/release. Submit the public repository URL and exact commit using the
   marketplace's submission form. Do not imply marketplace approval beforehand.

The [official publishing guide](https://plugins.omarchy.org/publish.html) requires
a public GitHub repository, root manifest, README, license, and safe install/removal.
A preview is optional. Submission is reviewed separately from repository creation.

Proposed listing:

- Name: Keycorner
- Repository: https://github.com/lbiasuz/keycorner
- Category: System
- Tags: keyboard, overlay, vial
- Summary: A movable, translucent Crosses 54 Rev2 keyboard reference for Omarchy,
  with live key highlights, layer previews and persistent settings.
- Limitations: exact layer telemetry and arbitrary keyboard support are not included.
