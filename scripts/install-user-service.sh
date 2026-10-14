#!/usr/bin/env bash
# Install files only. Does not enable/start lighting or request root access.
set -euo pipefail
project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
cargo build --release --bin apex-mini-rgb
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
install -m 0755 target/release/apex-mini-rgb "$HOME/.local/bin/apex-mini-rgb"
install -m 0644 systemd/user/apex-mini-ripple.service "$HOME/.config/systemd/user/apex-mini-ripple.service"
systemctl --user daemon-reload
printf '%s\n' "Installed Apex Mini RGB binary and user service (NOT enabled)." \
  "First test input WITHOUT writing lighting: $HOME/.local/bin/apex-mini-rgb --test-input --seconds 10" \
  "Then, when ready, enable background ripple with: systemctl --user enable --now apex-mini-ripple.service" \
  "Stop and disable: systemctl --user disable --now apex-mini-ripple.service" \
  "View status: systemctl --user status apex-mini-ripple.service" \
  "View service log: journalctl --user -u apex-mini-ripple.service -n 40 --no-pager"
