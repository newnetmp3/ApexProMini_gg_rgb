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
| 7. Lighting effects and Prism profiles | **31 historical selectors with distinct reactive source-timing model; exact parity pending** | Recovered 30 complete GG 108.3 preset definitions plus GG106 redPulse/faze rename; source-derived palettes, durations, signed wavelengths and phase types render offline for 68 Mini keys. Base/idle/reactive compositor + profiles added; 36 additional masked variants implemented across eight records, with approximate Mini coordinate alignment; vendor graphics compositor incomplete |
| 8. Read-only device state/firmware status | Not started | Real readback without altering settings |
| 9. Actuation/Rapid Trigger/remapping | Not started | Independently verified commands and rollback |
| 10. Package Linux service/application | **Service ripple verified; GUI implementation merged (PR #9), owner QA pending** | User service enabled/active with two input readers; owner also verified visible ripples while it runs. Logon/reconnect/stop restoration and distribution packaging pending |

**Confirmed hardware:** A red, S green, clear restoring onboard lighting, and entire keyboard solid blue. **Resolved ripple blocker:** Mini `event11` and `event12` initially failed with `Permission denied (os error 13)`. The scoped input ACL milestone was followed by owner-confirmed **working keypress ripple**. Do not infer exact GG preset fidelity or background daemon reliability from this single live success.

**Latest development:** PR [#4](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/4), squash SHA `570ccddee9c254bd2de28c7e9b1fb62ece192756`, merged full-keyboard effect infrastructure and catalog. **Next hardware gates:** verify optional systemd user service lifecycle, performance and reconnect; test the other animated effects and compare native ripple's spatial behavior to SteelSeries GG. Do not mark these as working before physical feedback.

**Hardware milestone documentation:** [PR #7](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/7) merged as `045b0845561e222983aa9b9db79b77cc912bb7f4`, recording owner-confirmed native ripple operation. Next: verify the optional systemd daemon and exact official preset behavior.

**Latest service test (owner systemd status and follow-up visible ripple):** user-level `apex-mini-ripple.service` installed, enabled and running for ~4m42s with two detected evdev readers, the correct vendor HID interface, and no startup error in the displayed log. Owner additionally answered **yes** when asked whether keypress ripples appear while the daemon is running. Thus service-owned live reactive lighting is hardware-confirmed; logout/login, stop/clear, and reconnect reliability are not. Next: validate live keypress ripples in the background; then clean stop/clear and reconnect/restart.

**Systemd user service documentation:** [PR #8](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/8) merged as `b53923e1228a7af9da0c3f5c226e1f402b8b7705`, recording owner-verified service installed/enabled/active with two input readers. CI green; visible effects in the background, stop/clear and reconnection still require validation.

## Input device access and service acceptance criteria

- When a `--list-inputs` scan detects the device but reports `[no read access]`, offer only a dedicated Mini `uaccess` seat ACL rule; avoid `sudo` services, global `input` group grants and world-readable evdev.
- Rule must match `event*` keyboard inputs under `1038:161e`, with a `71-` prefix that precedes systemd `73-seat-late.rules`.
- Owner-confirmed **working ripple** after following the scoped ACL installation/testing guidance. The exact `--test-input` output was not supplied, so individual diagnostic event IDs are not independently documented.

## Service and reactive input acceptance criteria

- Read-only diagnostics identify the correct wired Mini input device(s), with visible A/S/Space keydown codes; no other devices read.
- **Working native ripple owner-confirmed**. Precise expanding geometry, typing unaffected under load, and exact GG preset parity require separate observations.
- Repeated presses, held keys and no-press periods behave correctly; no unintended writes after Ctrl+C.
- **Observed:** `systemd --user` unit runs under the owner's user slice (UID 1000), enabled and active for ~4m42s. **Not yet observed:** automatic restart after device disconnect and successful hidraw/evdev re-enumeration.
- Service installation is disabled by default, but **owner explicitly enabled it**, and it is now active. The code does not intentionally persist or log key contents.

## Native GUI and profile editor milestone

**Implemented on `main`, PR [#9](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/9) (squash SHA `edf7e45a8082a87b6ebf3e5119afc96f30684248`):** native Wayland/X11 Rust GUI (eframe/egui) with a 68-key RGB preview, fourteen native effect choices, color/brightness/speed/FPS controls, per-key RGB overrides, safe Save/Apply, user service controls and a KDE desktop launcher. Service reloads a versioned, validated, atomically saved user profile; the GUI never competes for the HID writer.

**Acceptance:** GUI **compile passed in CI**; Arch/KDE GUI launch and rendering **not yet owner-verified**; profile survives restarts; applying a new preset changes hardware, keypress ripple continues after switching, per-key override works, service stop restores onboard lighting. *Until owner tests, implementation ≠ verified behavior.*

Full port remains a separate roadmap: load actual GG preset configurations and match **all default SteelSeries GG Prism presets** with layered base/idle/reactive effects, parameter editing, masks/zones and save/import.

## Factory catalog conversion and layer compositor (PR #10 merged)

**PR [#10](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/10) merged** as `8629738a38d39d867b8a5137b6fee9fd897e53c3`; Rust + GUI + Python PR CI passed. **31 historical identifiers ≠ 31 distinct source definitions:** the owner's GG108.3 Prism SQL migrations contain 30 complete preset configurations, and GG106 later renames the existing `faze` locale key to `redPulse`. Independent Rust selectors `gg:<name>` now provide **first-variant source-parameter-derived previews** for each historical name. All 31 can produce the Mini's 513-byte frame offline; no new hardware write commands were introduced.

Added a reusable `base + idle + reactive` pure lighting compositor, compatible with existing version-1 profiles; when no layer is configured, prior confirmed native ripple rendering remains unchanged.

**Still blocking strict parity:** **36 additional multi-region masks now implemented but coordinate alignment unverified**, partial foreground/background/aperture scene rendering now implemented for 3 presets; original GG aperture shader, alpha and graph sequencing remain unverified, device-specific geometry, 121-version profile catalogue differences, **five heatmap analog threshold bands mapped offline but missing verified live analog data**, and side-by-side real-hardware confirmation. Full details and next-test matrix: [`docs/FACTORY_PRESETS.md`](FACTORY_PRESETS.md).

## Automated offline visual verification

**Merged in [PR #11](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/11), SHA `45d9bb363cfc5518c6659e2c07b93808abe522a3`:** read-only binary `prism-preview` renders a complete SVG sheet with **31 factory-style modes × three phases**, including synthetic keypress samples for reactive presets. GitHub Actions uploads a downloadable `apex-mini-prism-factory-previews` artifact. This supports remote inspection and regression review with no keyboard or graphical desktop. Generator Rust/GUI/Python CI green and gallery artifact successfully uploaded to GitHub Actions. Actual GG visual fidelity is still not verified.

**Next fidelity gates** after the gallery: GG foreground/background/aperture compositor, six-row to Mini-five-row coordinate calibration, analog Hall sensor readback, timing/opacity matching and device-side comparisons. SVG color regression is **not** proof of true vendor parity.

## Source-derived compound graphics (PR #12 merged; shader parity pending)

[PR #12](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/12) merged as `ca2deeca512a43a44a1dc08aaa65bcf5468b319c` with Rust, Clippy, GUI, Python and SVG-gallery CI passing. It adds an independent pure renderer for `westCoast`, `haze` and `warpDrive` foreground/background/aperture structures. Gradient colors, periods, wave signs and geometry classes are sourced from authorized local GG configuration analysis, but the bounded cosine-based aperture interpolation and compositing equations are our own. Offline Rust frame tests apply; full vendor shader parity is **not** achieved or owner-verified.

**Independent generic support merged (PR #13):** z-ordered scenes, per-layer opacity, Normal/Add/Multiply/Screen blends, origin/rotation/scale, repeats and phase offsets, cosine/band apertures. West Coast, Haze and Warp Drive migrated to declarative scene recipes. These are original, offline-tested primitives and not GG shader equivalence. See [`docs/COMPOSITOR.md`](COMPOSITOR.md).

**Next fidelity milestone:** verify GG's actual mask/anchor/opacity semantics from authorized reference behavior, then accurately port reactive graphics and regional layout coordinates, preserving original HID and native ripple. No GUI QA while user is remote.

## Comprehensive GG preset parity (long-term requirement)

The target is **all default presets shipped by official SteelSeries GG Prism**, not merely a few similar-looking effects. The Mac GG 108.3 Prism SQL migrations yielded **31 unique preset identifier keys** (documented separately), but the actual current UI list and behavioral details still require verification. Inventory other GG versions and record palettes, timing curves, key masks, base/idle/reactive layering and variants. Implement each independently with comparison tests, plus keyboard layout editor and profile persistence. Track parity case-by-case in [`docs/PRESET_COVERAGE.md`](PRESET_COVERAGE.md). Do not mark any provisional native effect as an exact official preset until visually and behaviorally validated.

## Effects parity: independent compositor foundation

**Merged [PR #13](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/13)**, squash SHA `72172a31a26e93f73e2e29e69144c24af6d13538`; Rust/Clippy, native GUI compile, Python and SVG-gallery CI green. Source GG 108.3 functional palette/timing/wavelength values remain distinct from our independently designed shader. `src/compositor.rs` defines reusable ordered scene recipes; effects are rendered offline and the 513-byte Feature serializer is unchanged. New tests cover layer order, blending/opacity, transforms, mask repetition and numerical color samples.

**Not yet accepted as full parity:** actual GG shader equations, per-preset layer order and alpha, source 6-row / Mini 5-row geometry, reactive heat envelopes and analog readback, real-world visual comparison. For fidelity tracking see [`docs/COMPOSITOR.md`](COMPOSITOR.md) and [`docs/FACTORY_PRESETS.md`](FACTORY_PRESETS.md).

## Factory reactive source-period envelopes (development milestone)

**[PR #14](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/14) merged** as `37a8ebb2363eb6715a4e7aeb956c1a0dd193cf9e` (Rust tests/Clippy, GUI compilation, Python and SVG gallery CI green). It builds independent **GG 108.3-derived** keypress-trigger profiles: `gg:reactiveRipple` (250ms), `gg:reactiveLine` (500ms) and `gg:reactiveSingleKey` (1250ms). Nominal periods come from the historical GG migration; expanding radial/row wavefront and exact-key fade curves are **new approximations**, not reproduced GG source shaders.

The pure Rust engine handles separately timed overlapping events, no-press baseline, invalid/future events, exact expiry and bounded fade. The daemon and GUI now choose event-history windows appropriate for **source period × profile speed**, fixing truncation for slow Single Key while preserving original working native ripple. The offline gallery evaluates samples relative to each source trigger duration.

**Software acceptance passed:** headless Rust unit tests, strict Clippy, GUI compile, Python diagnostic tests and SVG gallery. **Physical Mini and actual vendor behavior remain untested** while owner is remote. See [`docs/REACTIVE_FACTORY.md`](REACTIVE_FACTORY.md).

**Remaining work:** source-aperture timing and shader reconstruction, exact key geometry, analog Hall input, layered priority and full side-by-side GG preset parity. This implementation does **not** establish exact vendor equivalent behavior.

## Engineering quality gates

- Default commands must be read-only; HID writes require explicit owner opt-in.
- Match VID/PID, usage page and both report lengths before writing.
- Never truncate packets, silently pick a HID interface, or accept arbitrary firmware/actuation commands.
- No proprietary GG binaries, descriptors, plaintext, or recovered passphrase in the public repo.
- Run `cargo test --all-targets`, `cargo clippy` and Python unit tests in CI. Formatting CI check is pending; add `cargo fmt --check` in a subsequent cleanup.
- Hardware success needs the owner's observation; don't mark it done based on HID API return codes.
- Update `docs/HANDOVER.md` and this roadmap alongside every meaningful change.
