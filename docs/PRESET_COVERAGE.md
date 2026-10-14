# SteelSeries GG / Prism preset coverage plan

**Goal:** reproduce **every built-in lighting preset actually shipped with SteelSeries GG** for the wired Apex Pro Mini, including base, idle, and reactive layers, in a native Linux application.

## Evidence categories

- **Hardware-confirmed:** the owner observed that A turns red and S turns green via the direct Feature command, and that the Output clear command restores normal lighting.
- **Implemented with offline tests:** the native effect renderers below, using complete 68-entry frames.
- **Awaiting live testing:** any animated, full-frame, multi-key, or keypress-driven effects.
- **Not inventoried:** the exact vendor preset list, default gradients, timing curves, palette positions, per-key masks, and GG version differences. **Do not claim parity for any of these until compared against official local data.**

## First native renderer set

These are **our provisional descriptive effect names**, not assertions about GG's exact preset titles or equivalence.

| Native effect | Runtime | User settings | Hardware verification |
|---|---|---|---|
| `solid` | One static full-keyboard frame | base color, brightness | Pending whole-keyboard test |
| `gradient` | Static 2-color left-right blend | base, accent, brightness | Pending |
| `rainbow` | Animated spatial multi-hue cycle | speed, brightness | Pending |
| `wave` | Animated moving rainbow wave | speed, brightness | Pending |
| `breathing` | Whole-keyboard rhythmic brightness | base, speed, brightness | Pending |
| `color-shift` | Animated blend between two colors | base, accent, speed, brightness | Pending |
| `sparkle` | Deterministic shimmering individual LEDs | base, accent, speed, brightness | Pending |
| `ripple` | Physical keydown starts expanding colored spatial ring | base, accent, speed, brightness; **USB input node** | Pending keypress test |
| `reactive` | Physical keydown lights pressed LED then fades | base, accent, speed, brightness; **USB input node** | Pending keypress test |

`ripple` and `reactive` use **non-grabbing Linux evdev input** scoped to the identified wired Apex Pro Mini USB device; on Wayland they do not require a compositor-wide hotkey hook. Key data is never logged, persisted, or transmitted. Input-device access depends on local seat/udev permissions.

The approximate US-60% spatial layout presently used for position-dependent effects covers all 68 logical HID entries (including international variants). Accurate physical coordinates, legends, and region layouts still need visual validation.

## Roadmap to faithful GG default preset parity

1. **Inventory shipped presets:** extract *preset names and parameter JSON* from the owner's local GG 121 Windows / GG 108 Mac Prism resources and/or a GG-created Prism database. Record official names/version and obtain screenshots or settings samples. The actual installed defaults must come from the licensed user's installation, not guesses.
2. **Define a versioned preset catalog:** separate canonical effect engine primitives from GG's supplied presets (parameters, colors, gradient stops, time domains, random seeds, masks, overlay blends).
3. **Multi-layer lighting:** support GG `base`, `idle`, and `reactive` states with independent effects and transition times; support multi-key filters/zones.
4. **Faithful effect fidelity:** tune wave orientation, rainbow palette, ripple falloff/speed, blooming, sparkling, gradient interpolation and breathing timing against GG recordings or exported configurations.
5. **Profile import/export:** implement a native profile schema and importer for GG Prism JSON, without redistributing proprietary embedded preset assets.
6. **GUI/editor:** keyboard-map preview, preset selector, per-key selection, gradient editor, brightness/speed sliders, save/apply and resume.
7. **Acceptance matrix:** compare each shipped GG preset on the owner's Mini to our implementation by key and over time; label unavailable input API or firmware variants separately. Only then mark **full preset parity**.

**No data from the Windows/Mac proprietary installers is committed.** Research uses official resources only locally, and the published renderer code is independent.
