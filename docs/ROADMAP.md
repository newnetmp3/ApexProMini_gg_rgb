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
| 7. Lighting effects and Prism profiles | **Static verified; input diagnostics merged — PR #5** | Owner confirmed solid whole-board blue. First ripple did not respond. Auto multi-node evdev, read-only keypress diagnostic and optional user service merged (SHA `608c668e`); input/ripple behavior awaits hardware retest |
| 8. Read-only device state/firmware status | Not started | Real readback without altering settings |
| 9. Actuation/Rapid Trigger/remapping | Not started | Independently verified commands and rollback |
| 10. Package Linux service/application | Not started | Native installation, permissions, regressions and CI |

**Confirmed hardware:** owner observed **A red, S green, clear restoring onboard lighting, and the entire keyboard solid blue**. Full-board static frames work. **Ripple did not respond to keypresses**; input monitoring needs diagnosis and retest before it can be marked complete.

**Latest development:** PR [#4](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/4), squash SHA `570ccddee9c254bd2de28c7e9b1fb62ece192756`, merged full-keyboard effect infrastructure and catalog. **Next hardware gates:** read-only Linux evdev `--test-input` showing A/S keydowns; then actual keypress-triggered ripple and spatial layout accuracy. Do not mark these as working before physical feedback.

## Service and reactive input acceptance criteria

- Read-only diagnostics identify the correct wired Mini input device(s), with visible A/S/Space keydown codes; no other devices read.
- Ripple event fires on owner hardware with physical expanding lighting around the pressed key, normal typing unaffected.
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
