# ApexProMini_gg_rgb — rolling development handover

**Updated:** 2026-10-14  
**Repository:** https://github.com/newnetmp3/ApexProMini_gg_rgb  
**Latest merged factory geometry milestone:** [PR #15](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/15), squash SHA `40b40eddea874dca364a448e9bdca220903a09eb` — independent width-aware, ANSI-inspired key-unit geometry for **factory-style** Prism presets. New `src/layout.rs` locates all 68 Mini HID usages (64 primary + 4 layout/firmware alternatives), centers long Space/Shift/Backspace/Enter keys appropriately, maps the physical five rows to approximate historical six-row/22-column regions and improves factory ripple/line/heatmap origins. Offline SVG gallery uses proportional key widths. **Known-working native `ripple` retains its original coordinates**, the 513-byte HID serializer and systemd daemon are unchanged. No physical owner QA while remote, and the provisional geometry is **not** proven GG parity. See `docs/KEY_GEOMETRY.md`.  
**Latest merged milestone:** [PR #8](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/8), squash SHA `b53923e1228a7af9da0c3f5c226e1f402b8b7705`, documenting owner-verified active/enabled systemd user daemon with two Mini evdev readers. Rust and Python PR checks passed. **Owner's subsequent report: “IT WORKS!” in response to the ripple retest. Keypress-triggered ripple is now hardware-confirmed.** **The owner has now installed, enabled, and observed the systemd user service active and running for 4m42s** (user@1000; PID 561326); it detected the correct `0xffc0` 513-byte Feature/65-byte Output interface and started two Mini evdev readers. **User explicitly confirmed that keypress ripple works with the systemd service running**. Login persistence, physical USB reconnect, and clean stop/clear are separate outstanding QA gates.  
**Latest merged development milestone:** [PR #9](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/9), squash SHA `edf7e45a8082a87b6ebf3e5119afc96f30684248`. The native Rust desktop GUI, 14-effect prototype engine, strict saved profiles/per-key overrides and profile-aware single-writer systemd service are on `main`. **GUI compile, Rust tests/Clippy and Python CI passed; owner's KDE GUI/edited-profile hardware acceptance is still pending.**  
**Latest merged effects milestone:** [PR #10](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/10), squash SHA `8629738a38d39d867b8a5137b6fee9fd897e53c3`, with 31 factory-style historical selectors, 36 additional region variants, base/idle/reactive compositor, and analog heatmap color threshold lookup. CI passed, **but exact GG parity and physical QA remain pending**.  
**Latest merged remote QA milestone:** [PR #11](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/11), squash SHA `45d9bb363cfc5518c6659e2c07b93808abe522a3`, adds read-only 31-preset × three-phase SVG gallery and CI artifact. Rust tests, Clippy, optional GUI compilation, Python diagnostics and SVG artifact build passed. User remains remote; defer GUI/device QA.
**Latest merged effects milestone:** [PR #12](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/12), squash SHA `ca2deeca512a43a44a1dc08aaa65bcf5468b319c`, adds independent foreground/background/aperture scene rendering for West Coast, Haze and Warp Drive. All PR Rust, Clippy, GUI compilation, Python and SVG gallery checks passed. This is source-parameter-derived, with an **approximate aperture shader**, not exact SteelSeries GG parity. Owner remains remote; GUI/device QA deferred. Owner is remote, so **defer GUI/hardware QA** and concentrate on factory preset conversion. Actual GG 108.3 Prism SQL migrations were recovered offline. We found **30 complete source definitions**, and GG 106's `redPulse` name is a rename of existing `faze`. `src/factory.rs` implements all **31 historical identifiers as parameter-derived renderable previews**, not 31 equivalent vendor algorithms. A new `src/layers.rs` base/idle/reactive compositor and backward-compatible profile extensions are integrated with the daemon and GUI. **Hardware acceptance for these new factory modes is pending**. See `docs/FACTORY_PRESETS.md`.  
**Latest merged effects milestone:** [PR #13](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/13), squash SHA `72172a31a26e93f73e2e29e69144c24af6d13538`. Full ordered compositor with opacity, four blends, spatial transforms and apertures; West Coast, Haze and Warp Drive refactored into recipes. Headless Rust tests + strict Clippy, native GUI compile, Python diagnostics and SVG gallery CI all passed. **GG shader parity and live hardware QA remain unverified; user is still remote.**  
**Implementation summary:** `src/compositor.rs` introduces a pure ordered-scene renderer with Normal/Add/Multiply/Screen blends, opacity, origin/scale/rotation, spatial repetition/phase offsets, masked apertures and independent per-layer animation. `src/graphics.rs` now represents West Coast, Haze and Warp Drive as layer recipes using that engine, while preserving the existing Mini HID transport. Offline scene/motion/mask tests and `docs/COMPOSITOR.md` added. **These are new, independent approximations, NOT recovered vendor shader code or proven GG parity.** GUI and keyboard testing remain deferred at the user's request.  
**Latest merged reactive milestone:** [PR #14](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/14), squash SHA `37a8ebb2363eb6715a4e7aeb956c1a0dd193cf9e` (all PR Rust unit tests/Clippy, GUI compile, Python diagnostics and SVG gallery green): three distinct, independently modeled factory reactive modes, using original GG 108.3 trigger periods: Ripple 250ms, Line 500ms, Single Key 1250ms. New `src/reactive_factory.rs` isolates motion and input hit lifespan from the original native ripple, supports multiple overlapping hits, and supplies adaptive event-history timeouts for slow speed profiles (Single Key 12.5s at 0.1x; the old fixed 2.5s cut it off). CLI daemon, GUI simulated preview and headless SVG previews are integrated. **Factory spatial/fade curves remain approximations; user is remote, so physical testing remains deferred.** See `docs/REACTIVE_FACTORY.md`.  
**Hardware:** Wired SteelSeries Apex Pro Mini `1038:161e`; owner runs Arch Linux, KDE Wayland.

## Verified milestones

1. USB control interface 1, HID page `0xffc0`, usage `1`; read-only Linux descriptor probe verifies report ID `0`, 65-byte Output and 513-byte Feature reports. `/dev/hidrawN` is not stable.
2. Existing upstream `steelseriesgg-rs` generic HID issue: extra zero report-ID and deduplicated APPLY fixed with a local patch. The owner confirmed 3/3 transport writes but **no physical LED change**. PID `161e` still falls back to generic one-zone RGB there.
3. GG 108.3 macOS and 121.0 Windows installers were supplied privately, along with extracted Windows `SteelSeriesEngine.exe`. Using the independently researched `steelpatcher` technique, passphrase extraction succeeded and encrypted Mini device definitions decrypted. Proprietary assets and secrets stay out of Git.
4. **New 2026-10-14 milestone:** extracted and decrypted inherited `base_apex_2022_wired.edevice` and `individually-lit-keyboard.edevice` from the owner's Mac package. Discovered official `direct_write` structure: Feature report of **513 bytes**, `00 40 NN` followed by up to **68 entries** of `[HID,R,G,B]`. The official fixed field has 68 entries; `clear_direct_write` is Output report of **65 bytes**, prefix `00 41`. RGB commands are now derived from official static definitions, not guessed.
5. Added independent Rust crate `apex-pro-mini-gg-rgb`, with pure serializers, strict HID descriptor and device checks, golden-byte unit tests, and executable `apex-mini-rgb`. The binary defaults to read-only inspection; `--dry-run` never opens a device; live `--write-key` and `--clear` require literal `--confirm-device-write`.
6. **2026-10-14 hardware verification:** owner confirmed **A red**, **S green**, and **`0x41` clear successfully restoring onboard lighting**. Owner subsequently confirmed **the entire keyboard turned blue** using `--effect solid`. This hardware-verifies whole-board 68-key static RGB frame output.
7. **2026-10-14 failure and confirmed blocker:** Owner reports no keypress ripple; `--list-inputs` finds `/dev/input/event11` and `/dev/input/event12` (both wired Mini) but indicates **no read access**; `--test-input` fails on both with `Permission denied (os error 13)`. Input permissions are **the verified immediate blocker**. Any renderer-level issues remain untested and should not be ruled out until events can be read. Do **not** mark reactive effects working.

8. **2026-10-14 resolution — hardware verified:** after PR #6's device-scoped udev permissions fix and instructions, the owner responded **“IT WORKS!”** to the ripple test. Record the original no-access symptoms as historical diagnostic evidence. This confirms the previously missing **live keypress-triggered ripple is visibly working on the wired Mini**. The user has not separately confirmed precise ripple geometry, multiple simultaneous waves, daemon autostart, reconnection, or matching the official GG Reactive Ripple preset.

9. **Background service installation/start verified from systemd owner output (Oct 13, 2026, 23:14 EDT):** `apex-mini-ripple.service` is **loaded, enabled, active (running)** under `/user.slice/user-1000.slice/user@1000.service` with executable `%h/.local/bin/apex-mini-rgb`; uptime ~4m42s, 768 KB RAM (2.7 MB peak), two active Mini evdev readers and the expected 513-byte Feature / 65-byte Output reports on `/dev/hidraw7`. No startup error appears in the supplied logs. **This is service-liveness evidence, not separate confirmation of visible ripple while running via systemd.** Device node numbers are temporary.

10. **Background ripple hardware-confirmed in user daemon:** owner explicitly answered **“yes”** when asked whether pressing keys causes visible ripples while `apex-mini-ripple.service` is running. Combined with the earlier systemd status, this verifies service-owned keypress-reactive RGB (not only manual CLI). No reboot/unplug or stop/clear lifecycle test has been supplied yet.

11. **2026-10-14 factory catalog conversion (merged PR #10; Rust + GUI + Python CI passed):** parsed GG's Prism migration resources privately; **30 actual preset records** plus one newer `redPulse` rename (`faze`). Independent Rust factory interpreter includes authentic first-variant functional RGB palette stops, animation durations, signed spatial wavelength and phase category for all 31 historical identifiers, plus 68-key rendering tests. **Not a full copy of the vendor effects engine:** eight records have **36 additional masked layout variants now implemented as approximate row/column selections**; several records have backgrounds/apertures that are still not modeled faithfully. Added pure `base + idle + reactive` compositor and `version=1` profile fields without changing the Mini HID writer or the earlier default ripple behavior.

12. **2026-10-14 read-only remote gallery (PR #11 merged, all CI passed):** Added headless `src/bin/prism_preview.rs`, generating an SVG visual test sheet for all 31 historic factory selectors with three synthetic animation phases each; keypress modes use simulated A-key hits, **never actual input**. A new CI job uploads the output as `apex-mini-prism-factory-previews`. No HID, evdev, GUI, proprietary assets or device access.

13. **2026-10-14 multi-graphics compositor stage (PR #12 merged; all CI green):** New `src/graphics.rs` handles separate gradient foreground, background and bounded spatial aperture layers for three complex presets (West Coast, Haze, Warp Drive) without changing any driver writes. Exact vendor shader masks remain unrecovered; this is a partial, testable structural port.

14. **2026-10-14 independent compositing milestone ([PR #13](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/13) merged, all CI green):** reusable ordered z-stack, four channel blend modes, alpha, origin/rotation/scale, spatial repetitions, phase offsets and mask options. West Coast/Haze/Warp Drive now use declarative scene recipes with source-derived RGB/period fields, but **not vendor mask/opacity math**. Full 68-key offline frame tests and numerical color fixtures; underlying HID/proven ripple untouched. See `docs/COMPOSITOR.md`.

15. **2026-10-14 factory reactive timing milestone ([PR #14](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/14) merged, all CI green):** Factory Ripple/Line/Single Key each have explicit hit envelopes tied to source nominal period, radial/row/exact-key isolation, bounded overlap, and invalid/expired filtering; `Stack::key_history_seconds` prevents truncation for slow GG factory presets without changing native ripple's earlier 2.5s event retention. GUI simulated reactive-overlay preview and offline SVG phase-relative time samples updated. Only functional periods are source verified; real GG curves remain to be compared.

16. **2026-10-14 factory-only width-aware geometry milestone ([PR #15](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/15) merged; all CI green):** Replaced equal-slot positioning for `gg:` factory-mode RGB with a five-row 15-unit physical model, preserving all 68 official HID codes via 64 primary and four optional aliases. Factory reactive, heatmap, region mask and headless gallery renderers share one provisional geometry. Native owner-verified ripple and hardware protocol are unchanged. Unit tests ensure known-code coverage, row widths, positional bounds, spacebar centering, source-grid mapping and no key-loss. Accurate device/ISO calibration must await owner reference data or hardware access.

## Safety: do not conflate these conditions

- **Statically verified:** command `0x40` / `0x41` report formats, key slot count and HID report types.
- **Hardware observed:** correct endpoint/report lengths; generic `0x21` does not change Mini RGB; **A red, S green, `0x41` clear restore, and whole-keyboard solid blue**. **Ripple initially failed due to evdev EACCES; owner now confirms it works after the permission-fix milestone**.
- **Hardware confirmed:** per-key A red, S green; full-board solid blue; `0x41` clear; **keypress-triggered ripple after fixing evdev access**.
- **Not yet verified:** distinct simultaneous colors, other animated presets, geometric fidelity of the ripple, exact GG Reactive Ripple parity, service persistence across login/unplug/replug, reliable stop/clear, profiles, and firmware variants.

## Current contents

- `src/lib.rs`: no-IO Rust packet encoding and report descriptor parser with tests.
- `src/main.rs`: guarded Linux discovery, opt-in direct write/clear, effect dispatch and dry run.
- `src/effects.rs`: native full-board frame rendering for solid, gradient, rainbow, wave, breathing, color-shift, sparkle, ripple and reactive.
- `src/input.rs`: Mini-only passive Linux evdev, now opens all matching real keyboard event nodes automatically, validates `input_event` keydown, filters repeats/releases, reports reader loss and counts unmapped codes only during diagnostics. No key grabs or persisted typing.
- `src/animate.rs`: bounded-rate animation with Ctrl+C/SIGTERM lighting restore; automatically selects Mini evdev readers. Adds read-only `--test-input --seconds 10` (prints key IDs only when requested) and read access status via `--list-inputs`.
- `docs/PRESET_COVERAGE.md`: nine provisional native effects plus **31 official GG 108.3 Prism preset identifiers**. Not all necessarily visible at once.
- `systemd/user/apex-mini-ripple.service`, `scripts/install-user-service.sh`, `docs/SERVICE.md`: optional user-level reactive lighting service with automatic restart after input loss. **Owner has now enabled and started it successfully**; **owner has now confirmed visible ripple while daemonized**. Reboot/reconnect and controlled stop remain unverified. No root daemon or proprietary software.
- `Cargo.toml`, `.github/workflows/rust.yml`: zero-external-dependency Rust build/tests/Clippy.
- `tools/hid_probe.py` + `tests/test_hid_probe.py`: original read-only Python probe.
- `tools/decrypt_descriptor_local.py`: private-only wrapper around independent steelpatcher parser.
- `docs/PROTOCOL.md`: verified static RGB serializer and execution instructions.
- `docs/SETUP.md`: Arch setup and safe test steps.
- `.gitignore`: vendor binaries, decrypted descriptors, captures, firmware and credentials excluded.
- `patches/steelseriesgg-rs-hid-report-fix.patch`: earlier locally verified upstream transport patch.

## Latest merged milestone — PR #9 native GUI and saved effect profiles

- **Owner-verified baseline:** manual RGB per-key and whole-board solid, direct-mode restore, keypress ripple, **and keypress ripple from the background `systemd --user` service**.
- **New engine:** `src/profile.rs` strictly validates versioned `key=value` RGB profiles, including speed, brightness, frame rate and per-key HID overrides; saves atomically. No keystrokes or GG proprietary assets stored.
- **Updated daemon integration:** `systemd/user/apex-mini-ripple.service` reads `~/.config/apex-mini-rgb/profile.conf` via `--profile`. `scripts/install-user-service.sh` seeds only missing profiles, atomically replaces in-use executables, and reloads the user unit without automatically starting it.
- **Native GUI:** optional `eframe` feature and `apex-mini-gui` binary (`src/gui.rs`) with **fourteen** selectable native effect algorithms (original nine plus comet, rainbow split, prismatic sweep, heatmap and reactive line), live simulated key-map preview, per-key RGB painting, color pickers, speed/brightness/FPS, save/apply and service start/stop/enable controls. It does **not** write HID itself; the user daemon remains the sole writer.
- **GUI install:** `bash scripts/install-gui.sh` compiles GUI, upgrades the profile-aware user daemon and installs a KDE-compatible `.desktop` launcher. No root daemon. `docs/GUI.md` specifies QA steps and limitations.
- **Known parity gap:** 31 SteelSeries GG migration preset identifiers are indexed and fourteen native algorithms exist, but most factory parameters and effects are not yet faithfully reproduced. Native presets are independent prototypes, not a faithful port yet.
- **Verification:** **PR #9 GitHub Actions passed** for headless Rust unit tests + Clippy, optional Linux/Wayland/X11 GUI compile, and Python diagnostics. Owner must still validate actual Plasma window and profile application on real hardware.

## Immediate next work — effects only while owner is remote

1. **PR #13 merged** as `72172a31a26e93f73e2e29e69144c24af6d13538`; all Rust unit tests, strict Clippy, Python diagnostics, native GUI compilation and offline SVG gallery checks green.
2. **Implemented now:** pure ordered color-layer compositor with four blend modes, alpha, origin/scale/rotation, repeats and apertures. Three compound scenes migrated with unchanged known-safe HID transport.
3. **PR #14 merged and CI green:** independent factory reactive envelopes for `gg:reactiveRipple`, `gg:reactiveLine`, `gg:reactiveSingleKey` (source periods 250/500/1250ms), bounded overlap, phase-relative SVG sampling, 12.5s slow event-history preservation in daemon and GUI. Native owner-confirmed ripple is unaffected.
4. **After this PR:** recover original GG reactive/aperture/mask movement and factory layer order/opacity from authorized sample observations; add time/key/color golden fixtures. Do not call independent Gaussian/cosine masks vendor equivalents.
5. **PR #15 merged and CI green:** width-aware five-row/15u Mini factory grid replaces equispaced centers in `gg:` modes and read-only SVG. **Next:** verify exact vendor LED coordinates/ISO variations and all 36 historical region masks against authorized reference data; owner hardware QA deferred.
6. Continue exact reactive falloff, multi-hit policies and analog heatmap design **without unverified Hall sensor reads or arbitrary HID commands**.
7. Owner is remote; don't request GUI or keyboard access. Review offline SVG gallery built in PR #11; postpone physical preset-by-preset acceptance until owner returns.

## Latest merged permission fix — PR #6, active-seat Mini-only udev rule

- `udev/71-apex-pro-mini-input.rules`: matches `SUBSYSTEM=input`, `event*`, `ID_INPUT_KEYBOARD=1`, USB VID:PID `1038:161e`; sets `MODE=0660` and `TAG+=uaccess` before `73-seat-late.rules`. Limits keypress access to logged-in active seat ACL rather than the global `input` group.
- `scripts/install-input-permissions.sh`: explicit local installer requiring `sudo` only for one rule + udev reload; **does not start the service**. Reconnect keyboard after installation.
- `tests/test_udev_rule.py`: assertions preventing broad rules or ordering regressions.
- **PR #6 merged:** Rust/Clippy and Python test CI passed. **Owner follow-up: visible ripple now works.** The exact diagnostic counts were not supplied; do not invent per-key event observations. Service autostart/reconnection is a separate pending test.

## Latest merged fix — PR #5 reactive evdev monitoring and user daemon

- **Known input:** real 68-key full board solid blue; ripple keypress detection failed.
- **Confirmed 2026-10-14 permission failure:** the PR #5 diagnostic identifies two Mini event nodes but cannot open either (`EACCES`). No keydown events can reach the renderer until input permissions are addressed.
- **New code merged:** `src/input.rs` multi-reader, strict wired Mini selection, keydown parser and reader-stop notification; `src/animate.rs` auto-input and bounded diagnostic; `src/main.rs` dispatch; optional service unit and installer. **Rust/Clippy and Python PR CI green.**
- **Safety:** daemon is opt-in; no input grabs, no key contents logged to service output, no root process, explicit HID write confirmation in service ExecStart; service stops with known `0x41` clear.

## Latest merged milestone — PR #4

- Reusable pure Rust full-key frame renderer with nine provisional effects, including spatial ripple and local keypress reactive overlays.
- 68-entry RGB frame updates over already verified `0x40` Feature transport.
- Linux evdev input reader on a specifically validated USB 1038:161e keyboard node, no input grabs/logs/network and no compositor-specific hook.
- Animation FPS/speed/brightness bounds; SIGINT/SIGTERM handling; `0x41` restoration when effects stop or input disconnects.
- Offline renderer/input/CLI tests added, Rust unit tests and Clippy plus Python checks passed. **These new effect behaviors have not been verified on the owner's physical keyboard.**
- Official GG 108.3 Mac Prism migration research identified **31 distinct preset identifier keys** with the catalog and gaps documented in `docs/PRESET_COVERAGE.md`. These are not necessarily 31 simultaneously visible menu choices; no exact preset has been marked fully matched.



## Completed PR and CI history

- **2026-10-14 PR #1:** Repository, initial handover, Python read-only HID probe and CI.
- **2026-10-14 PR #2:** Official Mini wired `0x40` direct RGB and `0x41` clear serialization, guarded independent Rust CLI, unit tests, Rust CI and updated setup/research. Branch tests and Clippy passed; **A-key red `0x40` lighting was subsequently confirmed by the owner**. Other features remain unverified. Squash SHA `2ccb9f67d63d`.
- **2026-10-14 PR [#3](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/3):** Owner confirmed physical **A key turned red**; docs merged to `main` as `71fc653c701fad5f63daa23157c1e09d6d3a03cf`. **Follow-up hardware checks:** `0x41` clear restored onboard lighting and **S key turned green**; full-key frames and live presets still awaiting tests.

- **2026-10-14 PR [#4](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/4):** Nine provisional native RGB effect renderers, full-key static frame, physically scoped Linux evdev keypress ripple/reactive input, bounded animation loop and Ctrl+C clear. All PR branch CI green; **new animations/ripple are not yet hardware verified**. Squash commit `570ccddee9c254bd2de28c7e9b1fb62ece192756`. Cataloged 31 official GG Prism preset identifiers for future exact behavior matching.

- **2026-10-14 PR [#5](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/5):** Fixed the evdev reader's dependence on one manual node; added read-only key event diagnostic, automatic multi-interface monitoring, reader disconnect handling, Linux keycode corrections, and optional disabled-by-default user-level systemd ripple daemon. PR checks passed; **ripple subsequently became hardware-confirmed after PR #6's scoped ACL fix**. Squash SHA `608c668e580ec154065ace5810723f81e9a8bcbb`.

- **2026-10-14 PR [#6](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/6):** Scoped systemd-logind udev `uaccess` ACL for Mini event nodes and opt-in installer, with regression tests and instructions. Squash SHA `e22b2c4da690449d5a238feb5b5a6ac2a96999e8`. CI green on PR. Owner subsequently confirmed visible ripple working; systemd daemon and reconnect handling still unverified.

- **2026-10-14 hardware validation (following PR #6):** Owner replied **“IT WORKS!”** after the Mini-specific udev access fix and ripple retest. This closes the missing-keypress ripple blocker, not the GG preset-parity or background service acceptance gates.

- **2026-10-14 PR [#7](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/7):** Recording owner-confirmed keypress ripple operation following PR #6's permissions fix. Rust, Clippy and Python PR checks passed; squash SHA `045b0845561e222983aa9b9db79b77cc912bb7f4`. Service lifecycle and exact GG preset parity remain pending.

- **2026-10-13 hardware/systemd session evidence:** user installed `scripts/install-user-service.sh`, explicitly enabled the user service, and showed an active PID 561326 for 4m42s. Successful endpoint discovery and two matching Mini input readers; no startup errors in supplied status. Visible per-key animation while daemonized and service lifecycle behavior are **pending**.

- **2026-10-13 PR [#8](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/8):** Recorded owner-confirmed background user daemon startup/enabled/running, stable 4m42s status and Mini input endpoints; Rust and Python checks passed. Squash SHA `b53923e1228a7af9da0c3f5c226e1f402b8b7705`. Visible ripple under the daemon, reconnect and stop-cleanup require follow-up tests.

- **2026-10-13 user daemon validation:** Owner confirmed visible keypress ripple while the enabled systemd user service was running, resolving the separate daemon-animation acceptance gate. Reconnect/logout/stop not yet tested.
- **2026-10-14 PR [#9](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/9):** Native eframe/egui GUI, 14 independent effect modes, per-key RGB paint overrides, validated on-disk profile and profile-aware single-writer daemon. Linux GUI compile + Rust tests/Clippy + Python CI passed. Squash SHA `edf7e45a8082a87b6ebf3e5119afc96f30684248`. GUI/runtime **hardware acceptance is pending**.

- **2026-10-14 factory conversion (in progress, branch `feature/prism-factory-presets-dataset`):** recovered the 30 complete Prism preset records and the GG106 redPulse/faze rename; authored 31 historical selectors (`gg:<name>`), local palette/timing previews and a pure layered compositor. CI and merge pending. No owner's live keyboard available for these newly added effects.

- **2026-10-14 regional preset stage (pending PR):** Added `src/regions.rs` with 36 additional masked source variants from the owner's GG 108.3 migrations. Factory rendering now selects by approximate Mini row/column and falls back to unmasked base. Exact position calibration and vendor layer compositing remain pending.

- **2026-10-14 PR [#10](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/10):** Source-parameter-derived previews for 30 complete historical GG factory effects plus the redPulse/faze rename, 36 regional variants, base/idle/reactive layers, and offline heatmap analog thresholds; all PR CI passed. Squash SHA `8629738a38d39d867b8a5137b6fee9fd897e53c3`. Strict vendor parity not yet achieved.
- **2026-10-14 PR [#11](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/11):** Headless SVG gallery for 31 modes × three phases plus simulated reactive keypress frames; all CI passed, artifact verified on Actions. Squash SHA `45d9bb363cfc5518c6659e2c07b93808abe522a3`. No physical input or HID writes.

- **2026-10-14 PR [#12](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/12):** Source-parameter-derived foreground/background/aperture implementation for West Coast, Haze and Warp Drive via `src/graphics.rs`. GUI compilation, headless Rust/Clippy, Python diagnostics and SVG gallery CI all passed. Squash SHA `ca2deeca512a43a44a1dc08aaa65bcf5468b319c`. Proprietary shader math remains unverified; no owner hardware QA while remote.

- **2026-10-14 [PR #13](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/13):** New `src/compositor.rs` with ordered graphic layers, four blend modes, alpha, spatial transforms and repeatable masks; three compound scenes migrated. Rust/Clippy, GUI compile, Python and SVG-gallery CI passed. Squash SHA `72172a31a26e93f73e2e29e69144c24af6d13538`. Independent model, not vendor shader parity.

- **2026-10-14 [PR #14](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/14):** Dedicated factory Ripple/Line/Single Key pure Rust reactive envelopes using source nominal durations, adaptive 12.5s slow-key history, GUI/daemon integration and phase-relative headless gallery. Rust tests, Clippy, GUI compile, Python diagnostics, gallery CI all passed; merge SHA `37a8ebb2363eb6715a4e7aeb956c1a0dd193cf9e`. Actual GG wave masks still approximate; device parity QA deferred.

- **2026-10-14 [PR #15](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/15):** Width-aware key centers and 68-HID coverage in `src/layout.rs`, integrated into factory renderers/reactives and read-only gallery, **not** owner-confirmed native ripple. Rust tests/Clippy, optional GUI compilation, Python diagnostics and SVG-gallery CI all passed; squash SHA `40b40eddea874dca364a448e9bdca220903a09eb`. Exact vendor physical/ISO and 22-column region calibration remains unverified. See `docs/KEY_GEOMETRY.md`.

## Rolling documentation convention

For each substantial code/research milestone: inspect actual repository state; implement and test; update this file and `docs/ROADMAP.md` in the same PR or commit. Include date, branch, exact verified results, tests and next step. Avoid claiming success based on speculative software/hardware behavior.

**Continuation prompt:**

> Continue `newnetmp3/ApexProMini_gg_rgb` using `docs/HANDOVER.md` and `docs/ROADMAP.md`; inspect the GitHub repo, implement the next verified milestone, test, and update both rolling documents.
