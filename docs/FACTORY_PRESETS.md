# SteelSeries GG Prism factory preset port — implementation audit

**Research source:** owner-supplied macOS `SteelSeriesGG108.3.0.pkg`, `SteelSeriesPrismSyncV2.pkg`, bundled GG-versioned Prism SQLite migrations. Analysis was performed locally without executing GG, changing the user's keyboard, or committing proprietary SQL/JSON.

**Key finding:** **30 distinct complete historical preset configurations** were present in the examined migrations. The later GG 106 database migration renames the pre-existing `faze` preset's display key to `redPulse`. Therefore the 31 previously identified locale keys represent **30 underlying graphics definitions plus one historical alias**, not 31 independent effects simultaneously available in one release.

## Implemented in this stage

The independent Rust `src/factory.rs` module defines all 31 historical selectors under the `gg:` namespace. The renderer uses per-definition palette stop colors, nominal animation durations, spatial wavelength signs/magnitudes and phase geometry categories summarized from the owner's source resources. It produces the same safe **68-key RGB frame** format already verified on the wired Mini.

Try these **read-only** previews (no keyboard input / HID access):

```bash
cargo test --all-targets
cargo check --features gui --bin apex-mini-gui
cargo run --release -- --preview-effect gg:aqua
cargo run --release -- --preview-effect gg:prism
cargo run --release -- --preview-effect gg:reactiveRipple
```

You can also select each factory-style preview in the native GUI. Profiles accept `effect=gg:aqua` (distinct from the independent `effect=rainbow` prototype), including `gg:faze` and the `gg:redPulse` historical alias.

The profile and single-writer service now accept a base preset plus optional `idle`, `reactive`, and `idle_seconds` fields. Existing `version=1` saved profiles continue to load without those fields. A reactive overlay is transparent when no physical keypress occurs; an idle preset replaces the base after the selected interval without input.

Example user-owned **profile format** (do not apply until ready to test on the actual Mini):

```ini
version=1
effect=gg:aqua
color=081020
accent=ff5500
speed=1.0
brightness=0.75
fps=24
idle=gg:twilight
idle_seconds=30
reactive=gg:reactiveRipple
```

## What is verified and what remains

| Capability | Current status |
|---|---|
| 31 historical GG identifiers independently indexed | **Implemented; Rust unit tests** |
| 30 configurations' first representative palette/timing/spatial fields | **Derived and implemented; unit-tested** |
| `redPulse` as `faze` rename | **Confirmed by GG 106 migration** |
| 68-key full-frame RGB encoding for each selector | **Rust tests**; GG-style effects not individually tested on the owner's keyboard |
| Data-driven sampled gradients, color cycles and simple reactive patterns | **Independent approximations** |
| Base + idle + reactive composition, existing profile compatibility | **Implemented and software-tested** |
| Multiple vendor variants / region masks | **Not yet faithfully supported** |
| Vendor compositor: foreground/background graphics, apertures, blending, repetition and origin transforms | **Not yet faithfully supported** |
| Per-key physical RGB coordinate/layout fidelity | **Requires device-side QA** |
| User's existing standalone ripple / service | **Previously confirmed working**; no altered HID transport |
| Full parity to SteelSeries GG Prism default presets | **Not achieved** |

### Exact remaining complexities identified in the official data

Of the 30 historical definitions, **eight have multiple region/layout variants:** `wabashAndLake`, `rainbow`, `colorFusion`, `aqua`, `freeway`, `comet`, `pride`, `twilight`. The extracted variants are associated with distinct key-selection masks or spatial regions; the first-stage renderer currently uses the **first representative variant** of each preset.

Additional compositing complexity appears in `westCoast`, `haze`, `warpDrive` and the `reactiveLine`, `reactiveRipple`, and `reactiveSingleKey` presets, which contain background/aperture features. The exact mathematical implementation of GG's compositor has **not** been reproduced; our current effect phase and layer blending code is an original approximation designed for portability.

The `heatmap` source defines an actuation-triggered special effect with its own heat thresholds; the first renderer approximates recent key activity, not those exact thresholds. **Source-level parameter parity is only partial**, despite the 31 working selectors.

## Recommended next development slices (do not wait for GUI QA)

1. **Regional variants:** derive independent row/key masks from layout-specific segments and apply the remaining stored gradients per supported physical Mini key, starting with `pride`, `rainbow`, `aqua`, `freeway`, and `comet`.
2. **Generalized graphics compositor:** implement per-layer foreground/background/aperture/mask, phase transforms and opacity; model vendor stack semantics separately from MIDI/HID report encoding.
3. **Idle and reactive accuracy:** parameterize exact trigger envelopes, cooldown, locally vs globally centered wavefronts, repeat counts and heat decay. Compare with native SteelSeries GG once owner has access.
4. **Multi-device/preset version compatibility:** resolve which historical GG presets are actually shown for the wired Mini under GG 108.3 and 121, identify renamed/deleted entries, and retain versioned per-device compatibility.
5. **Visual/hardware parity evidence:** every factory preset must receive offline golden key/color/timing tests and owner visual comparison; only then mark individual preset rows as **verified equivalent**.

## Safe operation

All rendering/unit tests and `--preview-effect` commands are device-independent and read-only. Actual user service commands still explicitly require `--confirm-device-write`, device VID/PID + HID report checks and Mini-only evdev permissions. No official SteelSeries package, encryption passphrase, decrypted descriptors, original preset JSON or SQL has been committed.
