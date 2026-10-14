# Development setup — Arch Linux

## 1. Check out this repository

```bash
git clone git@github.com:newnetmp3/ApexProMini_gg_rgb.git
cd ApexProMini_gg_rgb
python3 tools/hid_probe.py
python3 -m unittest discover -s tests -v
```

The probe only reads `/sys/class/hidraw` and report descriptors; it never opens any `/dev/hidraw*` node for writing.

## 2. Build and test the independent native Rust RGB utility

The Rust crate has no external dependencies. On Arch Linux, install `rust` and `cargo` if needed, then:

```bash
cargo test --all-targets
cargo build --release

# Read-only: must discover exactly one Mini control endpoint with Feature=513, Output=65
./target/release/apex-mini-rgb

# Offline packet preview; does not open a HID node:
./target/release/apex-mini-rgb --dry-run A ff0000
```

**Optional owner-approved hardware test** (only if the above succeeds):

```bash
./target/release/apex-mini-rgb --write-key A ff0000 --confirm-device-write
# Look at the keyboard: did the A key become red?
./target/release/apex-mini-rgb --clear --confirm-device-write
```

The last two commands write HID reports and can temporarily alter onboard lighting. They do not update firmware or actuation. The user must decide to run them; the CLI does not send anything without the complete explicit confirmation argument.

The Mini's `0x40` (direct Feature 513-byte) and `0x41` (clear Output 65-byte) **have been confirmed by the owner**: A red, S green, and clear restored normal onboard lighting. The new full-board RGB effects have not yet been hardware-tested. If no LEDs change, save console output instead of guessing other report bytes.

## 3. Test full-keyboard lighting and keypress ripple

```bash
# Safe read-only effects preview
./target/release/apex-mini-rgb --preview-effect wave

# Optional whole-board solid-color test; press Ctrl+C to restore:
./target/release/apex-mini-rgb --effect solid --color 1122ee --confirm-device-write

# Animated presets, each stops/restores with Ctrl+C:
./target/release/apex-mini-rgb --effect rainbow --fps 20 --brightness 0.5 --confirm-device-write
./target/release/apex-mini-rgb --effect breathing --color ff0055 --speed 1.1 --confirm-device-write

# Identify available keyboard input-event devices (read-only):
./target/release/apex-mini-rgb --list-inputs

# Replace eventN with the path just listed; passive Linux evdev, no keyboard grab:
./target/release/apex-mini-rgb --effect ripple --input /dev/input/eventN --accent ff5500 --confirm-device-write
```

Important: `--effect` and `--preview-effect` are different. Preview does not open a device or send USB reports. Active effects require `--confirm-device-write` and verify the exact Mini USB ID, vendor HID usage page, and feature/output sizes. Ripple/reactive also require **explicitly chosen Mini evdev device** and permission to read it; this works independently of Wayland compositor shortcuts. Do not use `sudo` blindly if input permissions fail—inspect the node's ACL and your active seat permissions.

The new renderer uses approximate key geometry including international layouts. Per-key ripple positioning needs keyboard-side confirmation; the input reader never writes files, grabs keys, or records typed strings. `Ctrl+C` or `SIGTERM` attempts `0x41` clear to restore onboard lighting; a crash or hard kill cannot guarantee restoration.

## 4. Work on the upstream transport separately

The owner has an existing `steelseriesgg-rs` checkout, with the transport patch already applied; **do not reapply it blindly**. For a clean upstream clone, inspect and apply the patch:

```bash
cd /path/to/steelseriesgg-rs
git status --short
git apply --check /path/to/ApexProMini_gg_rgb/patches/steelseriesgg-rs-hid-report-fix.patch
git apply /path/to/ApexProMini_gg_rgb/patches/steelseriesgg-rs-hid-report-fix.patch
cargo test
cargo build --release
```

If `git apply --check` fails, inspect `git diff` before continuing: the upstream patch may already be present. Never reset a modified worktree just to apply this patch.

## 5. Local-only GG descriptor decryption

This research used the owner's own GG Windows `SteelSeriesEngine.exe` and macOS `.edevice` assets. Do **not** commit those files, the recovered passphrase, or decrypted plaintext. `.private/` is excluded by `.gitignore`.

Use the independently maintained [steelpatcher](https://github.com/hurryman2212/steelpatcher) parser. Install its Python dependency into a local virtual environment:

```bash
git clone https://github.com/hurryman2212/steelpatcher.git ../steelpatcher
python3 -m venv .venv
source .venv/bin/activate
pip install cryptography
```

Place your authorized copies of `SteelSeriesEngine.exe` and `apex_pro_mini.edevice` in this repository's `.private/` directory. Then run the wrapper with the steelpatcher source on the Python module search path:

```bash
PYTHONPATH=../steelpatcher python3 tools/decrypt_descriptor_local.py \
  --engine .private/SteelSeriesEngine.exe \
  --descriptor .private/apex_pro_mini.edevice
```

This produces `.private/apex_pro_mini.edevice.scm` with owner-only permissions, prints no secret and does **not** transmit any data. Repeat with `base_apex_2022_mini_layout_lighting.edevice` if available. These files are proprietary vendor sources for private analysis.

Only make independent protocol notes and original implementation code public; do not paste the decrypted source into issues, commits, or PRs.

## 6. Hardware write policy

Only the explicit Rust CLI commands `--write-key`, `--clear` and `--effect` can write to the keyboard, and only with `--confirm-device-write`. The default discovery and dry-run commands remain read-only. Mini's observed Feature report is 513 bytes: the experimental 645-byte TKL-2023 packet is **incompatible**, and must not be forced or truncated. Do not attempt firmware update, unknown HID command fuzzing, or actuation settings changes. New write-capable tests must be opt-in and independently verified first.

## 7. Development handover

After every meaningful implementation milestone, update `docs/HANDOVER.md` **and** `docs/ROADMAP.md` together with tests and evidence. Use `docs/PROTOCOL.md` to separate confirmed bytes from hypotheses.
