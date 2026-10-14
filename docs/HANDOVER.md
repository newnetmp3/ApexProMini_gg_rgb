# ApexProMini_gg_rgb — rolling development handover

**Updated:** 2026-10-14  
**Repository:** https://github.com/newnetmp3/ApexProMini_gg_rgb  
**Latest merged milestone:** PR #4, full-board effects and Prism preset catalog. **Current branch:** `fix/reactive-evdev-daemon`; targeted evdev input diagnostics and optional user service, tests/PR pending. **New owner feedback:** whole keyboard solid blue works; **keypress ripple did not trigger**. First physical full-board success; ripple still unverified.  
**Hardware:** Wired SteelSeries Apex Pro Mini `1038:161e`; owner runs Arch Linux, KDE Wayland.

## Verified milestones

1. USB control interface 1, HID page `0xffc0`, usage `1`; read-only Linux descriptor probe verifies report ID `0`, 65-byte Output and 513-byte Feature reports. `/dev/hidrawN` is not stable.
2. Existing upstream `steelseriesgg-rs` generic HID issue: extra zero report-ID and deduplicated APPLY fixed with a local patch. The owner confirmed 3/3 transport writes but **no physical LED change**. PID `161e` still falls back to generic one-zone RGB there.
3. GG 108.3 macOS and 121.0 Windows installers were supplied privately, along with extracted Windows `SteelSeriesEngine.exe`. Using the independently researched `steelpatcher` technique, passphrase extraction succeeded and encrypted Mini device definitions decrypted. Proprietary assets and secrets stay out of Git.
4. **New 2026-10-14 milestone:** extracted and decrypted inherited `base_apex_2022_wired.edevice` and `individually-lit-keyboard.edevice` from the owner's Mac package. Discovered official `direct_write` structure: Feature report of **513 bytes**, `00 40 NN` followed by up to **68 entries** of `[HID,R,G,B]`. The official fixed field has 68 entries; `clear_direct_write` is Output report of **65 bytes**, prefix `00 41`. RGB commands are now derived from official static definitions, not guessed.
5. Added independent Rust crate `apex-pro-mini-gg-rgb`, with pure serializers, strict HID descriptor and device checks, golden-byte unit tests, and executable `apex-mini-rgb`. The binary defaults to read-only inspection; `--dry-run` never opens a device; live `--write-key` and `--clear` require literal `--confirm-device-write`.
6. **2026-10-14 hardware verification:** owner confirmed **A red**, **S green**, and **`0x41` clear successfully restoring onboard lighting**. Owner subsequently confirmed **the entire keyboard turned blue** using `--effect solid`. This hardware-verifies whole-board 68-key static RGB frame output.
7. **2026-10-14 failure:** owner reported **keypress ripple does not occur** after blue full-board lighting worked. Root cause not yet measured: first reader depended on one manually supplied evdev interface and had no read-only keypress diagnostic. Do **not** mark reactive effects as working.

## Safety: do not conflate these conditions

- **Statically verified:** command `0x40` / `0x41` report formats, key slot count and HID report types.
- **Hardware observed:** correct endpoint/report lengths; generic `0x21` does not change Mini RGB; **A red, S green, `0x41` clear restore, and whole-keyboard solid blue**. **Ripple failed to react to presses**.
- **Unverified:** simultaneous **different colors** on multiple keys, animated wave/rainbow, keypress ripple, exact physical coordinates, preset parity, profiles and firmware variants. Single-color full-board frame is verified; ripple failure remains open.

## Current contents

- `src/lib.rs`: no-IO Rust packet encoding and report descriptor parser with tests.
- `src/main.rs`: guarded Linux discovery, opt-in direct write/clear, effect dispatch and dry run.
- `src/effects.rs`: native full-board frame rendering for solid, gradient, rainbow, wave, breathing, color-shift, sparkle, ripple and reactive.
- `src/input.rs`: Mini-only passive Linux evdev, now opens all matching real keyboard event nodes automatically, validates `input_event` keydown, filters repeats/releases, reports reader loss and counts unmapped codes only during diagnostics. No key grabs or persisted typing.
- `src/animate.rs`: bounded-rate animation with Ctrl+C/SIGTERM lighting restore; automatically selects Mini evdev readers. Adds read-only `--test-input --seconds 10` (prints key IDs only when requested) and read access status via `--list-inputs`.
- `docs/PRESET_COVERAGE.md`: nine provisional native effects plus **31 official GG 108.3 Prism preset identifiers**. Not all necessarily visible at once.
- `systemd/user/apex-mini-ripple.service`, `scripts/install-user-service.sh`, `docs/SERVICE.md`: optional, **disabled until owner enables**, user-level reactive lighting service with automatic restart upon input loss. Never requires running as root or installing proprietary software.
- `Cargo.toml`, `.github/workflows/rust.yml`: zero-external-dependency Rust build/tests/Clippy.
- `tools/hid_probe.py` + `tests/test_hid_probe.py`: original read-only Python probe.
- `tools/decrypt_descriptor_local.py`: private-only wrapper around independent steelpatcher parser.
- `docs/PROTOCOL.md`: verified static RGB serializer and execution instructions.
- `docs/SETUP.md`: Arch setup and safe test steps.
- `.gitignore`: vendor binaries, decrypted descriptors, captures, firmware and credentials excluded.
- `patches/steelseriesgg-rs-hid-report-fix.patch`: earlier locally verified upstream transport patch.

## Immediate next work

1. Finish CI and merge `fix/reactive-evdev-daemon`, update this file and roadmap with verified commit/PR and test outcomes.
2. Ask owner to run **read-only** `./target/release/apex-mini-rgb --list-inputs` and `./target/release/apex-mini-rgb --test-input --seconds 10`, tapping A, S and Space. Expect HID usages 4, 22 and 44. This isolates evdev event delivery from rendering; do not infer key events from successful static RGB alone.
3. If the diagnostic observes keydowns, test `--effect ripple --confirm-device-write` (automatically selected Mini event nodes) and verify the visible ring on the board. If no events, inspect selected input node(s), sysfs capabilities and seat ACLs; do not suggest `sudo` blindly.
4. Once ripple works, optionally install and enable the explicit `systemd --user` ripple daemon following `docs/SERVICE.md`. Its event stream is transient and unlogged. Check unplug/reconnect and SIGTERM restoring onboard lighting.
5. Continue exact SteelSeries GG default Prism preset parity using the 31-identifier catalog, with separate physical verification of animation parameters and reactive layout.

## Current fix branch — reactive evdev monitoring and user daemon

- **Known input:** real 68-key full board solid blue; ripple keypress detection failed.
- **Root-cause hypothesis (not yet verified):** wrong/idle manual evdev interface or missing evdev visibility; now reads all valid keyboard nodes and exposes a read-only keydown diagnostic.
- **New code:** `src/input.rs` multi-reader, strict wired Mini selection, keydown parser and reader-stop notification; `src/animate.rs` auto-input and bounded diagnostic; `src/main.rs` dispatch; optional service unit and installer.
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

## Rolling documentation convention

For each substantial code/research milestone: inspect actual repository state; implement and test; update this file and `docs/ROADMAP.md` in the same PR or commit. Include date, branch, exact verified results, tests and next step. Avoid claiming success based on speculative software/hardware behavior.

**Continuation prompt:**

> Continue `newnetmp3/ApexProMini_gg_rgb` using `docs/HANDOVER.md` and `docs/ROADMAP.md`; inspect the GitHub repo, implement the next verified milestone, test, and update both rolling documents.
