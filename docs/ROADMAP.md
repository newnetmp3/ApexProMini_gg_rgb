# Apex Pro Mini native Linux roadmap

Status as of **2026-10-14**. Completion indicates reproducible evidence, not planned functionality.

| Milestone | Status | Acceptance criterion |
|---|---|---|
| 0. Establish repository, research provenance, documentation, tests | In progress (bootstrap PR) | Files land on `main`, tests pass |
| 1. Identify hardware endpoints and report sizes | **Verified** | Owner's Linux HID descriptor shows `0xFFC0`, Output 65, Feature 513 |
| 2. Fix generic HID transport in upstream checkout | **Verified by logs** | Correct framing and actual APPLY sent; not yet merged upstream |
| 3. Recover and inspect official Mini device definitions | **Verified locally** | Password extracted locally; two exact Mini descriptors decrypt successfully |
| 4. Identify exact RGB command serialization | **Not started** | Capture and independently reproduce actual `direct_write` HID payload bytes |
| 5. Implement isolated Rust Mini codec | **Not started** | Static packet golden tests, strictly bounded report sizes |
| 6. Verify Mini static RGB and per-key selection | **Blocked on milestone 4** | Real hardware responds correctly, three colors and sample keys |
| 7. Add effects, zones, and Prism profiles | **Not started** | Saved/loaded profiles, static, gradient, idle and reactive states |
| 8. Add safe read-only device state and firmware reporting | **Not started** | Status readback without modifying profiles |
| 9. Add actuation / Rapid Trigger / remapping | **Not started** | Each command independently verified, rollback path and safe tests |
| 10. Package and integrate Linux application | **Not started** | Native install/service, udev access, architecture and CI tests |

**Out of scope for initial RGB milestone:** Sonar audio, Moments recording, firmware flashing, unsupported devices, publishing vendor binaries or recovered descriptor secrets.

## Engineering quality gates

- Keep read-only detection separated from opt-in write paths.
- Accept only `1038:161e`, validate the `0xFFC0` HID collection and report sizes on every access.
- No guessed device commands, no report truncation, no accidental writes to input-only interfaces.
- Run unit tests and, when a Rust project is introduced, `cargo fmt --check`, `cargo test` and `cargo clippy`.
- Label hardware-observed results distinctly from static vendor documentation and speculative hypotheses.
- Update `docs/HANDOVER.md` and this roadmap with every meaningful milestone.
