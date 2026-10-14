# Wired Apex Pro Mini: provisional width-aware factory-effect geometry

**Milestone:** `feature/factory-key-geometry-v2`. Owner remains remote, so this implementation is **software-tested only**. It does not claim to reproduce the vendor's original coordinate map, nor any physical keyboard optical measurements.

## Reason for change

The first native effect implementation placed every HID usage at the same horizontal interval, including 2u Backspace, 2.25u Enter, 2.25u/2.75u Shift, and the large Space bar. A keypress ripple uses the centers of those LEDs as its expansion origins. This produces visible spatial distortions, especially when effects begin near Space or Shift. It also affects selection of 22-column, six-row historical Prism regional palettes.

**The existing user-confirmed native `ripple` has not been changed.** Factory selectors `gg:<name>` now use a **separate**, independent geometry model, so the new map can be calibrated without jeopardizing the existing live background service.

## Implemented source-aware geometry

`src/layout.rs` defines an ANSI-style conventional five-row keyboard in **15 keyboard-width units**:

| Physical row | HID model | Key widths / layout assumptions |
|---|---|---|
| 0 | Escape, 1–0, minus, equals, Backspace | 1u standard keys, 2u Backspace |
| 1 | Tab, Q–P, brackets, backslash | 1.5u Tab and backslash |
| 2 | Caps, A–L, punctuation, Enter | 1.75u Caps and 2.25u Enter |
| 3 | Shift, Z–M, punctuation, right Shift | 2.25u left Shift and 2.75u right Shift |
| 4 | Bottom modifiers, large Space and optional firmware variants | 5.5u Space, variable modifier widths |

The five rows remain centered in a **normalized 0–1 x/y plane**, but long keys now have appropriate physical midpoint coordinates and widths. Factory ripple, factory reactive line, historical region masks, factory heatmap approximation and the read-only 31-preset SVG gallery share those coordinates.

The 68 known Mini addressable HID codes comprise **64 primary positions plus four layout/firmware alternates**: `137` near Backspace, `50` near Enter, `100` beside left Shift, and `135` near right Shift. The exact presence and locations of these alternatives are **not verified for the owner's physical keyboard**. Some alternate center positions overlap their corresponding full-width key, rather than creating a nonexistent sixth row.

### Historical source-grid approximation

The original catalog has region descriptions on a **six-row by 22-column Prism grid**, whereas the physical Mini model has five rows. The current provisional correspondence is:

- Physical Mini rows 0–4 → historical source rows 1–5; historical source row 0 is absent.
- Physical key center x multiplied by 21 and rounded → historical source column 0–21.
- Source `src/regions.rs` uses the resulting positions for the 36 alternate regional palettes. This is an **approximate calibration**, not an official physical address map.

## Offline verification

The tests assert:

1. Each of the five rows covers exactly 15 keyboard units; all 68 verified HID usages map once to a row or explicit alternate, with no unknown HID codes.
2. All normalized coordinates remain within the board.
3. Space is centered and is at least 5u wide; wide keys have appropriate widths and the A/S/W relationships are maintained.
4. Factory reactive wavefronts use the same width-aware geometry as their target LEDs. Same-row versus other-row keypress propagation and exact-key isolation remain covered.
5. Every factory selector still renders a 68-entry color frame serializable into the established 513-byte Mini HID Feature report.
6. The headless SVG gallery uses variable-width key rectangles instead of rendering Space as a short 1u key.

Run without USB, evdev, a GUI or any hardware writes:

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --release --bin prism-preview -- --output prism-presets-preview.svg
```

## What remains for precise GG parity

- Retrieve layout-specific device LED/key x/y positions from authorized descriptors or user-side read-only measurements; do **not** substitute assumptions for vendor facts.
- Validate actual LED positions under Space, Shift and ISO alternatives on the wired Mini, and calibrate the 22-column historical masks by key rather than interpolating from generic keycap widths.
- Verify keycap size/shape and spacing under the native KDE GUI keyboard preview when the owner is back at the device; GUI rendering is **deferred**, not considered verified.
- Compare factory reactive Ripple/Line/Single Key animation geometry and fades against matching GG version reference effects with the same physical layout.
- Preserve user-verified standalone `ripple` unless owner explicitly agrees to migrate it and confirms a regression-safe keyboard test.

**Safety:** No VID/PID, USB protocol, feature packet bytes, daemon lifecycle, keylogger behavior, firmware or actuation settings have been changed. This module contains original explanatory geometry, not vendor graphical assets.
