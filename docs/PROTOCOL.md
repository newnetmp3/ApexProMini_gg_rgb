# Apex Pro Mini HID RGB protocol: verified static structure

**Hardware:** wired SteelSeries Apex Pro Mini, VID/PID `1038:161e`.

## Linux hardware evidence (owner's Arch workstation)

- USB HID control interface 1, usage page `0xffc0`, usage `0x01`.
- Report ID `0x00`, **65-byte Output** and **513-byte Feature** reports (the leading byte is counted in each).
- Earlier generic `ssgg` command `0x21` wrote without errors but **did not change lighting**.
- The upstream `ssgg` transport double report-ID issue and skipped final APPLY were fixed and observed as transmitted; still no visible RGB effect. No permissions problems on the control interface.
- Device node numbers such as `/dev/hidraw7` are temporary and must never be hardcoded.

## Official macOS GG 108.3 descriptor analysis — independently implemented

Locally decrypted the owner's `apex_pro_mini.edevice`, `base_apex_2022_mini_layout_lighting.edevice`, and the newly extracted inherited `base_apex_2022_wired.edevice`. The latter defines the previously missing wire serializer:

| Operation | Command | API transport | Exact report size |
|---|---|---|---|
| `direct_write` | `0x40` | `HIDFEATURE` | **513 bytes** |
| `clear_direct_write` | `0x41` | `HIDIO` (Output) | **65 bytes** |

The `direct_write` byte layout is:

```text
00 40 NN [HID R G B] * 68 ... zero padding to 513 bytes
 |  |  |           |
 ID CMD actual count (0..68); remaining declared slots unused
```

**Fixed serialized field:** The official struct declares **68 key entries** for this exact Mini model, each represented by HID code and RGB components, with `NN` indicating how many entries contain meaningful colors. The 70 firmware key-index slots are a *different* internal mapping; two are zero/unused. USB HID key codes are not firmware slot numbers.

For a single red `A` key, the expected report prefix is `00 40 01 04 FF 00 00`; remaining bytes are zero. The `clear_direct_write` Output prefix is `00 41` padded to 65 bytes. The independent Rust serialization logic and offline tests live in `src/lib.rs`. **No proprietary descriptor text or password is committed.**

**Important:** These frame definitions are **verified against vendor static specifications**, *not yet observed working on a physical keyboard*. No claim of live functionality should be made until the owner confirms it.

## Safety and unanswered questions

- The `apex-mini-rgb` binary performs only read-only discovery unless the exact `--confirm-device-write` flag is provided.
- It checks `1038:161e`, vendor usage page `0xffc0`, and exact report sizes before opening a HID node for writing.
- It has an opt-in **one-key RGB Feature** operation and a separate **clear-direct-mode Output** operation. No flashing, actuation, profile writes, or arbitrary HID commands.
- Official GG may require initialization, profile state, an active Prism service, or a particular write order before a command becomes visible. Those interactions are not yet experimentally confirmed.
- A successful `ioctl` or HID `write` does **not** prove the RGB effect appeared. Record physical keyboard behavior.

## Manual opt-in smoke test after CI passes

```bash
cargo test
cargo build --release
./target/release/apex-mini-rgb
./target/release/apex-mini-rgb --dry-run A ff0000
./target/release/apex-mini-rgb --write-key A ff0000 --confirm-device-write
# Inspect the actual A-key LED, then restore onboard lighting:
./target/release/apex-mini-rgb --clear --confirm-device-write
```

Do not execute the optional device writes until the owner wants to try them. If a write fails, do not retry with `sudo` without checking the relevant hidraw node permissions.

## Resources

- `docs/RESEARCH.md`: installer provenance and static research, without proprietary files.
- `docs/HANDOVER.md`: rolling milestone status.
- `docs/ROADMAP.md`: completion criteria and upcoming work.
- Upstream Rust: https://github.com/Ven0m0/steelseriesgg-rs
- Encrypted descriptor research: https://github.com/hurryman2212/steelpatcher
