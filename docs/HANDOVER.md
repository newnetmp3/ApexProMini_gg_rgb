# ApexProMini_gg_rgb — rolling development handover

**Updated:** 2026-10-14  
**Repository:** https://github.com/newnetmp3/ApexProMini_gg_rgb  
**Integration status:** Bootstrap branch \`bootstrap/research-handover\`, intended for PR into \`main\`. Check the current GitHub branch and workflow results before continuing.  
**Target:** Wired SteelSeries Apex Pro Mini, VID:PID \`1038:161e\` on Arch Linux / KDE Plasma / Wayland. This repo is independent from upstream \`Ven0m0/steelseriesgg-rs\`.

## Goal

Create a native Linux/Rust implementation for **verified** Apex Pro Mini RGB and, subsequently, profiles, per-key illumination, actuation, rapid trigger, and remapping. Use official SteelSeries GG as a protocol research reference, not as code to redistribute.

## Confirmed owner-hardware evidence

- Connected Mini exposes control interface 1, usage page \`0xFFC0\`, usage \`0x01\`, with \`/dev/hidraw7\` on the latest observed session (device number may change).
- Kernel descriptor reports **Output 65 bytes**, **Feature 513 bytes**, both report ID 0. These are hardware-observed values.
- \`ssgg\` \`0.1.5\` opens the correct interface and submits HID writes without permission errors.
- The local \`patches/steelseriesgg-rs-hid-report-fix.patch\` fixed a duplicate report ID and a skipped APPLY report. The owner confirmed logs after applying it: correctly framed \`00 21 FF FF 00 00...\` and 3/3 writes successful. **Keyboard lighting did not change.** USB success != accepted Mini RGB protocol.
- Current upstream routing for PID \`161e\` is \`GenericKeyboard\` with no dedicated key map or zone map.
- **Do not send** the 645-byte Apex Pro TKL 2023 RGB feature packet to the Mini; its advertised maximum feature report is 513 bytes.

## Official GG static research completed

- Input packages: macOS GG 108.3.0 \`.pkg\`, Windows GG 121.0.0 setup \`.exe\`, and Windows \`SteelSeriesEngine.exe\` provided privately.
- macOS Engine Core includes \`apex_pro_mini.edevice\`, \`base_apex_2022_mini_layout_lighting.edevice\`, \`apex_pro_mini.migration\`, and firmware metadata identifying wired Mini firmware 1.19.7.
- Independent [steelpatcher](https://github.com/hurryman2212/steelpatcher) reverse-engineering work provided a password-extraction algorithm for Windows Engine. **That extraction succeeded** on supplied Windows Engine, and **both .edevice specifications decrypted successfully** using the resulting password. The password and decrypted vendor files remain private and are not committed.
- Official decrypted Mini spec independently confirms usage \`0xFFC0\`, feature size **513** bytes, a **70-position** firmware key-index table with **68 nonzero HID mappings**, shared Mini lighting definitions, and firmware-version-aware profile schemas (3–6). This is static reference data only.
- Shared lighting spec calls high-level Engine APIs \`direct_write\` / \`clear_direct_write\`, but their final HID packet serializer and command bytes are **not yet established**. A high-level API name is not a device command byte.

## Current repository contents

- \`docs/RESEARCH.md\` — official asset inventory and reproducible evidence without vendor plaintext.
- \`docs/PROTOCOL.md\` — confirmed transport facts and unknown wire encoding.
- \`tools/hid_probe.py\` — safe read-only HID report inventory; runs without external packages.
- \`tools/decrypt_descriptor_local.py\` — opt-in local-only wrapper around steelpatcher code; does not print or store the passphrase.
- \`patches/steelseriesgg-rs-hid-report-fix.patch\` — previously hardware-log-verified upstream transport fix; applied locally by owner to separate upstream checkout.
- \`tests/test_hid_probe.py\` — synthetic HID descriptor parsing tests.
- \`.gitignore\` — excludes vendor binaries, plaintext specs, captured traces, and all \`.private/\` content.

## Next engineering tasks

1. **Locate actual RGB HID API implementation** (e.g., definitions behind \`direct_write\`, inherited \`individually-lit-keyboard\`, and \`base_apex_2022_wired\`). Obtain permitted local copies of inherited includes from own installer, or derive command packets from verified captures; do not infer packet bytes from API names.
2. Design an independent Rust packet serializer with unit tests against verified report bytes (65-byte output versus 513-byte feature distinguished).
3. Provide an opt-in static red/green/blue test that verifies exact VID/PID and report sizes before any write. No broad command scanning, flashing, macros, or actuation testing.
4. Test real key lighting and per-key order on owner's Mini; document results and regressions. Only then mark RGB supported.
5. Expand to Prism effects and profiles, then separately validate actuation/rapid trigger.

## Rolling-handover process

At the end of **every meaningful milestone**, update **this file** and \`docs/ROADMAP.md\` in the same PR/commit as the code and tests. Include: date, branch/commit/PR, verified behavior versus hypothesis, tests run, remaining problems, next concrete steps. Do not write an unverified feature as completed. Next chat handover instruction:

> Continue \`newnetmp3/ApexProMini_gg_rgb\` from \`docs/HANDOVER.md\` and \`docs/ROADMAP.md\`. Inspect the latest GitHub state, execute the next verified milestone, and update both rolling documents alongside code and tests.
