# Apex Pro Mini protocol — observations vs open questions

**Target:** wired SteelSeries Apex Pro Mini, USB VID/PID \`1038:161e\`.

## Verified on owner's Linux hardware

| Property | Value |
|---|---|
| Device selection | Interface 1; HID usage page \`0xffc0\`, usage \`0x01\` |
| HID Output report | ID 0; 65 bytes including report-ID byte |
| HID Feature report | ID 0; 513 bytes including report-ID byte |
| Writes through generic \`ssgg\` | 3/3 successful after transport patch; **no visible RGB effect** |
| Example command attempted | Generic zoned RGB output begins \`00 21 FF ...\`; ignored/ineffective on Mini |
| Report-ID bug | Earlier logs showed extra leading zero; fixed and retested |
| APPLY deduplication | Previously skipped; fixed and retested |

The active \`hidraw\` number is not stable and **must be rediscovered**.

## Verified by static examination of provided GG installers

- GG Mini descriptor identifies wired PID \`0x1038161e\`, vendor control usage page \`0xffc0\`, feature report size **513**, and model firmware key table of **70 slots**, including 2 unused slots.
- Mini lighting uses an inherited individually-lit keyboard framework and high-level Engine APIs named \`direct_write\` and \`clear_direct_write\`. This identifies an integration path, **not the byte-level HID report**.
- Device-specific firmware/profile handling supports several versions of the onboard profile schema (3, 4, 5, 6); avoid attempting profile writes until formats are understood.
- Current available firmware metadata notes \`1.19.7\`; this is **installer-supplied metadata**, not a live read of the user's device firmware.

## Open wire-protocol questions

1. Which concrete HID command byte and report type implement \`direct_write\` on wired Mini?
2. Does the RGB payload go over 65-byte Output, 513-byte Feature, or both?
3. What are the initialization, direct-mode selection, and clear/commit semantics?
4. Are LEDs addressed by USB HID usage, firmware key index, or a fixed contiguous layout?
5. What is the safe rate limit/frame size and does it vary with firmware?
6. How are device profile changes reflected by Prism and onboard lighting?

**Do not reuse** the \`0x40\` 645-byte direct feature packet documented for Apex Pro TKL (2023 PID \`0x1628\`) on the Mini; it cannot fit the Mini's advertised Feature report. A successful USB write is not device confirmation.

## Reference implementation research

- Native Linux Rust project: https://github.com/Ven0m0/steelseriesgg-rs
- Descriptor analysis tool: https://github.com/hurryman2212/steelpatcher
- No decrypted SteelSeries descriptor text or decryption passphrase belongs in this public repository.
