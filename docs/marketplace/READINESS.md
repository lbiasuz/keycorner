# Marketplace preparation

Publication is blocked until explicitly authorized. Keep the repository private and release draft; do not submit a marketplace issue yet.

| Area | Evidence | Status |
| --- | --- | --- |
| Git installation | Real Omarchy add/enable in isolated HOME; bundled native executable; setup succeeds | Passed |
| Git update | Fast-forward fixture commit through real updater; hidden/paused settings and opacity preserved | Passed |
| Disable/remove | IPC target disappears; checkout removed; preferences preserved; compositor config valid | Passed |
| Runtime/configuration | Rust unit and CLI integration tests, JS onboarding tests, Qt highlight/shortcut tests | Passed |
| Existing firmware integrations | CSTC40 and Crosses integration tested during development, including layer changes and mouse-button highlighting | Tested on these devices |
| New Vial devices | Capability discovery, multiple-device selection and fallback covered by automated tests | Independent third keyboard pending |
| Ordinary keyboards | ANSI presets, evdev mapping and setup policy covered by automated tests | Fresh physical setup pending |
| Fresh-account permissions | No broad input-group requirement; explicit device-scoped authenticated setup | Real fresh-account grant/revoke pending |
| Preview/listing | Synthetic ANSI screenshot and submission copy prepared | Ready for review |
| Public repository/release/submission | Requires explicit publication instruction | Deferred |

The lifecycle sandbox provided a separate HOME, compositor and session bus, with no input device nodes. Missing system services kept capture inactive. It proves lifecycle behavior, not authentication, lock integration or physical keyboard compatibility on a clean installation.

## Remaining manual acceptance checks

1. On a fresh Omarchy account, add and enable the plugin. Confirm the permission action is explicit, targets only the selected device, and works after reconnect. Revoke it and confirm capture stops.
2. Connect an ordinary keyboard, choose its ANSI preset, and verify key/modifier press and release, reconnect and device switching.
3. Connect an independently configured Vial keyboard. Verify discovery, permission/setup prompts, labels and reconnect. With multiple Vial keyboards connected, verify explicit selection is required. Exact live layer reporting is expected only with the optional firmware extension.
4. Check show/hide, pause, lock/unlock and click-through recovery using configured shortcuts. Never type a password during a capture test.
5. Review the listing and preview, then obtain explicit authorization before changing visibility or submitting anything.

Keep device identifiers, layouts, logs and account details outside Git. Do not describe pending checks as passed.
