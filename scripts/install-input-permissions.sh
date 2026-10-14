#!/usr/bin/env bash
# Install a narrowly scoped udev rule using sudo; does not change other
# keyboard permissions, add groups, or enable any user daemon.
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_rule="$root/udev/71-apex-pro-mini-input.rules"
destination="/etc/udev/rules.d/71-apex-pro-mini-input.rules"

if [[ ! -r "$source_rule" ]]; then
  echo "Rule not found: $source_rule" >&2
  exit 1
fi
if ! command -v udevadm >/dev/null 2>&1; then
  echo "udevadm is required. This installer is intended for systemd Linux." >&2
  exit 1
fi
if ! command -v sudo >/dev/null 2>&1; then
  echo "sudo is required to install one udev rule under /etc/udev/rules.d/." >&2
  exit 1
fi
echo "Installing a seat-based input ACL for wired Apex Pro Mini 1038:161e only."
echo "The active local seat user will be able to read Mini keyboard key events."
echo "This does not install/start the background daemon."
sudo install -D -m 0644 "$source_rule" "$destination"
sudo udevadm control --reload-rules
echo
echo "Rule installed: $destination"
echo "Disconnect and reconnect the Apex Pro Mini to apply it to newly enumerated"
echo "event nodes, then check:"
echo "  ./target/release/apex-mini-rgb --list-inputs"
echo "  ./target/release/apex-mini-rgb --test-input --seconds 10"
echo
echo "If still unreadable, inspect getfacl on the NEW event nodes and verify"
echo "that your graphical login session is active on a local seat."
echo "To undo: sudo rm '$destination' && sudo udevadm control --reload-rules"
