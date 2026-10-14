#!/usr/bin/env bash
# Arch Linux / KDE Wayland: build and install native desktop GUI for the owner.
# Explicit invocation only. Does not automatically start/enable lighting.
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
# Ensure the user daemon knows the shared profile format; preserves any
# existing running service and user profile (no forced restart).
bash scripts/install-user-service.sh
cargo build --release --features gui --bin apex-mini-gui
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
install -m 0755 target/release/apex-mini-gui "$HOME/.local/bin/apex-mini-gui"
desktop="$HOME/.local/share/applications/apex-mini-prism.desktop"
cat > "$desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Apex Mini Prism
Comment=Native Linux keyboard RGB effects and preset editor
Exec=$HOME/.local/bin/apex-mini-gui
Icon=input-keyboard
Terminal=false
Categories=Settings;Utility;
StartupNotify=true
EOF
chmod 0644 "$desktop"
echo "GUI installed. Launch from your application menu (Apex Mini Prism), or:"
echo "  $HOME/.local/bin/apex-mini-gui"
echo "The current background service was not interrupted. Use 'Apply & restart lighting' in the GUI."
