# SteelSeries GG / Prism preset coverage plan

**Goal:** reproduce **every built-in lighting preset actually shipped with SteelSeries GG** for the wired Apex Pro Mini, including base, idle, and reactive layers, in a native Linux application.

## Evidence categories

- **Hardware-confirmed:** A turns red; S turns green; the full keyboard turns solid blue; the Output clear command restores normal lighting; and **keypress ripple produces visible effects** following the Mini input permission fix.
- **Implemented with offline tests:** the native effect renderers below, using complete 68-entry frames.
- **Awaiting live testing:** other animated modes, simultaneous multicolor frames, exact physical ripple layout and multi-press behavior, idle/reactive layering, and long-running daemon operation. Full-frame solid color and generic keypress ripple are already observed working.
- **Not inventoried:** the exact vendor preset list, default gradients, timing curves, palette positions, per-key masks, and GG version differences. **Do not claim parity for any of these until compared against official local data.**

## First native renderer set

These are **our provisional descriptive effect names**, not assertions about GG's exact preset titles or equivalence.

| Native effect | Runtime | User settings | Hardware verification |
|---|---|---|---|
| `solid` | One static full-keyboard frame | base color, brightness | **Verified: entire keyboard solid blue** |
| `gradient` | Static 2-color left-right blend | base, accent, brightness | Pending |
| `rainbow` | Animated spatial multi-hue cycle | speed, brightness | Pending |
| `wave` | Animated moving rainbow wave | speed, brightness | Pending |
| `breathing` | Whole-keyboard rhythmic brightness | base, speed, brightness | Pending |
| `color-shift` | Animated blend between two colors | base, accent, speed, brightness | Pending |
| `sparkle` | Deterministic shimmering individual LEDs | base, accent, speed, brightness | Pending |
| `ripple` | Physical keydown starts expanding colored spatial ring | base, accent, speed, brightness; **USB input node** | **Keypress-triggered lighting working (owner-confirmed); exact spatial propagation and GG fidelity pending** |
| `reactive` | Physical keydown lights pressed LED then fades | base, accent, speed, brightness; **USB input node** | Pending keypress test |
| `comet` | Moving illuminated head with fading trail | base, accent, speed, brightness | Pending; independent prototype |
| `rainbow-split` | Mirrored shifting rainbow | speed, brightness | Pending; independent prototype |
| `prismatic-sweep` | Multi-axis hue field | speed, brightness | Pending; independent prototype |
| `heatmap` | Keypress activity fades into colored hotspots | base, accent, speed, brightness | Pending; independent prototype |
| `reactive-line` | Keypress emits a luminous line across the key's row | base, accent, speed, brightness | Pending; independent prototype |

`ripple` and `reactive` use **non-grabbing Linux evdev input** scoped to the identified wired Apex Pro Mini USB device; on Wayland they do not require a compositor-wide hotkey hook. Key data is never logged, persisted, or transmitted. Input-device access depends on local seat/udev permissions.

The approximate US-60% spatial layout presently used for position-dependent effects covers all 68 logical HID entries (including international variants). Accurate physical coordinates, legends, and region layouts still need visual validation.

## Native GUI foundation

The first `apex-mini-gui` (Rust eframe/egui) editor exposes fourteen native renderer prototypes (original nine, plus comet, mirrored rainbow, prismatic sweep, keypress heatmap, and reactive line), a simulated keyboard-map preview, validated color/timing controls, and static overrides painted onto individual keys. It controls the **single already-working systemd user daemon** through a saved per-user profile, never writing HID concurrently. A 31-identifier historical GG preset catalog is visible in the GUI **as unimplemented or prototype-only**, not as 31 falsely working effects.

**Pending on-device QA:** desktop rendering on KDE Wayland, applying edited profile and per-key override, persistence, actual animated effects, service stop/restoration. Only `solid` and `ripple` have prior hardware confirmation, separate from the new GUI.

## Official GG 108.3.0 Prism preset identifier inventory (31)

**Source:** owner's locally inspected `SteelSeriesGG108.3.0.pkg` → PrismSyncV2 package → bundled Prism database migration SQL. This table reports **31 distinct `name_locale_key` identifiers referenced by the bundled migrations**, not necessarily 31 presets simultaneously available in the UI. Presets can be version-dependent, deleted, hidden, or special-case defaults. The exact visibility and per-device compatibility still require inspecting GG's current database and UI. The public repo contains **only identifiers and metadata**, not proprietary preset JSON.

