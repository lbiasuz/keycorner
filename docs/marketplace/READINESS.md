# Marketplace preparation

Publication is blocked until explicitly authorized. Keep the repository private and release draft; do not submit a marketplace issue yet.

| Area | Evidence | Status |
| --- | --- | --- |
| Git installation | Real Omarchy add/enable in isolated HOME; bundled native executable; setup succeeds | Passed |
| Git update | Fast-forward fixture commit through real updater; hidden/paused settings and opacity preserved | Passed |
| Disable/remove | IPC target disappears; checkout removed; preferences preserved; compositor config valid | Passed |
| Runtime/configuration | Rust unit and CLI integration tests, JS onboarding tests, Qt highlight/shortcut tests | Passed |
| Existing firmware integrations | CSTC40 and Crosses integration tested during development, including layer changes and mouse-button highlighting | Tested on these devices |
| New Vial devices | Capability discovery, multiple-device selection and fallback covered by automated tests | Broader hardware unverified; no third device available |
| Ordinary keyboards | Physical ABNT2 keyboard: permission grant, 107-key preset, tested letters/extra keys/modifiers/numpad and releases; pause shortcut stops reader while visible, hide stops reader, and same-port reconnect resumes without re-adding | Passed for this keyboard; physical US ANSI untested |
| Permissions | Rust authenticated device-scoped grant tested on a previously inaccessible ordinary keyboard | Grant passed on existing account; fresh-account/revocation pending |
| Preview/listing | Reviewed overlay/setup/shortcut screenshots, feature tour and submission copy prepared | Ready for review |
| Public repository/release/submission | Requires explicit publication instruction | Deferred |

The lifecycle sandbox provided a separate HOME, compositor and session bus, with no input device nodes. Missing system services kept capture inactive. It proves lifecycle behavior, not authentication, lock integration or physical keyboard compatibility on a clean installation.

## Remaining manual acceptance checks

1. On a fresh Omarchy account, add and enable the plugin. Confirm the permission action is explicit, targets only the selected device, and works after reconnect. Revoke it and confirm capture stops.
2. Ordinary ABNT2 permission/setup, key/modifier/numpad releases, pause/hide and same-port reconnect have passed. Physical US ANSI and additional standard devices remain useful follow-up coverage.
3. Connect an independently configured Vial keyboard. Verify discovery, permission/setup prompts, labels and reconnect. With multiple Vial keyboards connected, verify explicit selection is required. Exact live layer reporting is expected only with the optional firmware extension.
4. Check show/hide, pause, lock/unlock and click-through recovery using configured shortcuts. Never type a password during a capture test.
5. Review the listing and preview, then obtain explicit authorization before changing visibility or submitting anything.

Keep device identifiers, layouts, logs and account details outside Git. Do not describe pending checks as passed.

## First-release layout coverage

All 18 presets in the [layout catalog](../STANDARD_LAYOUTS.md) are included. Physical testing is not a prerequisite for including a clearly documented preset. Additional keyboards can be supported through compatibility issues and pull requests. This does not convert untested hardware into a passed validation claim.

A clean local reinstall was also completed after removing the prior plugin, keyboard settings and managed desktop rules/includes. Fresh configuration contained no selected device or saved profiles. Setup added only the ordinary ABNT2 keyboard, with zero Vial profiles; the user confirmed layout and key-release behavior. Existing system device permission grants were retained, so this is not a fresh-account permission test. Private recovery copies remain outside the repository.
