# ApexProMini GG RGB for Linux

Native Linux research and driver-development workspace for the **wired SteelSeries Apex Pro Mini (USB `1038:161e`)**, using verified HID observations and local, legally obtained SteelSeries GG resources.

**Current status:** **First hardware success:** the wired Mini's A key **turned red** using the new Linux Rust `0x40` / 513-byte Feature command (owner-verified 2026-10-14). The `0x41` / 65-byte Output clear packet is implemented but **not yet confirmed working on the physical keyboard**; neither are other keys, multiple simultaneous colors, effects, or profiles.

## Quick start (read-only)

On Arch Linux, clone this repository and inspect the connected keyboard without writing to it:

```bash
git clone git@github.com:newnetmp3/ApexProMini_gg_rgb.git
cd ApexProMini_gg_rgb
python3 tools/hid_probe.py
python3 -m unittest discover -s tests -v
```

Expected *observed* control endpoint on the owner's machine: usage page `0xFFC0`, interface 1, report ID 0, **65-byte Output** and **513-byte Feature** reports. The `/dev/hidrawN` path can change after reconnecting the keyboard. The probe discovers it dynamically and **never sends HID commands**.

## Experimental native Rust RGB test (strictly opt-in)

The new standalone zero-dependency Rust crate includes the official Mini command serializer and checks the exact HID vendor ID, product ID, vendor usage page, and report sizes before opening a device for writes.

```bash
cargo test --all-targets
cargo build --release

# Safe, read-only checks:
./target/release/apex-mini-rgb
./target/release/apex-mini-rgb --dry-run A ff0000

# Optional owner-approved experimental hardware test (A key only):
./target/release/apex-mini-rgb --write-key A ff0000 --confirm-device-write
./target/release/apex-mini-rgb --clear --confirm-device-write
```

The two final commands **change RGB state**; the A-key color write has been verified on real hardware, but clear/direct-mode restoration still needs confirmation. Run them only after reviewing read-only output and accepting that lighting may need a keyboard reconnect or native profile reset. There is no firmware, macro, or actuation functionality in this crate. The `clear` command restores control to onboard lighting if the firmware honors its official command; a USB success message alone does not prove it.

See [`docs/PROTOCOL.md`](docs/PROTOCOL.md) for the recovered frame layout and [`docs/ROADMAP.md`](docs/ROADMAP.md) for acceptance criteria.

## Source repositories

- [steelseriesgg-rs](https://github.com/Ven0m0/steelseriesgg-rs): native Rust GG replacement; generic transport patch preserved in `patches/`.
- [steelpatcher](https://github.com/hurryman2212/steelpatcher): independent reference implementation for decrypting SteelSeries `.edevice` descriptors locally.
- Official SteelSeries GG installer files, and decrypted `.edevice` descriptors, **must be supplied locally by each developer** and must not be committed to this public repository.

## Reproducible local descriptor analysis

The uploader provided Windows GG 121.0.0 `SteelSeriesEngine.exe` and macOS GG 108.3.0 `.edevice` assets. The descriptor-password extraction described by steelpatcher **worked** on the Windows Engine; both the wired Mini specification and its shared Mini lighting definition were decrypted successfully. This does not verify the resulting USB command bytes.

Follow [`docs/RESEARCH.md`](docs/RESEARCH.md) and [`docs/SETUP.md`](docs/SETUP.md) for the precise evidence and private, local-only workflow.

## Roadmap and handover

- [`docs/ROADMAP.md`](docs/ROADMAP.md): staged scope with verified progress and acceptance tests.
- [`docs/HANDOVER.md`](docs/HANDOVER.md): **rolling handover**. Update it, the roadmap, and tests with each meaningful milestone; keep the latest known blockers and next action explicit.
- [`docs/PROTOCOL.md`](docs/PROTOCOL.md): independent observations versus unverified hypotheses.

## Safety and redistribution

**No device writes by default.** No firmware flashing, HID command fuzzing, or changing actuation settings. Prototypes must be explicitly opt-in, validated for `1038:161e`, and backed by reliable packet evidence. Do not distribute vendor installers, firmware, raw or decrypted device specifications, or recovered descriptor passphrases. Keep those files under `.private/` (ignored by Git).

This repository is an independent community effort and is not affiliated with SteelSeries.
