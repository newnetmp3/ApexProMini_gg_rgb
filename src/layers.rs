//! Layered independent lighting compositor for base, idle and reactive modes.
//!
//! The original HID transport stays unchanged. This pure renderer can be
//! fuzzed/tested offline; each tick still emits one 68-key frame. GG's exact
//! blending curves and animation masks are separate parity milestones.
use crate::effects::{self, Config, KeyHit, Preset};
use crate::{KeyColor, Rgb};

#[derive(Clone, Copy, Debug)]
pub struct Stack {
    pub base: Preset,
    pub idle: Option<Preset>,
    pub reactive: Option<Preset>,
    /// How long after the last keypress until the idle mode replaces base.
    pub idle_after_seconds: f32,
}

impl Stack {
    pub fn needs_input(self) -> bool {
        self.base.needs_key_events() || self.idle.is_some() || self.reactive.is_some()
    }

    /// Retain events for the longest configured animation, including a
    /// factory reactive overlay. Does not affect the actual RGB serializer.
    pub fn key_history_seconds(self, speed: f32) -> f32 {
        let base = self.base.key_history_seconds(speed);
        let idle = self.idle.map_or(2.5, |p| p.key_history_seconds(speed));
        let reactive = self.reactive.map_or(2.5, |p| p.key_history_seconds(speed));
        base.max(idle).max(reactive)
    }
    pub fn render(self, config: Config, now: f32, last_key: f32, hits: &[KeyHit]) -> Vec<KeyColor> {
        let idling = self.idle.is_some()
            && now - last_key >= self.idle_after_seconds.max(0.1);
        let background = if idling {
            self.idle.unwrap_or(self.base)
        } else {
            self.base
        };
        let mut result = effects::frame(background, config, now, hits);
        let Some(overlay) = self.reactive else { return result; };
        if hits.is_empty() { return result; }
        let baseline = effects::frame(overlay, config, now, &[]);
        let triggered = effects::frame(overlay, config, now, hits);
        for ((key, before), during) in result.iter_mut().zip(baseline.iter()).zip(triggered.iter()) {
            // Difference-from-neutral alpha means the reactive layer is
            // transparent between keypresses, not a constant color overlay.
            let delta = [
                during.rgb.red.abs_diff(before.rgb.red),
                during.rgb.green.abs_diff(before.rgb.green),
                during.rgb.blue.abs_diff(before.rgb.blue),
            ].into_iter().max().unwrap_or(0);
            let weight = (delta as f32 / 255.0).clamp(0.0, 1.0);
            key.rgb = blend(key.rgb, during.rgb, weight);
        }
        result
    }
}

fn blend(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    Rgb { red: f(a.red, b.red), green: f(a.green, b.green), blue: f(a.blue, b.blue) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode_direct_write, factory};

    #[test]
    fn plain_existing_ripple_is_not_changed_when_layers_disabled() {
        let stack = Stack {
            base: Preset::Ripple, idle: None, reactive: None, idle_after_seconds: 15.0
        };
        let hits = [KeyHit { hid: 4, when: 1.0 }];
        for now in [0.0, 1.1, 4.0] {
            let before = effects::frame(Preset::Ripple, Config::default(), now, &hits);
            let after = stack.render(Config::default(), now, 1.0, &hits);
            assert_eq!(before.len(), after.len());
            assert!(before.iter().zip(after).all(|(a, b)| a.hid == b.hid && a.rgb == b.rgb));
        }
    }

    #[test]
    fn idle_switches_after_inactivity_and_returns_after_keypress() {
        let settings = Config { brightness: 1.0, ..Config::default() };
        let stack = Stack {
            base: Preset::Solid,
            idle: Some(Preset::Factory(factory::find("gg:steelseriesOrange").unwrap())),
            reactive: None, idle_after_seconds: 5.0,
        };
        let active = stack.render(settings, 6.0, 4.0, &[]);
        let idle = stack.render(settings, 6.0, 0.0, &[]);
        let resumed = stack.render(settings, 6.0, 6.0, &[]);
        assert!(active.iter().all(|key| key.rgb == settings.base));
        assert!(idle.iter().all(|key| key.rgb != settings.base));
        assert!(resumed.iter().all(|key| key.rgb == settings.base));
    }

    #[test]
    fn overlay_changes_key_colors_only_after_keypress() {
        let settings = Config { brightness: 1.0, ..Config::default() };
        let stack = Stack { base: Preset::Solid, idle: None,
            reactive: Some(Preset::Factory(factory::find("gg:reactiveRipple").unwrap())),
            idle_after_seconds: 30.0 };
        let base = stack.render(settings, 0.02, 0.0, &[]);
        let active = stack.render(settings, 0.02, 0.0, &[KeyHit { hid: 4, when: 0.0 }]);
        assert!(base.iter().zip(active.iter()).any(|(a, b)| a.rgb != b.rgb));
        assert_eq!(encode_direct_write(&active).unwrap()[2], 68);
    }

    #[test]
    fn stack_keeps_slow_factory_reactive_overlay_event_history() {
        let native = Stack { base: Preset::Ripple, idle: None, reactive: None,
            idle_after_seconds: 15.0 };
        assert_eq!(native.key_history_seconds(0.1), 2.5);
        let layered = Stack { base: Preset::Solid, idle: None,
            reactive: Some(Preset::from_name("gg:reactiveSingleKey").unwrap()),
            idle_after_seconds: 15.0 };
        assert!((layered.key_history_seconds(0.1) - 12.5).abs() < 1e-5);
    }
    #[test]
    fn input_required_when_idle_or_reactive_uses_events() {
        assert!(!Stack { base: Preset::Solid, idle: None, reactive: None,
            idle_after_seconds: 30.0 }.needs_input());
        assert!(Stack { base: Preset::Solid, idle: Some(Preset::Gradient), reactive: None,
            idle_after_seconds: 30.0 }.needs_input());
        assert!(Stack { base: Preset::Solid, idle: None, reactive: Some(Preset::Reactive),
            idle_after_seconds: 30.0 }.needs_input());
    }
}
