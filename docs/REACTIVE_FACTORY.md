# Factory reactive lighting — event envelopes and regression testing

**Implemented on main:** [PR #14](https://github.com/newnetmp3/ApexProMini_gg_rgb/pull/14), merge SHA `37a8ebb2363eb6715a4e7aeb956c1a0dd193cf9e`; all Rust/Clippy/GUI/Python/gallery PR checks passed. Owner hardware test remains deferred.

**Scope:** the three historical SteelSeries GG Prism factory selectors `gg:reactiveRipple`, `gg:reactiveLine`, and `gg:reactiveSingleKey`. Owner is remote, so this milestone only uses read-only tests/preview generation. The already **hardware-verified native `ripple`** remains unchanged.

## Metadata that was recovered

The authorized local GG 108.3 Prism migration provided a nominal period for each factory effect:

| Historical selector | Nominal period | New independent behavior |
|---|---:|---|
| `gg:reactiveRipple` | 250 ms | Circular expanding pulse from the pressed key, with fading wavefront |
| `gg:reactiveLine` | 500 ms | Horizontal expanding pulse confined primarily to the pressed key's row |
| `gg:reactiveSingleKey` | 1,250 ms | Only the pressed key lights, then fades until the period expires |

**Source fact vs original interpretation:** the selector and period are source-derived. Our wavefront width, travel distance, fade curve, blend policy, and normalized keyboard coordinates are **independent approximations**, not SteelSeries shader implementations. For a faithful port, actual GG spatial masks and fade curves require validation with the correct keyboard.

## Shared pure renderer

`src/reactive_factory.rs` implements three distinct trigger shapes and source-duration-scaled effects. `speed` (bounded by user profile validation to 0.1–5.0) scales duration; it does not change key identity. Simultaneous or repeated keypresses combine using bounded maximum intensity instead of allowing RGB overflows. Events with invalid key IDs, future timestamps, or expired lifetimes contribute no light.

Both the CLI daemon and GUI simulation use an event-history lifetime derived from the slowest selected base or reactive overlay. The previous **fixed 2.5-second pruning window** truncated a `gg:reactiveSingleKey` animation at 0.1× speed (which should retain its source-relative envelope for 12.5 seconds). With the new per-preset window, the slow decay reaches its intended end before its event is discarded; the existing native `ripple` retains its original 2.5-second history and unchanged renderer.

There is still a bound of 32 recent keypress events in the live animation loop. Heavy simultaneous typing may evict older effects rather than capture an unbounded event history; no keypress data is logged or persisted.

## Offline verification

The CI suite covers source nominal periods, speed scaling, single-key isolation, correct expiry, radial propagation, row-constrained line propagation, two-key interference, invalid/future event handling, source-speed event-history retention and unchanged previous native ripple output.

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --release --bin prism-preview -- --output prism-presets-preview.svg
```

The **SVG preview gallery** now samples factory reactives at 8%, 40% and 80% of their source nominal trigger period instead of showing every factory reactive at the same three absolute times. For example, `gg:reactiveSingleKey` is sampled at 0.1, 0.5 and 1.0 seconds, while `gg:reactiveRipple` is sampled at 0.02, 0.10 and 0.20 seconds. These frames are synthesized from A-key events and do not open `/dev/input`, `/dev/hidraw`, the GUI, or the network.

## Provisional physical-position improvement

Factory `gg:reactiveRipple` and `gg:reactiveLine` now use `src/layout.rs` width-aware key centers instead of treating Space, Shift, Enter and Backspace as equally spaced 1u keys. Their origin and destination coordinates are now consistent with factory RGB rendering and the read-only SVG gallery. Nominal trigger periods and independently modeled wavefront math are unchanged. **The owner-confirmed native `ripple` continues to use its previously working mapping.** This is provisional ANSI-style 60% geometry, not a calibrated physical measurement or a verified match to GG's own keyboard shader. See [KEY_GEOMETRY.md](KEY_GEOMETRY.md).

## Remaining fidelity tasks

1. Reconstruct actual GG reactive aperture shape and pixel-mask falloff from authorized comparative output, instead of treating the independent Gaussian front as exact.
2. Verify factory layer-over-background composition and how simultaneous events overlap in GG, including any priority or repetition rules.
3. Calibrate the wired Mini's exact physical key positions and the 36 multi-variant graphics region masks.
4. Obtain analog actuation readings only after a verified read-only protocol exists; do not fabricate Hall-sensor HID commands to implement the true GG heatmap.
5. Validate timing and per-key output on the owner's real keyboard after remote work ends. **Native reactive ripple working on hardware does not establish parity of these factory selectors.**

No proprietary GG assets, decrypted device files or passwords are distributed in this module or document.
