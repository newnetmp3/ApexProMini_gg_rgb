# Independent Prism-compatible graphic compositor

This document describes our **open, independently implemented rendering engine**, not the proprietary SteelSeries GG graphics implementation.

**Milestone:** `feature/prism-layer-compositor-v2`, following [PR #12](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/12). Development is deliberately headless while the owner is remote.

## Composition model

`src/compositor.rs` implements reusable, pure, testable ingredients:

| Component | Behavior | Status |
|---|---|---|
| Color palette | Ordered RGB stop interpolation at U8 phase positions | Implemented, exact vendor color space unknown |
| Motion | Static or animated cyclic phase, signed wavelength | Implemented, mathematical equivalence unverified |
| Spatial origin | Independent phase origin coordinates | Implemented |
| Spatial transform | Per-axis scale and rotation | Implemented |
| Repetition / offset | Multiply spatial cycles and offset the phase | Implemented |
| Layer stack | Back-to-front ordered graphics against a defined clear color | Implemented |
| Layer opacity | Continuous opacity 0–1 | Implemented |
| Blend modes | Normal, Add, Multiply, Screen | Implemented as independently defined sRGB channel operations |
| Aperture | Cosine coverage, feathered moving band and optional invert | Implemented; these **are not GG's original mask shaders** |
| Mini device transport | 68-key frame serialized to 513-byte Feature HID | Previously hardware-verified and unchanged |

Each frame is evaluated per RGB key without device I/O in the compositor. Layers render sequentially, so changing their order is measurable. The final RGB value is always bounded to the 0–255 range for each channel. The renderer itself keeps no keystrokes or files.

### Scene recipes currently connected

The existing `src/graphics.rs` now defines three compound presets declaratively using `compositor.rs`: `gg:westCoast` (background gradient, foreground gradient, mask); `gg:haze` (animated background, magenta foreground, slower mask); and `gg:warpDrive` (foreground and mask against black).

**Crucial limitation:** Configuration metadata supplied gradient colors, animation periods, and spatial wavelength categories, but not the vendor's complete shader logic. The independently implemented masks and transforms are **approximations**, so these scenes are functional but not certified as GG-equivalent. All 31 historic factory selectors remain available; other ones still use the simpler factory renderer.

## Offline tests and acceptance

The Rust suite tests sequential layer ordering, alpha endpoints, four blend modes, origin shifts, rotation, spatial repeats, offset, positive/negative wave direction, inverted masks, RGB palette edge conditions, and complete 68-key output. Specific numerical color fixtures for `westCoast` and `haze` guard against unintended renderer changes, but do not stand in for official GG screenshots.

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --release --bin prism-preview -- --output prism-presets-preview.svg
```

The last command is **read-only** and works without the keyboard or GUI. CI uploads an SVG artifact with 31 historic selectors × three frames per selector.

## Fidelity roadmap (remaining)

1. Recover or infer original behavior of aperture masks through authorized GG sample observations and configuration research, then write golden offline key/color/timing fixtures rather than guessing the original shader.
2. Compare real source graphic order, opacity, mask repetition and anchoring per preset; map these verified settings to declarative scene recipes.
3. Expand the reusable compositor to remaining complex native source scenes, especially GG reactive line/ripple/single-key overlays.
4. Correct the six-row logical GG-to-five-row Mini geometry and the 36 regional mask placements.
5. Verify Hall actuation readback before enabling GG's true actuation-based heatmap. There are no invented sensor HID commands.
6. Once the owner returns to the physical keyboard, compare every factory-style mode with the appropriate GG version and board layout. Mark **exact preset parity** only after passing side-by-side tests.

Existing owner-confirmed native ripple, the user-level systemd service, and underlying HID command format are unaffected by this rendering-only change. No vendor executable, proprietary GLSL/shader code, encrypted device descriptor, password or preset JSON is published.
