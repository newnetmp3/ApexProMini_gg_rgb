//! Independent effect renderer for wired Apex Pro Mini per-key RGB.
//! No USB access, IO, or mutable global state: all animation frames are testable offline.
use crate::{KeyColor, Rgb, ADDRESSABLE_HID_CODES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Solid,
    Breathing,
    Rainbow,
    Wave,
    ColorShift,
    Gradient,
    Ripple,
    Reactive,
    Sparkle,
}
impl Preset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid", Self::Breathing => "breathing", Self::Rainbow => "rainbow",
            Self::Wave => "wave", Self::ColorShift => "color-shift", Self::Gradient => "gradient",
            Self::Ripple => "ripple", Self::Reactive => "reactive", Self::Sparkle => "sparkle",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "solid" => Some(Self::Solid),
            "breathing" => Some(Self::Breathing),
            "rainbow" => Some(Self::Rainbow),
            "wave" => Some(Self::Wave),
            "color-shift" => Some(Self::ColorShift),
            "gradient" => Some(Self::Gradient),
            "ripple" => Some(Self::Ripple),
            "reactive" => Some(Self::Reactive),
            "sparkle" => Some(Self::Sparkle),
            _ => None,
        }
    }
    pub fn needs_key_events(self) -> bool {
        matches!(self, Self::Ripple | Self::Reactive)
    }
}

pub const PRESET_NAMES: &str =
    "solid, breathing, rainbow, wave, color-shift, gradient, sparkle, ripple, reactive";

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub base: Rgb,
    pub accent: Rgb,
    pub speed: f32,
    pub brightness: f32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            base: Rgb { red: 18, green: 30, blue: 65 },
            accent: Rgb { red: 0, green: 230, blue: 255 },
            speed: 1.0,
            brightness: 0.75,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyHit {
    pub hid: u8,
    /// Seconds elapsed since effect start; event data is not stored to disk.
    pub when: f32,
}

fn blend(a: Rgb, b: Rgb, fraction: f32) -> Rgb {
    let f = fraction.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 * (1.0 - f) + y as f32 * f).round() as u8;
    Rgb {
        red: mix(a.red, b.red),
        green: mix(a.green, b.green),
        blue: mix(a.blue, b.blue),
    }
}
fn dim(rgb: Rgb, scale: f32) -> Rgb {
    let f = scale.clamp(0.0, 1.0);
    Rgb {
        red: (rgb.red as f32 * f).round() as u8,
        green: (rgb.green as f32 * f).round() as u8,
        blue: (rgb.blue as f32 * f).round() as u8,
    }
}
fn hue(h: f32) -> Rgb {
    let h = h.rem_euclid(1.0) * 6.0;
    let x = 255.0 * (1.0 - ((h % 2.0) - 1.0).abs());
    let x = x.round() as u8;
    match h as u32 {
        0 => Rgb { red: 255, green: x, blue: 0 },
        1 => Rgb { red: x, green: 255, blue: 0 },
        2 => Rgb { red: 0, green: 255, blue: x },
        3 => Rgb { red: 0, green: x, blue: 255 },
        4 => Rgb { red: x, green: 0, blue: 255 },
        _ => Rgb { red: 255, green: 0, blue: x },
    }
}

/// Approximate spatial mapping of all 68 HID usages listed by the official wired
/// Mini definition, including international-layout variants. Five physical rows.
/// This is used only for effects and requires physical geometry QA later.
pub fn position(hid: u8) -> Option<(f32, f32)> {
    let rows: [&[u8]; 5] = [
        &[41, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 45, 46, 42, 137],
        &[43, 20, 26, 8, 21, 23, 28, 24, 12, 18, 19, 47, 48, 49],
        &[57, 4, 22, 7, 9, 10, 11, 13, 14, 15, 51, 52, 40, 50],
        &[225, 100, 29, 27, 6, 25, 5, 17, 16, 54, 55, 56, 229, 135],
        &[224, 227, 226, 139, 44, 138, 136, 230, 231, 240, 228],
    ];
    for (row, keys) in rows.iter().enumerate() {
        if let Some(col) = keys.iter().position(|&key| key == hid) {
            let extra = match row { 1 => 0.35, 2 => 0.50, 3 => 0.75, 4 => 0.25, _ => 0.0 };
            return Some(((col as f32 + extra) / 14.0, row as f32 / 4.0));
        }
    }
    None
}
fn coords(hid: u8) -> (f32, f32) {
    position(hid).unwrap_or((0.5, 0.5))
}

