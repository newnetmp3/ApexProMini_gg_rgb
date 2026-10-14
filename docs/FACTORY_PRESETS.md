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
| Multiple vendor variants / region masks | **36 additional mask/palette variants implemented across eight presets**; current Mini row/column alignment approximate and needs visual QA |
| Vendor compositor: foreground/background graphics, apertures, blending, repetition and origin transforms | **Reusable independent compositor implemented**: ordered layers, opacity, four blend modes, scaled/rotated/origin-based motion, repeated phase and cosine/band masks. Three complex scenes connected. Vendor shader equations and actual per-preset settings **not verified** |
| Per-key physical RGB coordinate/layout fidelity | **Provisional width-aware five-row / 68-HID factory map implemented**; exact LED physical coordinates, historical six-row projection and ISO variants need device-side QA |
| User's existing standalone ripple / service | **Previously confirmed working**; no altered HID transport |
| Full parity to SteelSeries GG Prism default presets | **Not achieved** |

### Exact remaining complexities identified in the official data

Of the 30 historical definitions, **eight have multiple region/layout variants:** `wabashAndLake`, `rainbow`, `colorFusion`, `aqua`, `freeway`, `comet`, `pride`, `twilight`. The extracted variants are associated with distinct key-selection masks or spatial regions; the second-stage renderer now includes **all 36 additional historical region variants**, via `src/regions.rs`, selected by representative row/column masks. The Mini has five physical rows mapped approximately to source logical rows 1–5; variant selection and phase coordinate scales still require device-side calibration.

Additional compositing complexity appears in `westCoast`, `haze`, `warpDrive` and the `reactiveLine`, `reactiveRipple`, and `reactiveSingleKey` presets, which contain background/aperture features. The independent module `src/graphics.rs` now reproduces the **structural distinction** between foreground, background and aperture for the first three using source-derived functional color/period parameters and an original bounded coverage function. **Its cosine-based mask is not the verified GG aperture shader.** Reactive source aperture curves also remain approximate.

The `heatmap` source defines an actuation-triggered special effect with its own heat thresholds; the renderer currently approximates recent key activity, while `heatmap_color_for_actuation` now independently implements the five source-defined analog depth/color threshold bands with boundary unit tests. Live analog actuation values are **not available from the verified evdev interface**, so the true hardware heatmap still needs a safe separately verified input protocol. **Source-level parameter parity is only partial**, despite the 31 working selectors.

## Recommended next development slices (do not wait for GUI QA)

1. **Regional QA and coordinate accuracy:** all 36 alternate row/column variants are now parameterized. Calibrate the source 6-row/22-column Prism grid against the actual wired Mini's 5 physical rows and logical-key usage positions (especially sparse `wabashAndLake` masks), with offline mapping tests followed by owner QA.
2. **Generalized graphics compositor:** implement per-layer foreground/background/aperture/mask, phase transforms and opacity; model vendor stack semantics separately from MIDI/HID report encoding.
3. **Idle and reactive accuracy:** parameterize exact trigger envelopes, cooldown, locally vs globally centered wavefronts, repeat counts and heat decay. Compare with native SteelSeries GG once owner has access.
4. **Multi-device/preset version compatibility:** resolve which historical GG presets are actually shown for the wired Mini under GG 108.3 and 121, identify renamed/deleted entries, and retain versioned per-device compatibility.
5. **Visual/hardware parity evidence:** every factory preset must receive offline golden key/color/timing tests and owner visual comparison; only then mark individual preset rows as **verified equivalent**.

## Source-derived regional preview engine

`src/regions.rs` now contains **36 independently summarized alternate layout variants** covering `rainbow` (6), `colorFusion` (2), `aqua` (5), `freeway` (6), `comet` (5), `pride` (6), `twilight` (3), and `wabashAndLake` (3). This is in addition to the 30 original first-variant definitions plus the `redPulse` alias.

- Per-row masks from a six-row GG grid are mapped to Mini logical rows 1–5; precise physical mapping is **not verified**.
- `wabashAndLake` uses sparse column masks from the source; the Mini's approximate 22-column x grid selects those bands.
- One out-of-order historical `comet` gradient stop was normalized for numeric interpolation.
- The renderer falls back to the base variant for keys without a matching region, never writes an unmapped HID key, and continues using the exact known 513-byte Mini RGB encoder.

Unit tests cover distinct row palettes, mask selection, 31 historical IDs, and full frame encoding. This is a **partial port of regional parameters**, not reproduction of GG's complete underlying graphics compositor.

## Read-only SVG regression gallery (remote QA)

