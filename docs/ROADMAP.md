# Apex Pro Mini native Linux roadmap

Status as of **2026-10-14**. Completion means stated acceptance criteria have actually been met.

| Milestone | State | Verification |
|---|---|---|
| 0. Project structure, rolling handover, CI | **Complete — PR #1** | Merged to main; Python CI green |
| 1. Identify hardware endpoints/report sizes | **Hardware verified** | Control `0xffc0`; 65-byte Output, 513-byte Feature |
| 2. Correct generic `ssgg` report transport | **Verified by owner logs** | Extra report-ID removed; APPLY actually sent; no LED effect |
| 3. Decrypt and inspect official Mini descriptors | **Verified locally** | Engine password extractor worked; four Mini/inherited definitions inspected |
| 4. Establish Mini-specific RGB serializer | **Static specification verified** | New inherited `base_apex_2022_wired` confirms `0x40` Feature, `0x41` Output and field layouts |
| 5. Implement independent Rust Mini codec | **Complete — PR #2** | Squash SHA `2ccb9f67d63d`; bounded frame builders, golden-byte unit tests, guarded device selection, Rust tests and Clippy green on branch |
| 6. Verify real Mini static RGB/per-key selection | **Whole-board solid RGB hardware verified** | A red, S green, `0x41` clear restored, **all physical keys turned blue**. Different simultaneous colors and layout fidelity still pending |
| 7. Lighting effects and Prism profiles | **Static RGB + native keypress ripple hardware-verified; GG parity still in progress** | After PR #6's Mini-scoped udev ACL, the owner reported **“IT WORKS!”** on ripple retest. Other animations, precise wave geometry, systemd service persistence and 31 official preset matches not yet verified |
| 8. Read-only device state/firmware status | Not started | Real readback without altering settings |
| 9. Actuation/Rapid Trigger/remapping | Not started | Independently verified commands and rollback |
| 10. Package Linux service/application | Not started | Native installation, permissions, regressions and CI |

**Confirmed hardware:** A red, S green, clear restoring onboard lighting, and entire keyboard solid blue. **Resolved ripple blocker:** Mini `event11` and `event12` initially failed with `Permission denied (os error 13)`. The scoped input ACL milestone was followed by owner-confirmed **working keypress ripple**. Do not infer exact GG preset fidelity or background daemon reliability from this single live success.

**Latest development:** PR [#4](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/4), squash SHA `570ccddee9c254bd2de28c7e9b1fb62ece192756`, merged full-keyboard effect infrastructure and catalog. **Next hardware gates:** verify optional systemd user service lifecycle, performance and reconnect; test the other animated effects and compare native ripple's spatial behavior to SteelSeries GG. Do not mark these as working before physical feedback.

**Hardware milestone documentation:** [PR #7](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/7) merged as `045b0845561e222983aa9b9db79b77cc912bb7f4`, recording owner-confirmed native ripple operation. Next: verify the optional systemd daemon and exact official preset behavior.

## Input device access and service acceptance criteria

- When a `--list-inputs` scan detects the device but reports `[no read access]`, offer only a dedicated Mini `uaccess` seat ACL rule; avoid `sudo` services, global `input` group grants and world-readable evdev.
- Rule must match `event*` keyboard inputs under `1038:161e`, with a `71-` prefix that precedes systemd `73-seat-late.rules`.
- Owner-confirmed **working ripple** after following the scoped ACL installation/testing guidance. The exact `--test-input` output was not supplied, so individual diagnostic event IDs are not independently documented.

## Service and reactive input acceptance criteria

- Read-only diagnostics identify the correct wired Mini input device(s), with visible A/S/Space keydown codes; no other devices read.
- **Working native ripple owner-confirmed**. Precise expanding geometry, typing unaffected under load, and exact GG preset parity require separate observations.
- Repeated presses, held keys and no-press periods behave correctly; no unintended writes after Ctrl+C.
- Optional `systemd --user` unit runs as the user's UID, never root; auto-restarts after device disconnect and re-enumerates hidraw/event nodes.
- Service installation remains disabled by default until explicitly enabled. No key contents are persisted or logged.

## Comprehensive GG preset parity (long-term requirement)

The target is **all default presets shipped by official SteelSeries GG Prism**, not merely a few similar-looking effects. The Mac GG 108.3 Prism SQL migrations yielded **31 unique preset identifier keys** (documented separately), but the actual current UI list and behavioral details still require verification. Inventory other GG versions and record palettes, timing curves, key masks, base/idle/reactive layering and variants. Implement each independently with comparison tests, plus keyboard layout editor and profile persistence. Track parity case-by-case in [`docs/PRESET_COVERAGE.md`](PRESET_COVERAGE.md). Do not mark any provisional native effect as an exact official preset until visually and behaviorally validated.

## Engineering quality gates

- Default commands must be read-only; HID writes require explicit owner opt-in.
- Match VID/PID, usage page and both report lengths before writing.
- Never truncate packets, silently pick a HID interface, or accept arbitrary firmware/actuation commands.
- No proprietary GG binaries, descriptors, plaintext, or recovered passphrase in the public repo.
- Run `cargo test --all-targets`, `cargo clippy` and Python unit tests in CI. Formatting CI check is pending; add `cargo fmt --check` in a subsequent cleanup.
- Hardware success needs the owner's observation; don't mark it done based on HID API return codes.
- Update `docs/HANDOVER.md` and this roadmap alongside every meaningful change.
