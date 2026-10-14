# Development setup — Arch Linux

## 1. Check out this repository

\`\`\`bash
git clone git@github.com:newnetmp3/ApexProMini_gg_rgb.git
cd ApexProMini_gg_rgb
python3 tools/hid_probe.py
python3 -m unittest discover -s tests -v
\`\`\`

The probe only reads \`/sys/class/hidraw\` and report descriptors; it never opens any \`/dev/hidraw*\` node for writing.

## 2. Work on the upstream transport separately

The owner has an existing \`steelseriesgg-rs\` checkout, with the transport patch already applied; **do not reapply it blindly**. For a clean upstream clone, inspect and apply the patch:

\`\`\`bash
cd /path/to/steelseriesgg-rs
git status --short
git apply --check /path/to/ApexProMini_gg_rgb/patches/steelseriesgg-rs-hid-report-fix.patch
git apply /path/to/ApexProMini_gg_rgb/patches/steelseriesgg-rs-hid-report-fix.patch
cargo test
cargo build --release
\`\`\`

If \`git apply --check\` fails, inspect \`git diff\` before continuing: the upstream patch may already be present. Never reset a modified worktree just to apply this patch.

## 3. Local-only GG descriptor decryption

This research used the owner's own GG Windows \`SteelSeriesEngine.exe\` and macOS \`.edevice\` assets. Do **not** commit those files, the recovered passphrase, or decrypted plaintext. \`.private/\` is excluded by \`.gitignore\`.

Use the independently maintained [steelpatcher](https://github.com/hurryman2212/steelpatcher) parser. Install its Python dependency into a local virtual environment:

\`\`\`bash
git clone https://github.com/hurryman2212/steelpatcher.git ../steelpatcher
python3 -m venv .venv
source .venv/bin/activate
pip install cryptography
\`\`\`

Place your authorized copies of \`SteelSeriesEngine.exe\` and \`apex_pro_mini.edevice\` in this repository's \`.private/\` directory. Then run the wrapper with the steelpatcher source on the Python module search path:

\`\`\`bash
PYTHONPATH=../steelpatcher python3 tools/decrypt_descriptor_local.py \
  --engine .private/SteelSeriesEngine.exe \
  --descriptor .private/apex_pro_mini.edevice
\`\`\`

This produces \`.private/apex_pro_mini.edevice.scm\` with owner-only permissions, prints no secret and does **not** transmit any data. Repeat with \`base_apex_2022_mini_layout_lighting.edevice\` if available. These files are proprietary vendor sources for private analysis.

Only make independent protocol notes and original implementation code public; do not paste the decrypted source into issues, commits, or PRs.

## 4. Hardware write policy

No current code in this repository writes to the keyboard. Mini's observed Feature report is 513 bytes: the experimental 645-byte TKL-2023 packet is **incompatible**, and must not be forced or truncated. Do not attempt firmware update, unknown HID command fuzzing, or actuation settings changes. New write-capable tests must be opt-in and independently verified first.

## 5. Development handover

After every meaningful implementation milestone, update \`docs/HANDOVER.md\` **and** \`docs/ROADMAP.md\` together with tests and evidence. Use \`docs/PROTOCOL.md\` to separate confirmed bytes from hypotheses.
