# Official GG installer research — reproducible findings

Analysis date: **2026-10-14**. Static analysis only; no proprietary software was executed on the target Linux device.

## Inputs supplied privately

| Local source | Size (bytes) | SHA-256 |
|---|---:|---|
| \`SteelSeriesGG108.3.0.pkg\` (macOS) | 299,941,744 | \`80c2eec20da51a6144bb18c70cf511ba986941ab0ebd224151ec54dc5e6b4b41\` |
| \`SteelSeriesGG121.0.0Setup.exe\` (Windows) | 433,340,936 | \`2fffd8fe5a22172f6d8c47bdcd79237c097f0691d6c7db531fa4d37e2dc21907\` |
| \`SteelSeriesEngine.exe\` (Windows, extracted separately) | 16,648,248 | \`8a53c82084a922290b62dc4e6ea61621a1f939171089036e99f2503bb5010cb9\` |

The Windows installation package was identified as NSIS; a full nested payload inventory is still pending. The **separately supplied Windows Engine executable** was successfully analyzed.

## macOS payload structure

The GG 108.3.0 package is XAR containing subpackages for Engine Core, Prism, main GG and uninstaller. Engine Core contains the following **exact wired Mini** assets:

- \`deviceSpecifications/apex_pro_mini.edevice\` (encrypted; SHA-256 \`91d894fd23d19073853ca0d2266430139ec6606970ab1d7e70e06a17b6bfaa1d\`)
- \`deviceSpecifications/base_apex_2022_mini_layout_lighting.edevice\` (encrypted; SHA-256 \`b12b277eefab957d02b1e589028244009b747a7778d3164b8ce07741e699248d\`)
- \`configurationMigrations/apex_pro_mini.migration\` (readable)
- Firmware metadata for \`apex_pro_mini\`, firmware \`1.19.7\`.

These files remain with the owner only. Vendor executables, firmware, original descriptors, and decrypted descriptor text are **not stored in this repository**.

## Confirmed descriptor-decryption milestone

- Descriptor files are text-armored encrypted OpenPGP packets using AES-128 and iterated/salted S2K SHA-256.
- [steelpatcher](https://github.com/hurryman2212/steelpatcher) contains a documented Go PE function-table parser and \`extract_descriptor_password\` / \`decrypt_descriptor\` functions.
- With the privately supplied Windows \`SteelSeriesEngine.exe\`, the extractor recovered the required password. Both GG 108.3 macOS \`.edevice\` descriptors decrypted successfully.
- Reproducible *local-only* decryption wrapper: \`tools/decrypt_descriptor_local.py\`; see \`docs/SETUP.md\`.
- No password, decrypted vendor text, or proprietary firmware has been published.

## Independent observations derived from the decoded specification

- Exact wired device is \`0x1038161e\`; vendor control usage page \`0xFFC0\`, usage 1.
- Device feature size is **513 bytes**, matching the owner's Linux descriptor probe.
- Internal firmware mapping includes **70 slots**, 68 with nonzero HID usages; this is **not the same as the key count of an arbitrary physical layout**.
- Lighting calls a named high-level \`direct_write\` API inherited from shared device frameworks; resolving **actual USB report bytes** remains the main obstacle.
- Device includes distinct primary and secondary actuation thresholds, per-key mapping, and firmware-specific onboard profile schemas (3–6). These are not safe to modify until report protocols are independently verified.

## Required next research

1. Resolve the inherited Engine include implementing the \`direct_write\` serializer (e.g., \`individually-lit-keyboard\` and wired Apex base), or acquire known-good RGB-only HID traces from official software.
2. Determine command byte, payload structure, report type (65-byte Output or 513-byte Feature), required setup, apply/clear, and per-key addressing.
3. Write independent Rust packet builders and golden-data tests from confirmed packets.
4. Perform guarded hardware-only RGB testing with owner approval; record observed LED effects before claiming support.

No decryption of official assets alone proves that a candidate HID report will work.
