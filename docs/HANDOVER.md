# ApexProMini_gg_rgb — rolling development handover

**Updated:** 2026-10-14  
**Repository:** https://github.com/newnetmp3/ApexProMini_gg_rgb  
**Last merged hardware milestone:** PR #3, A-key red verified. **Current development:** `feature/multikey-static-rgb` (PR pending); native full-keyboard effects engine and passive Linux keypress input added. Recheck current GitHub main/PR and CI before continuing.  
**Hardware:** Wired SteelSeries Apex Pro Mini `1038:161e`; owner runs Arch Linux, KDE Wayland.

## Verified milestones

1. USB control interface 1, HID page `0xffc0`, usage `1`; read-only Linux descriptor probe verifies report ID `0`, 65-byte Output and 513-byte Feature reports. `/dev/hidrawN` is not stable.
2. Existing upstream `steelseriesgg-rs` generic HID issue: extra zero report-ID and deduplicated APPLY fixed with a local patch. The owner confirmed 3/3 transport writes but **no physical LED change**. PID `161e` still falls back to generic one-zone RGB there.
3. GG 108.3 macOS and 121.0 Windows installers were supplied privately, along with extracted Windows `SteelSeriesEngine.exe`. Using the independently researched `steelpatcher` technique, passphrase extraction succeeded and encrypted Mini device definitions decrypted. Proprietary assets and secrets stay out of Git.
4. **New 2026-10-14 milestone:** extracted and decrypted inherited `base_apex_2022_wired.edevice` and `individually-lit-keyboard.edevice` from the owner's Mac package. Discovered official `direct_write` structure: Feature report of **513 bytes**, `00 40 NN` followed by up to **68 entries** of `[HID,R,G,B]`. The official fixed field has 68 entries; `clear_direct_write` is Output report of **65 bytes**, prefix `00 41`. RGB commands are now derived from official static definitions, not guessed.
5. Added independent Rust crate `apex-pro-mini-gg-rgb`, with pure serializers, strict HID descriptor and device checks, golden-byte unit tests, and executable `apex-mini-rgb`. The binary defaults to read-only inspection; `--dry-run` never opens a device; live `--write-key` and `--clear` require literal `--confirm-device-write`.
6. **2026-10-14 hardware verification:** the owner confirmed **A red**, then **`0x41` clear successfully restored onboard lighting**, then **S green**. This validates two independent LED locations/colors and direct-mode exit. Multi-key frames and animations still require observation. Proprietary device files are never committed.

## Safety: do not conflate these conditions

- **Statically verified:** command `0x40` / `0x41` report formats, key slot count and HID report types.
- **Hardware observed:** correct endpoint/report lengths; generic `0x21` does not change Mini RGB; **A red, S green, and `0x41` clear restoring onboard lighting**.
- **Unverified:** simultaneous multi-key colors, full-frame static lighting, live animations, evdev keypress ripple, exact physical coordinate fidelity, GG preset parity, profile interaction and firmware variants. Milestone 6 is partially hardware-verified.

## Current contents

- `src/lib.rs`: no-IO Rust packet encoding and report descriptor parser with tests.
- `src/main.rs`: guarded Linux discovery, opt-in direct write/clear, effect dispatch and dry run.
- `src/effects.rs`: native full-board frame rendering for solid, gradient, rainbow, wave, breathing, color-shift, sparkle, ripple and reactive.
- `src/input.rs`: non-grabbing Linux evdev, scoped to the wired Mini, only in-memory keydown codes.
- `src/animate.rs`: bounded-rate animation and Ctrl+C/SIGTERM restore, opt-in write and explicit USB-input selection.
- `docs/PRESET_COVERAGE.md`: nine provisional native effects plus **31 official GG 108.3 Prism preset identifiers** extracted from bundled database migration SQL. The identifiers are not necessarily 31 simultaneously available user-facing presets; every match still needs GG parameter/visual validation.
- `Cargo.toml`, `.github/workflows/rust.yml`: zero-external-dependency Rust build/tests/Clippy.
- `tools/hid_probe.py` + `tests/test_hid_probe.py`: original read-only Python probe.
- `tools/decrypt_descriptor_local.py`: private-only wrapper around independent steelpatcher parser.
- `docs/PROTOCOL.md`: verified static RGB serializer and execution instructions.
- `docs/SETUP.md`: Arch setup and safe test steps.
- `.gitignore`: vendor binaries, decrypted descriptors, captures, firmware and credentials excluded.
- `patches/steelseriesgg-rs-hid-report-fix.patch`: earlier locally verified upstream transport patch.

## Immediate next work

1. Verify **all Rust unit tests + Clippy + Python tests** on the current feature branch; open/merge PR after green checks. Record final commit and actual CI result.
2. Owner tests `--preview-effect wave` (read-only), then opt-in `--effect solid --color 2200ff --confirm-device-write`, followed by Ctrl+C. Confirm whether **all physical key LEDs change and onboard mode restores**.
3. Owner uses `--list-inputs` to find an exact physical Mini `/dev/input/eventN`, then opt-in `--effect ripple --input /dev/input/eventN --accent ff5500 --confirm-device-write`. Confirm visible rings from letter keypresses; investigate seat ACL/udev if event open fails (do not use sudo blindly).
4. Validate effects and approximate key coordinates on the real keyboard. Fix any non-responsive mapped keys, timing/tearing or degraded input behavior, and update docs.
5. **Long-term GG parity:** inventory real built-in GG Prism preset names, parameters, gradients, idle/reactive modes from user-owned GG installation, compare against native renders, and add missing effects systematically. Track each in `docs/PRESET_COVERAGE.md`. Add GUI/editor and stored profiles after core effects stabilize.

## Current work in progress (feature/multikey-static-rgb)

- Reusable pure Rust full-key frame renderer with nine provisional effects, including spatial ripple and local keypress reactive overlays.
- 68-entry RGB frame updates over already verified `0x40` Feature transport.
- Linux evdev input reader on a specifically validated USB 1038:161e keyboard node, no input grabs/logs/network and no compositor-specific hook.
- Animation FPS/speed/brightness bounds; SIGINT/SIGTERM handling; `0x41` restoration when effects stop or input disconnects.
- Offline renderer/input/CLI tests added. **These new effect behaviors have not been verified on the owner's physical keyboard.**



## Completed PR and CI history

- **2026-10-14 PR #1:** Repository, initial handover, Python read-only HID probe and CI.
- **2026-10-14 PR #2:** Official Mini wired `0x40` direct RGB and `0x41` clear serialization, guarded independent Rust CLI, unit tests, Rust CI and updated setup/research. Branch tests and Clippy passed; **A-key red `0x40` lighting was subsequently confirmed by the owner**. Other features remain unverified. Squash SHA `2ccb9f67d63d`.
- **2026-10-14 PR [#3](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/3):** Owner confirmed physical **A key turned red**; docs merged to `main` as `71fc653c701fad5f63daa23157c1e09d6d3a03cf`. **Follow-up hardware checks:** `0x41` clear restored onboard lighting and **S key turned green**; full-key frames and live presets still awaiting tests.

## Rolling documentation convention

For each substantial code/research milestone: inspect actual repository state; implement and test; update this file and `docs/ROADMAP.md` in the same PR or commit. Include date, branch, exact verified results, tests and next step. Avoid claiming success based on speculative software/hardware behavior.

**Continuation prompt:**

> Continue `newnetmp3/ApexProMini_gg_rgb` using `docs/HANDOVER.md` and `docs/ROADMAP.md`; inspect the GitHub repo, implement the next verified milestone, test, and update both rolling documents.
