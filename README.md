# ApexProMini GG RGB for Linux

Native Linux research and driver-development workspace for the **wired SteelSeries Apex Pro Mini (USB `1038:161e`)**, using verified HID observations and local, legally obtained SteelSeries GG resources.

**Current status:** **Verified on real hardware:** A key red, S key green, and `0x41` clear restoring onboard lighting. **New in development:** nine native full-key RGB renderers, including rainbow/wave/breathing and keypress ripple/reactive using a verified physical Linux input node. Whole-board solid blue has been physically verified; **keypress ripple failed to react to keypresses** and its Linux evdev monitoring is being diagnosed and **are not yet certified as exact SteelSeries GG Prism presets**.

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

The two final commands **change RGB state**; both single-key lighting and clear/restoration have now been verified on real hardware. Run them only after reviewing read-only output and accepting that lighting may need a keyboard reconnect or native profile reset. There is no firmware, macro, or actuation functionality in this crate. The `clear` command restores control to onboard lighting if the firmware honors its official command; a USB success message alone does not prove it.

See [`docs/PROTOCOL.md`](docs/PROTOCOL.md) for the recovered frame layout and [`docs/ROADMAP.md`](docs/ROADMAP.md) for acceptance criteria.

## Full-keyboard lighting effects (experimental, opt-in)

```bash
cargo test --all-targets
cargo build --release

# Safe preview: constructs full 68-key RGB packet but writes nothing.
./target/release/apex-mini-rgb --preview-effect rainbow

# Control ALL keys with a static solid color, then press Ctrl+C to clear.
./target/release/apex-mini-rgb --effect solid --color 0033ff --confirm-device-write

# Moving full-key rainbow and animated breathing.
./target/release/apex-mini-rgb --effect wave --speed 1.2 --brightness 0.6 --confirm-device-write
./target/release/apex-mini-rgb --effect breathing --color ff0066 --confirm-device-write

# Keypress-triggered spatial ripple:
./target/release/apex-mini-rgb --list-inputs
./target/release/apex-mini-rgb --effect ripple --input /dev/input/eventN --color 081020 --accent ff5500 --confirm-device-write
```

Replace `eventN` with the exact keyboard input node printed by `--list-inputs`. It must be readable by your user, and the application verifies that it belongs to USB `1038:161e`. No keyboard grabbing or typed text logging. The effect stays active until Ctrl+C; normal onboard lighting should return via the already hardware-verified `0x41` clear report.

Effects currently available: **solid, gradient, rainbow, wave, breathing, color-shift, sparkle, ripple, reactive**. See [`docs/PRESET_COVERAGE.md`](docs/PRESET_COVERAGE.md) for the longer-term goal of **matching every official GG default preset**, including **31 official GG 108.3 preset identifiers already recovered from packaged SQL migrations** (not necessarily all simultaneously visible in GG), profiles, and base/idle/reactive layers.

**New effect rendering and keypress tracking need physical verification.** Avoid assuming a successful write means the effects are visually correct.

## Linux keypress monitoring (diagnostics and optional daemon)

The first ripple trial did not react even though solid full-keyboard blue worked. This build improves event monitoring, tries **all** verified Mini keyboard event nodes by default, and adds a read-only diagnostic:

```bash
./target/release/apex-mini-rgb --list-inputs
./target/release/apex-mini-rgb --test-input --seconds 10
# Tap A, S, Space; should print USB HID codes 4, 22, 44.
./target/release/apex-mini-rgb --effect ripple --color 081020 --accent ff5500 --confirm-device-write
```

Ripple automatically discovers the input device; `--input /dev/input/eventN` remains an optional explicit override. The monitor is passive and **does not grab typing or persist keystrokes**. The diagnostic prints numeric key IDs only on request.

You can optionally install a `systemd --user` service that keeps ripple running between terminal sessions (initially **disabled**):

```bash
bash scripts/install-user-service.sh
systemctl --user enable --now apex-mini-ripple.service
# To stop and return to onboard lighting:
systemctl --user disable --now apex-mini-ripple.service
```

**Important:** the daemon does not by itself solve a broken evdev input stream. Verify `--test-input` before enabling it. See [`docs/SERVICE.md`](docs/SERVICE.md) for permissions and troubleshooting; don't run input monitoring as root by default.

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
