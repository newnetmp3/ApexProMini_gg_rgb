# Linux keypress monitoring and reactive RGB service

**User-level systemd service:** `apex-mini-ripple.service`. No root daemon and no kernel module. Intended for Arch Linux (KDE Plasma / Wayland) running wired SteelSeries Apex Pro Mini, USB `1038:161e`.

## Why ripple did not trigger in the first prototype

The owner reported that **whole-keyboard static blue works** but **no keypress ripple appears**. This proves the 68-key RGB output path works on hardware. The exact input-side failure has **not yet been established**: the first implementation listened to only one manually selected evdev input node and had no read-only way to verify incoming keydown events. Some multi-interface HID keyboards expose more than one event node.

This milestone introduces multi-source detection, a diagnostic command, validated evdev parsing, and a user service. **The actual fix must be verified against keypresses on the owner's Mini**; do not claim working ripple until confirmed.

## Confirmed blocker on owner's Arch PC: permission denied (2026-10-14)

After PR #5, the owner ran `--list-inputs` and `--test-input --seconds 10`. The exact result was:

```text
/dev/input/event11 [no read access]
/dev/input/event12 [no read access]
Permission denied (os error 13)
```

This **confirms the immediate blocker is the local Linux input event permissions**, not lack of detected keyboard nodes or failure of the 513-byte RGB command. Ripple has **not yet been observed working** after a permission fix.

### Install the device-specific ACL rule (explicit opt-in)

The project includes `udev/71-apex-pro-mini-input.rules`, which matches **only** input `event*` keyboard devices attached to wired SteelSeries Apex Pro Mini USB `1038:161e`, then applies `TAG+="uaccess"` and `MODE="0660"`. The `71-` prefix deliberately runs before systemd's `73-seat-late.rules`. On an active local KDE/Wayland seat, logind should grant a temporary access-control-list entry to the active user. No system-wide `input` group membership, no root daemon, and no `chmod 666` needed.

```bash
cd ~/local-git/ApexProMini_gg_rgb
git pull --ff-only
bash scripts/install-input-permissions.sh
```

The script uses `sudo` **only to copy this exact udev rule** to `/etc/udev/rules.d/71-apex-pro-mini-input.rules` and reload udev rules. It does not enable, install or start the lighting service. **Disconnect and reconnect the Mini** after installation to re-enumerate event nodes and reapply ACLs. On a machine where this is the only input keyboard, ensure you can reconnect before unplugging.

Verify using the **new** event node numbers (they may differ from `event11` and `event12`):

```bash
./target/release/apex-mini-rgb --list-inputs
./target/release/apex-mini-rgb --test-input --seconds 10
```

During the 10 seconds, press A, S and Space: expected USB HID usages `4`, `22`, `44`. **Do not type passwords or sensitive text** during this explicitly requested key event diagnostic. Once events work, opt in to a manual ripple test before enabling the service.

If the devices still report no access, inspect the **current** paths:

```bash
udevadm info --query=property --name=/dev/input/event11 | grep -E '^(ID_INPUT_KEYBOARD|TAGS|CURRENT_TAGS|ID_VENDOR_ID|ID_MODEL_ID)='
getfacl -- /dev/input/event11
loginctl session-status
```

Replace `event11` with a path returned by `--list-inputs`. Check whether `uaccess` is present among the udev tags, whether your logged-in UID has an `rw-` ACL, and whether your graphical session is active on a local seat. If `ID_INPUT_KEYBOARD` is not `1`, the rule may need a justified device-specific adjustment. Do not blanket-change input permissions.

To uninstall the custom ACL rule:

```bash
sudo rm /etc/udev/rules.d/71-apex-pro-mini-input.rules
sudo udevadm control --reload-rules
# Reconnect keyboard to apply the change to newly enumerated nodes.
```

## Read-only diagnosis (first)

```bash
git pull --ff-only
cargo build --release

./target/release/apex-mini-rgb --list-inputs
./target/release/apex-mini-rgb --test-input --seconds 10
```

Press A, S and Space on the Mini during the 10-second test. Expected mapped HID numbers: A=`4`, S=`22`, Space=`44`. If the output reports 0, **the keyboard events are not reaching the reader**. The diagnostic does not access the RGB interface, issue HID writes, store events or use the network. It prints numeric key IDs on screen *only when explicitly invoked*; don't paste diagnostic results containing sensitive typing.

The reader now monitors **all validated keyboard input nodes** belonging to USB `1038:161e`, rather than guessing one. It rejects unknown keyboard devices and neither intercepts nor grabs input. It handles key down (`EV_KEY value=1`) and ignores key releases and repeats. If permissions prevent reading input nodes, inspect `ls -l /dev/input/eventN`, `getfacl /dev/input/eventN` and active-session ACL/udev setup; don't use root by default.

## Run ripple manually

```bash
./target/release/apex-mini-rgb --effect ripple --color 081020 --accent ff5500 --fps 24 --confirm-device-write
```

No `--input` path is required for auto-discovery. You may still pass `--input /dev/input/eventN` if a physical interface needs testing. The exact vendor HID endpoint is independently checked. Ctrl+C or SIGTERM restores onboard lighting with the already hardware-verified `0x41` command. A crash, SIGKILL or loss of keyboard connectivity can prevent automatic restoration.

## Install the persistent user daemon (opt-in)

Review the unit and script before running. Installation only copies files; **the service remains disabled** until explicitly enabled:

```bash
bash scripts/install-user-service.sh

# Enable after confirming --test-input prints A and S:
systemctl --user enable --now apex-mini-ripple.service
systemctl --user status apex-mini-ripple.service

# Disable and stop, restoring onboard lighting:
systemctl --user disable --now apex-mini-ripple.service
```

This uses `%h/.local/bin/apex-mini-rgb` and starts automatically for your **user session** when enabled. It attempts a restart on a nonzero exit (for example, disconnected keyboard or inaccessible evdev); after reconnect it rediscovers new hidraw and eventN nodes. Changing the unit's `ExecStart` lets you choose `--effect reactive` or change colors, FPS and speed. Reload and restart after changing unit files:

```bash
systemctl --user daemon-reload
systemctl --user restart apex-mini-ripple.service
```

No typed strings are stored or logged by the daemon. Numeric Linux input key codes are mapped to HID usages and kept only for the transient animation.

## Troubleshooting

```bash
./target/release/apex-mini-rgb --list-inputs
./target/release/apex-mini-rgb --test-input --seconds 10
systemctl --user status apex-mini-ripple.service
journalctl --user -u apex-mini-ripple.service -n 40 --no-pager
```

If the service is active but input tests show presses and ripple remains invisible, next inspect the renderer's event handling and actual output color changes with offline golden frames, not arbitrary HID packet guesses. If no input nodes are found, inspect the Mini's USB interfaces and `/sys/class/input` topology; keypress monitoring cannot work without event input from the device.
