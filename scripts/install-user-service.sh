#!/usr/bin/env bash
# Install files only. Does not enable/start lighting or request root access.
set -euo pipefail
project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
cargo build --release --bin apex-mini-rgb
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user" "$HOME/.config/apex-mini-rgb"
# Preserve existing user-customized profiles across upgrades.
if [[ ! -e "$HOME/.config/apex-mini-rgb/profile.conf" ]]; then
    install -m 0600 config/default.profile "$HOME/.config/apex-mini-rgb/profile.conf"
    echo "Created default profile; existing user profiles are never overwritten."
fi
# Atomic replacement is essential: the service may already be executing the
# old binary. Overwriting its inode directly would fail with "Text file busy".
stage="$(mktemp "$HOME/.local/bin/.apex-mini-rgb.XXXXXXXX")"
trap 'rm -f -- "$stage"' EXIT
install -m 0755 target/release/apex-mini-rgb "$stage"
mv -f -- "$stage" "$HOME/.local/bin/apex-mini-rgb"
trap - EXIT
install -m 0644 systemd/user/apex-mini-ripple.service "$HOME/.config/systemd/user/apex-mini-ripple.service"
systemctl --user daemon-reload
printf '%s\n' "Installed Apex Mini RGB binary, saved profile, and user service (NOT automatically enabled or started)." \
  "First test input WITHOUT writing lighting: $HOME/.local/bin/apex-mini-rgb --test-input --seconds 10" \
  "Profile: $HOME/.config/apex-mini-rgb/profile.conf" \
  "Then, when ready, enable background ripple with: systemctl --user enable --now apex-mini-ripple.service" \
  "Stop and disable: systemctl --user disable --now apex-mini-ripple.service" \
  "View status: systemctl --user status apex-mini-ripple.service" \
  "View service log: journalctl --user -u apex-mini-ripple.service -n 40 --no-pager"