The `prism-preview` executable evaluates **all 31 selectors at three different animation times** and draws each of the Mini's 68 logical LEDs in a standalone SVG page. Reactive modes receive only synthetic A-keypress data in memory. It never opens `/dev/hidraw`, `/dev/input`, a Wayland/X11 display, or network sockets.

```bash
cargo run --release --bin prism-preview -- --output prism-presets-preview.svg
# Open prism-presets-preview.svg in any browser or image viewer.
```

The GitHub Actions `Rust protocol checks` workflow also uploads `apex-mini-prism-factory-previews` as an artifact, useful for remote review without a graphical session or the physical keyboard. This is a **visual regression baseline**, not an official SteelSeries screenshot or proof of exact effect fidelity. Compare revisions by the same time/key matrix; later match against recorded GG behavior.

## Compound-graphics stage: West Coast, Haze and Warp Drive

The official source material for `westCoast` and `haze` contains both foreground and background gradient layers plus a gradient-less geometric aperture. `warpDrive` has foreground and aperture, without a separate background layer.

The new independent `src/graphics.rs` interprets those functional fields with a pure layer compositor. It calculates each foreground/background phase from source-derived duration, wavelength and phase geometry; a bounded time/space mask blends the layers, defaulting to black where no background exists. Pure unit tests verify multiple colors over the keyboard, time variation and full 68-key feature frame compatibility. `prism-preview` uses the updated frames without any GUI or hardware access.

**Fidelity limitation:** Original GG aperture shaders, anchor transforms, opacity and blending are not specified by the configuration alone. The current bounded cosine aperture is a **functional approximation**, not proof of parity. No proprietary graphics shader source is copied or executed.

## Reusable compositor foundation (new independent implementation)

`src/compositor.rs` now supports explicitly ordered background-to-foreground RGB graphics, per-layer opacity, Normal/Add/Multiply/Screen blend modes, independent phase and gradient palettes, scale/rotation/origin transforms, repetition and offsets, and cosine/feathered-band inverted masks. `src/graphics.rs` now describes West Coast, Haze and Warp Drive as declarative scene stacks. Unit tests verify blend endpoints, layer order, transform changes and periodic masks. A small set of numeric RGB fixtures prevents silent changes to our **approximate** compositor.

See [`docs/COMPOSITOR.md`](COMPOSITOR.md) for algorithm documentation and exact remaining acceptance criteria. **These controls are independently defined; they do not mean the historical GG shader code, layer metadata, opacity settings, or physical key layout has been reproduced correctly.**

## Factory reactive trigger-period stage

A new pure `src/reactive_factory.rs` independently models three GG source-named modes with the nominal 250/500/1250ms periods: radial `reactiveRipple`, row-constrained `reactiveLine`, and isolated pressed-key `reactiveSingleKey`. Each pulse has bounded fading and an exact expiration; repeated and simultaneous events combine without overdriving any RGB channel.

To avoid prematurely dropping long source-period key animations at slow profile speed, `Stack::key_history_seconds` informs the daemon's in-memory pruning threshold and the GUI's simulated reactive preview. A Single Key animation slowed to 0.1× now retains its event for up to 12.5 seconds, rather than the earlier 2.5-second cutoff. The owner-confirmed independent native `ripple` retains its original lifetime and rendering.

Offline SVG gallery phases are now sampled **relative to each GG reactive source period**, rather than using identical absolute sample times. All remains device-independent; **actual SteelSeries GG aperture geometry, interpolation, speed semantics, layer priority and fade curve parity are still unknown and should not be claimed**. See [`docs/REACTIVE_FACTORY.md`](REACTIVE_FACTORY.md).

## Five-row Mini width-aware geometry (factory modes only)

The historical renderer's first map spaced all keys uniformly. The independent `src/layout.rs` implementation now models five physical 15u keyboard rows and calculates the centers of large Space/Shift/Enter/Backspace keys using plausible keyboard-unit widths. It locates all 68 distinct Mini HID usages, of which four are optional locale/firmware variants. Factory-specific ripples, lines, analog-free heatmap approximations, source-region masks, and offline SVG previews use this consistent layout.

**The owner-confirmed native ripple's previous coordinates remain unchanged.** The historical Prism `y0..5 / x0..21` source grid is **approximately projected** onto physical rows `0..4` → source rows `1..5`; exact LED positions and GG row/column region mapping cannot be certified without device reference. [Geometry mapping and acceptance](KEY_GEOMETRY.md).

## Safe operation

All rendering/unit tests and `--preview-effect` commands are device-independent and read-only. Actual user service commands still explicitly require `--confirm-device-write`, device VID/PID + HID report checks and Mini-only evdev permissions. No official SteelSeries package, encryption passphrase, decrypted descriptors, original preset JSON or SQL has been committed.
