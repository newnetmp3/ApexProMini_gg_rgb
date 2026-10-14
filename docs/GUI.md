# Apex Mini Prism — native Linux GUI milestone

**Status:** First native GUI implementation merged as [PR #9](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/9), `edf7e45a8082a87b6ebf3e5119afc96f30684248`. GitHub Actions confirmed Linux native GUI compilation, headless Rust tests/Clippy and Python diagnostics. **Not yet visually/hardware tested on the owner's Arch installation.** Prior owner hardware checks did verify basic per-key RGB, 68-key whole-board solid lighting, keypress ripple, clear, and the systemd user daemon.

## Quick start (Arch Linux)

Update the repository, build the new optional GUI, and update the existing user service:

```bash
cd ~/local-git/ApexProMini_gg_rgb
git pull --ff-only
bash scripts/install-gui.sh
```

Then start **Apex Mini Prism** from your KDE application menu, or:

```bash
~/.local/bin/apex-mini-gui
```

Manual build without installation:

```bash
cargo run --release --features gui --bin apex-mini-gui
```

The native GUI compiles with the optional Rust `gui` feature. Existing `apex-mini-rgb` CLI builds remain headless and lightweight. The build needs a functioning Rust/cargo install plus system Wayland/X11/OpenGL libraries normally present on a Plasma desktop.

## Current editor features

- Preset selector for **fourteen independent native renderer prototypes**: solid, gradient, rainbow, wave, breathing, color shift, sparkle, ripple, reactive, comet, split rainbow, prismatic sweep, heatmap and reactive line. **These are not automatically identical to official GG presets.**
- Live **locally simulated** 68-logical-key keyboard preview using the same Rust renderer as the actual device; this is not live hardware readback
- Editable primary/accent RGB colors, brightness, speed, FPS (bounded and validated)
- Select a keyboard key in the preview, assign a persistent per-key RGB override, or clear the override
- Simulated sample keypress for ripple/reactive preview; does not intercept user typing
- Save profile without writing lighting
- **Apply & restart** the existing user-level systemd daemon using the saved profile; no second HID writer
- Explicit service start/enable, stop and disable buttons
- Visible application-side status and errors (for example if the old daemon unit needs an upgrade)
- Reference **31 historical SteelSeries GG 108.3 Prism preset identifiers** with *unmatched* status. These are catalog metadata, not a claim of official GG color/timing parity.

## Single-writer and persistence design

The GUI **never opens hidraw or evdev**. Existing `apex-mini-rgb` owns all device writes through the user's systemd service. `scripts/install-user-service.sh` installs an updated service whose `ExecStart` includes:

```text
--effect ripple --profile %h/.config/apex-mini-rgb/profile.conf --confirm-device-write
```

The stored profile takes precedence over the default command's ripple preset. The user file lives at `~/.config/apex-mini-rgb/profile.conf`. The installer creates a safe initial profile only if no profile exists and never overwrites user preferences. The CLI validates `version=1`, effect names, exact color codes, FPS 5–40, speed 0.1–5, brightness 0–1 and device HID usages before accepting it. File replacement is atomic.

The GUI's **Apply & restart** intentionally stops the old user service and starts it again with the new validated effect profile. This avoids multiple processes issuing 513-byte RGB frames to the same keyboard. If the service is active when the installer updates its unit, its existing process remains running until Apply or an explicit restart.

The RGB daemon still requires the user-approved `udev/71-apex-pro-mini-input.rules` for reactive effects, as described in [SERVICE.md](SERVICE.md). No user key events are written to the profile or log.

## Safety, defaults and controls

- Saving is local only and safe: **no HID writes until explicitly applying/starting** the user service.
- Stopping the service attempts the confirmed Mini `0x41` restore command.
- The user can disable persistent lighting at login independently of saving an RGB profile.
- On-screen preview/individual key layout uses a **provisional 60% geometry**; international variants and precise key sizes need calibration.
- Per-key RGB overrides are *absolute/static* and currently replace the selected key's value after the animation renderer. Per-key effects and layered base+reactive behavior are planned next.
- The service is a user process, never root; installing the GUI does not change the keyboard until Apply/Start.
- Do not run a separate manual `--effect` CLI command while the GUI-controlled service is active. Use GUI Apply rather than simultaneous HID writes.

## Required hardware acceptance tests

1. GUI launches on KDE Plasma/Wayland and displays a visible interactive keyboard without rendering glitches.
2. Select Solid; pick primary color; **Apply & restart** changes full-board lighting.
3. Select Ripple; Apply; keypress ripple still works on the Mini (no regression).
4. Change speed and brightness; Apply; verify visible effect responds.
5. Select one key and set an override; Apply; verify it changes only that LED while others follow the selected preset.
6. Stop via GUI; onboard lighting returns. Enable/start again and verify profile restores.
7. Confirm saved settings survive logout/login and that only one user-service process controls the HID device.
8. Compare next effects to SteelSeries GG's actual bundled Prism preset parameters; mark exact vendor parity only after visible comparisons.

## Next parity stages

- Resolve official GG 108.3/121 preset parameter dictionaries and behaviors from authorized local installation resources; maintain **31 preset identifiers** as the research catalog, not a completed effect count.
- Implement layered base / idle / reactive profiles, zones, overlays, and physical keyboard layout precision.
- Faithfully reproduce vendor gradients, color stops, wave/ripple speeds and heatmaps using side-by-side hardware review.
- Expand the GUI with separate preset library, profile management and per-key brush tools as those engine components land.
