# Apex Pro Mini native Linux roadmap

Status as of **2026-10-14**. Completion means stated acceptance criteria have actually been met.

| Milestone | State | Verification |
|---|---|---|
| 0. Project structure, rolling handover, CI | **Complete — PR #1** | Merged to main; Python CI green |
| 1. Identify hardware endpoints/report sizes | **Hardware verified** | Control `0xffc0`; 65-byte Output, 513-byte Feature |
| 2. Correct generic `ssgg` report transport | **Verified by owner logs** | Extra report-ID removed; APPLY actually sent; no LED effect |
| 3. Decrypt and inspect official Mini descriptors | **Verified locally** | Engine password extractor worked; four Mini/inherited definitions inspected |
| 4. Establish Mini-specific RGB serializer | **Static specification verified** | New inherited `base_apex_2022_wired` confirms `0x40` Feature, `0x41` Output and field layouts |
| 5. Implement independent Rust Mini codec | **Implemented on branch; CI verification pending** | Bounded report builders, golden-byte unit tests, guarded device detection |
| 6. Verify real Mini static RGB/per-key selection | **Pending owner's hardware test** | Light the A key, clear direct mode, repeat for additional keys/colors |
| 7. Lighting effects and Prism profiles | Not started | Working static/animated/idle/reactive effects and profile persistence |
| 8. Read-only device state/firmware status | Not started | Real readback without altering settings |
| 9. Actuation/Rapid Trigger/remapping | Not started | Independently verified commands and rollback |
| 10. Package Linux service/application | Not started | Native installation, permissions, regressions and CI |

The protocol structure is recovered and now testable; **successful writes are not equivalent to visible RGB response**.

## Engineering quality gates

- Default commands must be read-only; HID writes require explicit owner opt-in.
- Match VID/PID, usage page and both report lengths before writing.
- Never truncate packets, silently pick a HID interface, or accept arbitrary firmware/actuation commands.
- No proprietary GG binaries, descriptors, plaintext, or recovered passphrase in the public repo.
- Run `cargo test --all-targets`, `cargo clippy` and Python unit tests in CI. Add `cargo fmt --check` after format conformance.
- Hardware success needs the owner's observation; don't mark it done based on HID API return codes.
- Update `docs/HANDOVER.md` and this roadmap alongside every meaningful change.