| Official preset identifier suffix | Catalog mode | Native coverage |
|---|---|---|
| aqua | Base / idle | Not matched |
| chasingGhosts | Base / idle | Not matched |
| clown | Base / idle | Not matched |
| colorFusion | Base / idle | Not matched |
| comet | Base / idle | Native comet **prototype**, exact GG behavior not matched |
| disabled | Base / idle | Not matched |
| discoMode | Base / idle; bitmap flag | Not matched |
| drain | Base / idle | Not matched |
| faze | Base / idle | Not matched |
| freeway | Base / idle | Not matched |
| haze | Base / idle | Not matched |
| heatmap | Base / idle | Native keypress heatmap **prototype**, exact GG behavior not matched |
| pride | Base / idle | Not matched |
| prism | Base / idle | Native prismatic sweep **prototype**, exact GG behavior not matched |
| radioactive | Base / idle | Not matched |
| rainbow | Base / idle; bitmap flag | Generic rainbow prototype only; **not a parity match** |
| rainbowSplit | Base / idle | Native mirrored rainbow **prototype**, exact GG behavior not matched |
| reactiveLine | Reactive | Native reactive-line **prototype**, exact GG behavior not matched |
| reactiveRipple | Reactive | Native keypress ripple **hardware-confirmed**, but official preset parameters and fidelity **not yet matched** |
| reactiveSingleKey | Reactive | Generic reactive prototype only; **not a parity match** |
| redPulse | Needs version-specific inspection | Not matched |
| selfDestruct | Base / idle | Not matched |
| shavedIce | Base / idle | Not matched |
| solar | Base / idle | Not matched |
| staticFade | Base / idle | Not matched |
| steelseriesOrange | Base / idle | Not matched |
| twilight | Base / idle | Not matched |
| vaporDreams | Base / idle | Not matched |
| wabashAndLake | Needs version-specific inspection | Not matched |
| warpDrive | Base / idle | Not matched |
| westCoast | Base / idle | Not matched |

The three **reactive** identifiers are especially important for keypress-triggered lighting. The native ripple infrastructure now works after the device-only udev access fix, but this is not proof of official preset parity. Our first `ripple` and `reactive` implementations are infrastructure to reach them; we should replicate the official preset's timing, color, origin and propagation from its authorized local configuration data instead of treating generic effects as finished ports.

### Acceptance rules for each official preset

A preset is **complete** only when its vendor-defined name, palette, gradient stops, animation phase/speed, spatial layout, device key coverage, idle/reactive layering, and expected hardware output are documented and compared to GG behavior on the Mini. The initial fourteen native renderer names do **not** increase official-parity completion automatically.

## Roadmap to faithful GG default preset parity

1. **Inventory shipped presets:** extract *preset names and parameter JSON* from the owner's local GG 121 Windows / GG 108 Mac Prism resources and/or a GG-created Prism database. Record official names/version and obtain screenshots or settings samples. The actual installed defaults must come from the licensed user's installation, not guesses.
2. **Define a versioned preset catalog:** separate canonical effect engine primitives from GG's supplied presets (parameters, colors, gradient stops, time domains, random seeds, masks, overlay blends).
3. **Multi-layer lighting:** support GG `base`, `idle`, and `reactive` states with independent effects and transition times; support multi-key filters/zones.
4. **Faithful effect fidelity:** tune wave orientation, rainbow palette, ripple falloff/speed, blooming, sparkling, gradient interpolation and breathing timing against GG recordings or exported configurations.
5. **Profile import/export:** implement a native profile schema and importer for GG Prism JSON, without redistributing proprietary embedded preset assets.
6. **GUI/editor:** keyboard-map preview, preset selector, per-key selection, gradient editor, brightness/speed sliders, save/apply and resume.
7. **Acceptance matrix:** compare each shipped GG preset on the owner's Mini to our implementation by key and over time; label unavailable input API or firmware variants separately. Only then mark **full preset parity**.

**No data from the Windows/Mac proprietary installers is committed.** Research uses official resources only locally, and the published renderer code is independent.
