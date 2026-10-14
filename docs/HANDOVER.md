# ApexProMini_gg_rgb — rolling development handover

**Updated:** 2026-10-14  
**Repository:** https://github.com/newnetmp3/ApexProMini_gg_rgb  
**Latest merged work:** PR [#2](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/2), squash commit `2ccb9f67d63ddbd15b7dcc326dbefde505c0386d`, merged into `main` after Rust tests/Clippy and Python tests passed on the feature branch. Recheck main CI before the next milestone.  
**Hardware:** Wired SteelSeries Apex Pro Mini `1038:161e`; owner runs Arch Linux, KDE Wayland.

## Verified milestones

1. USB control interface 1, HID page `0xffc0`, usage `1`; read-only Linux descriptor probe verifies report ID `0`, 65-byte Output and 513-byte Feature reports. `/dev/hidrawN` is not stable.
2. Existing upstream `steelseriesgg-rs` generic HID issue: extra zero report-ID and deduplicated APPLY fixed with a local patch. The owner confirmed 3/3 transport writes but **no physical LED change**. PID `161e` still falls back to generic one-zone RGB there.
3. GG 108.3 macOS and 121.0 Windows installers were supplied privately, along with extracted Windows `SteelSeriesEngine.exe`. Using the independently researched `steelpatcher` technique, passphrase extraction succeeded and encrypted Mini device definitions decrypted. Proprietary assets and secrets stay out of Git.
4. **New 2026-10-14 milestone:** extracted and decrypted inherited `base_apex_2022_wired.edevice` and `individually-lit-keyboard.edevice` from the owner's Mac package. Discovered official `direct_write` structure: Feature report of **513 bytes**, `00 40 NN` followed by up to **68 entries** of `[HID,R,G,B]`. The official fixed field has 68 entries; `clear_direct_write` is Output report of **65 bytes**, prefix `00 41`. RGB commands are now derived from official static definitions, not guessed.
5. Added independent Rust crate `apex-pro-mini-gg-rgb`, with pure serializers, strict HID descriptor and device checks, golden-byte unit tests, and executable `apex-mini-rgb`. The binary defaults to read-only inspection; `--dry-run` never opens a device; live `--write-key` and `--clear` require literal `--confirm-device-write`.
6. **No hardware command has yet been tested from this new crate**. Source and public docs contain no proprietary GG files.

## Safety: do not conflate these conditions

- **Statically verified:** command `0x40` / `0x41` report formats, key slot count and HID report types.
- **Hardware observed:** correct endpoint/report lengths; generic `0x21` does not change Mini RGB.
- **Unverified:** whether directly sending the spec-derived `0x40` command changes visible LEDs, and any session initialization, profile or timing requirements. Do not mark milestone 6 complete until owner reports actual illumination.

## Current contents

- `src/lib.rs`: no-IO Rust packet encoding and report descriptor parser with tests.
- `src/main.rs`: guarded Linux discovery, opt-in direct write/clear and dry run.
- `Cargo.toml`, `.github/workflows/rust.yml`: zero-external-dependency Rust build/tests/Clippy.
- `tools/hid_probe.py` + `tests/test_hid_probe.py`: original read-only Python probe.
- `tools/decrypt_descriptor_local.py`: private-only wrapper around independent steelpatcher parser.
- `docs/PROTOCOL.md`: verified static RGB serializer and execution instructions.
- `docs/SETUP.md`: Arch setup and safe test steps.
- `.gitignore`: vendor binaries, decrypted descriptors, captures, firmware and credentials excluded.
- `patches/steelseriesgg-rs-hid-report-fix.patch`: earlier locally verified upstream transport patch.

## Immediate next work

1. PR #2 merged with Rust test + Clippy and Python test success on its branch. Check final `main` workflow status and follow-up formatting compliance.
2. Have owner clone/pull and run read-only `cargo run --` and `cargo run -- --dry-run A ff0000`.
3. With owner's explicit agreement, test `cargo run --release -- --write-key A ff0000 --confirm-device-write` and then `cargo run --release -- --clear --confirm-device-write`, reporting observed LED changes separately from successful USB submission.
4. If RGB works, expand to additional keys/colors and integrate the Mini driver into `steelseriesgg-rs`. If not, investigate GG session initialization/capture and firmware state; never fuzz random commands.
5. Continue with profile effects only after milestone 6 is observed on hardware.

## Completed PR and CI history

- **2026-10-14 PR #1:** Repository, initial handover, Python read-only HID probe and CI.
- **2026-10-14 PR #2:** Official Mini wired `0x40` direct RGB and `0x41` clear serialization, guarded independent Rust CLI, unit tests, Rust CI and updated setup/research. Branch tests and Clippy passed; hardware response remains **unverified**. Squash SHA `2ccb9f67d63d`.

## Rolling documentation convention

For each substantial code/research milestone: inspect actual repository state; implement and test; update this file and `docs/ROADMAP.md` in the same PR or commit. Include date, branch, exact verified results, tests and next step. Avoid claiming success based on speculative software/hardware behavior.

**Continuation prompt:**

> Continue `newnetmp3/ApexProMini_gg_rgb` using `docs/HANDOVER.md` and `docs/ROADMAP.md`; inspect the GitHub repo, implement the next verified milestone, test, and update both rolling documents.