/// A complete 68-entry key frame, in stable mapping order. Hardware receives
/// exactly one 513-byte Feature write for each rendered frame.
pub fn frame(preset: Preset, settings: Config, seconds: f32, hits: &[KeyHit]) -> Vec<KeyColor> {
    let t = seconds.max(0.0) * settings.speed;
    ADDRESSABLE_HID_CODES.iter().copied().map(|hid| {
        let (x, y) = coords(hid);
        let rgb = match preset {
            Preset::Solid => settings.base,
            Preset::Breathing => dim(settings.base, 0.07 + 0.93 * (0.5 + 0.5 * (t * 3.2).sin())),
            Preset::Rainbow => hue(x * 0.9 + y * 0.13 + t * 0.11),
            Preset::Wave => hue(x * 0.95 + y * 0.2 - t * 0.23),
            Preset::ColorShift => blend(settings.base, settings.accent, 0.5 + 0.5 * (t * 2.0).sin()),
            Preset::Gradient => blend(settings.base, settings.accent, x),
            Preset::Sparkle => {
                // Deterministic spatial glitter: no RNG or dependencies.
                let seed = ((hid as f32 * 71.133 + (t * 6.0).floor() * 17.71).sin() * 17891.0).fract().abs();
                blend(dim(settings.base, 0.30), settings.accent, if seed > 0.91 { 1.0 } else { 0.0 })
            }
            Preset::Ripple | Preset::Reactive => {
                let mut intensity = 0.0_f32;
                for hit in hits {
                    // Physical time in seconds: speed modifies animation propagation.
                    let age = seconds - hit.when;
                    if !(0.0..=2.5).contains(&age) { continue; }
                    let (hx, hy) = coords(hit.hid);
                    let distance = ((x - hx).powi(2) + ((y - hy) * 0.9).powi(2)).sqrt();
                    let value = if preset == Preset::Ripple {
                        let ring = distance - age * settings.speed * 0.8;
                        (-((ring / 0.105).powi(2))).exp() * (1.0 - age / 2.5)
                    } else if hid == hit.hid {
                        (1.0 - age * settings.speed / 1.1).clamp(0.0, 1.0)
                    } else { 0.0 };
                    intensity = intensity.max(value);
                }
                blend(dim(settings.base, 0.18), settings.accent, intensity)
            }
        };
        KeyColor { hid, rgb: dim(rgb, settings.brightness) }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_direct_write;

    #[test]
    fn all_presets_produce_valid_complete_frames() {
        for name in PRESET_NAMES.split(", ") {
            let effect = Preset::from_name(name).unwrap();
            for t in [0.0, 0.2, 3.5] {
                let colors = frame(effect, Config::default(), t, &[]);
                assert_eq!(colors.len(), 68);
                let report = encode_direct_write(&colors).unwrap();
                assert_eq!(report[2], 68);
                assert_eq!(report.len(), 513);
            }
        }
    }

    #[test]
    fn mapping_covers_all_official_hid_codes() {
        for &hid in &ADDRESSABLE_HID_CODES { assert!(position(hid).is_some(), "Missing coordinate for HID {hid}"); }
        assert!(position(0).is_none());
    }

    #[test]
    fn ripple_origin_and_propagation_are_observable() {
        let base = Config {
            base: Rgb { red: 0, green: 0, blue: 0 },
            accent: Rgb { red: 255, green: 0, blue: 0 },
            speed: 1.0,
            brightness: 1.0,
        };
        let hit = KeyHit { hid: 4, when: 0.0 };
        let keys = frame(Preset::Ripple, base, 0.0, &[hit]);
        let a = keys.iter().find(|k| k.hid == 4).unwrap();
        let m = keys.iter().find(|k| k.hid == 16).unwrap();
        assert!(a.rgb.red > m.rgb.red);
        let later = frame(Preset::Ripple, base, 0.5, &[hit]);
        let a_later = later.iter().find(|k| k.hid == 4).unwrap();
        assert!(a_later.rgb.red < a.rgb.red);
    }

    #[test]
    fn static_modes_are_time_invariant() {
        for preset in [Preset::Solid, Preset::Gradient] {
            let a = frame(preset, Config::default(), 0.0, &[]);
            let b = frame(preset, Config::default(), 4.0, &[]);
            assert_eq!(a.len(), b.len());
            assert_eq!(a.iter().map(|x| x.rgb.red).collect::<Vec<_>>(),
                       b.iter().map(|x| x.rgb.red).collect::<Vec<_>>());
        }
    }

    #[test]
    fn brightness_zero_is_dark() {
        let settings = Config { brightness: 0.0, ..Config::default() };
        let colors = frame(Preset::Rainbow, settings, 2.0, &[]);
        assert!(colors.iter().all(|k| k.rgb == Rgb { red: 0, green: 0, blue: 0 }));
    }
}
