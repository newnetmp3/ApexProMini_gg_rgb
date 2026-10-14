//! Independently implemented factory-style Prism effect interpreter.
//!
//! The compact parameters below are *derived facts* (public-facing preset
//! names, RGB stop colors, period and motion class) independently summarized
//! from the owner's GG 108.3 Prism migration resources. No vendor source JSON,
//! SQL or encrypted application files are included.
//!
//! IMPORTANT: This interprets one representative variant per preset with a
//! NEW approximation of the GG motion model. Proprietary gradient compositor,
//! multiple variants and physical masks are NOT yet reproduced exactly.
//! "Available" means renderable, NOT parity-certified or hardware-verified.
use crate::effects::{Config, KeyHit};
use crate::Rgb;

#[derive(Clone, Copy, Debug)]
pub struct FactoryPreset {
    pub id: &'static str,
    pub cli: &'static str,
    pub period_ms: u32,
    pub wavelength: i16,
    pub geometry: u8,
    pub stops: &'static [(u8, u32)],
    pub variants_in_source: u8,
}

macro_rules! preset {
    ($id:literal, $duration:expr, $wave:expr, $shape:expr, $count:expr, [$($pos:literal => $rgb:literal),* $(,)?]) => {
        FactoryPreset {
            id: $id,
            cli: concat!("gg:", $id),
            period_ms: $duration,
            wavelength: $wave,
            geometry: $shape,
            variants_in_source: $count,
            stops: &[$(($pos, $rgb)),*],
        }
    };
}

/// GG 108.3 migration locale keys; 30 complete historical records and the
/// later "redPulse" rename of "faze" (same underlying preset identifier).
///
/// Colors/durations alone are functional configuration parameters, not the
/// full vendor graphics/trigger engine. Only the representative first region
/// is parameterized for records with multiple source variants.
pub static PRESETS: &[FactoryPreset] = &[
    preset!("steelseriesOrange", 0, 0, 0, 1, []),
    preset!("vaporDreams", 30000, -8, 3, 1, [0=>0xff0040,64=>0xff00ff,128=>0x0000ff,192=>0x00ffa0,255=>0xff0040]),
    preset!("westCoast", 0, -24, 1, 1, [0=>0xff2000,230=>0xffaf00,245=>0x05b1ff,255=>0x05b1ff]),
    preset!("haze", 5000, 0, 0, 1, [0=>0xff00ff]),
    preset!("prism", 7500, 32, 0, 1, [0=>0xff0000,42=>0xffff00,85=>0x00ff00,127=>0x00ffff,170=>0x0000ff,212=>0xff00ff,255=>0xff0000]),
    preset!("reactiveLine", 500, 16, 2, 1, [0=>0xffffff]),
    preset!("reactiveRipple", 250, 3, 1, 1, [0=>0xffffff]),
    preset!("reactiveSingleKey", 1250, 0, 2, 1, [0=>0xffffff]),
    preset!("chasingGhosts", 9500, 48, 0, 1, [0=>0x000000,13=>0xffd500,51=>0xffd500,64=>0x000000,107=>0x000000,128=>0xff8000,143=>0x000000,158=>0x00eeff,173=>0x000000,189=>0xff0000,204=>0x000000,219=>0xff66d4,255=>0x000000]),
    preset!("warpDrive", 1200, 6, 1, 1, [0=>0x000000,38=>0xffffff,77=>0xffffff,255=>0x000000]),
    preset!("wabashAndLake", 2100, -25, 3, 4, [0=>0x000000,26=>0x50a000,39=>0x081000,77=>0x285000,102=>0x6edc00,153=>0x408000,204=>0x64c800,220=>0x80ff00]),
    preset!("solar", 31679, 64, 3, 1, [23=>0x01003d,52=>0xffea00,93=>0x00ccff,177=>0x00ffff,188=>0xe1ff00,215=>0xff0400,237=>0x66005a]),
    preset!("rainbow", 6120, 89, 0, 7, [0=>0xff0400,46=>0xf7ff00,85=>0x04ff00,125=>0x00ffff,170=>0x1100ff,217=>0xb700ff]),
    preset!("staticFade", 0, 12, 1, 1, [0=>0x82008f,85=>0xea0c5f,170=>0xff8820]),
    preset!("discoMode", 750, 2, 1, 1, [0=>0xff0000,36=>0x00ff00,73=>0xff3200,109=>0x00ffff,146=>0xff00ff,182=>0x0000ff,219=>0xff0000]),
    preset!("colorFusion", 5038, 64, 3, 3, [24=>0xffea00,118=>0xd500ff,229=>0xff0000]),
    preset!("shavedIce", 14071, -51, 3, 1, [0=>0xff0400,20=>0xff0400,85=>0xffffff,90=>0x001eff,114=>0x001eff,168=>0xffffff,173=>0xffed1f,197=>0xfff705,249=>0xffffff]),
    preset!("selfDestruct", 1679, 64, 0, 1, [1=>0x0f0a00,75=>0xff6900,125=>0xff3700,205=>0x0f0a00]),
    preset!("clown", 1679, 64, 1, 1, [1=>0x8a26d9,25=>0x0085ff,48=>0x7fff00,70=>0xffea00,90=>0xff1900,112=>0xffffff,135=>0xff1900,155=>0xffea00,178=>0x7fff00,200=>0x0085ff,220=>0x8a26d9]),
    preset!("radioactive", 8000, 51, 1, 1, [1=>0x183d00,42=>0x80ff00,61=>0x0dff00,85=>0x80ff00,125=>0x183d00,173=>0xa2ff00,196=>0xccff00,218=>0x88ff00,252=>0x183d00]),
    preset!("faze", 3185, 0, 0, 1, [0=>0xff0000,127=>0x000000]),
    preset!("aqua", 4000, 64, 1, 6, [0=>0x009dff,63=>0x0026ff,112=>0x000000,160=>0x00bdff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),
    preset!("rainbowSplit", 2400, 24, 0, 1, [100=>0x000000,110=>0xff0000,120=>0xff7f00,130=>0xffff00,140=>0x00ff00,150=>0x0000ff,160=>0x4b0082,170=>0x9400ff,180=>0x000000]),
    preset!("drain", 2774, -36, 1, 1, [60=>0x000000,80=>0xff0000,100=>0xffff00,120=>0x00ff00,140=>0x00ffff,160=>0x0000ff,180=>0xff00ff,200=>0x000000]),
    preset!("freeway", 1679, 64, 0, 7, [80=>0x000000,125=>0x00ff78,170=>0x000000]),
    preset!("disabled", 0, 0, 0, 1, []),
    preset!("comet", 1679, 64, 0, 6, [68=>0x000000,94=>0xffffff,120=>0x00eaff,170=>0x000000]),
    preset!("pride", 4690, 64, 0, 7, [0=>0xff0000,81=>0xff6600,171=>0xff0000]),
    preset!("twilight", 4690, 64, 0, 4, [0=>0xd60071,78=>0x9b59b6,174=>0xd60071]),
    preset!("heatmap", 0, 0, 0, 1, []),
    // GG 106 migration renames the existing Faze preset to Red Pulse.
    // Preserve both historical identifiers; this is NOT a 31st distinct
    // graphics definition in the 108.3 package.
    preset!("redPulse", 3185, 0, 0, 1, [0=>0xff0000,127=>0x000000]),
];

pub fn find(name: &str) -> Option<u8> {
    let key = name.strip_prefix("gg:")?;
    PRESETS.iter().position(|preset| preset.id.eq_ignore_ascii_case(key))
        .and_then(|index| u8::try_from(index).ok())
}

pub fn by_index(index: u8) -> Option<&'static FactoryPreset> {
    PRESETS.get(index as usize)
}

pub fn is_reactive(index: u8) -> bool {
    let Some(spec) = by_index(index) else { return false; };
    matches!(spec.id, "reactiveLine" | "reactiveRipple" | "reactiveSingleKey" | "heatmap")
}

fn from_hex(hex: u32) -> Rgb {
    Rgb {
        red: (hex >> 16) as u8,
        green: (hex >> 8) as u8,
        blue: hex as u8,
    }
}

fn mix(a: Rgb, b: Rgb, amount: f32) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    let f = |a: u8, b: u8| {
        (a as f32 * (1.0 - amount) + b as f32 * amount).round() as u8
    };
    Rgb {
        red: f(a.red, b.red),
        green: f(a.green, b.green),
        blue: f(a.blue, b.blue),
    }
}

fn sample(stops: &[(u8, u32)], phase: f32) -> Rgb {
    let Some(&(first, first_color)) = stops.first() else {
        return Rgb { red: 0, green: 0, blue: 0 };
    };
    if phase <= first as f32 { return from_hex(first_color); }
    for window in stops.windows(2) {
        let (from, col_a) = window[0];
        let (to, col_b) = window[1];
        if phase <= to as f32 {
            let width = u8::saturating_sub(to, from);
            if width == 0 { return from_hex(col_b); }
            let f = (phase - from as f32) / f32::from(width);
            return mix(from_hex(col_a), from_hex(col_b), f);
        }
    }
    from_hex(stops.last().expect("nonempty stops").1)
}

/// Pure approximation of GG phase classes. Geometry categories and color
/// parameters are derived from vendor configuration, but the compositor,
/// coordinate scale, variant masks, apertures and blending are original and
/// require visual QA before any claim of strict SteelSeries preset parity.
pub fn render(index: u8, hid: u8, x: f32, y: f32, seconds: f32, config: Config, hits: &[KeyHit]) -> Rgb {
    let Some(spec) = by_index(index) else { return Rgb { red: 0, green: 0, blue: 0 }; };
    if spec.id == "disabled" { return from_hex(0); }
    if spec.id == "steelseriesOrange" { return from_hex(0xff3200); }

    if spec.id == "heatmap" {
        let mut heat = 0.0_f32;
        for hit in hits {
            let age = seconds - hit.when;
            if !(0.0..=2.5).contains(&age) { continue; }
            if let Some((hx, hy)) = crate::effects::position(hit.hid) {
                let dist = ((x - hx).powi(2) + (y - hy).powi(2)).sqrt();
                heat += (-dist * 20.0).exp() * (1.0 - age / 2.5);
            }
        }
        return mix(from_hex(0x070070), from_hex(0xff3c00), heat.min(1.0));
    }
    if is_reactive(index) {
        let duration = (spec.period_ms as f32 / 1000.0).max(0.05) / config.speed.max(0.1);
        let mut strength = 0.0_f32;
        for hit in hits {
            let age = seconds - hit.when;
            if !(0.0..=5.0).contains(&age) || age > duration { continue; }
            let Some((hx, hy)) = crate::effects::position(hit.hid) else { continue; };
            let distance = ((x - hx).powi(2) + (y - hy).powi(2)).sqrt();
            let u = age / duration;
            let pulse = match spec.id {
                "reactiveSingleKey" => {
                    if hid == hit.hid { 1.0 - u } else { 0.0 }
                }
                "reactiveLine" => {
                    let band = (y - hy).abs();
                    let ring = (x - hx).abs() - u * 1.2;
                    (-(band / 0.13).powi(2)).exp() * (-(ring / 0.14).powi(2)).exp() * (1.0 - u)
                }
                _ => {
                    let ring = distance - u * 1.4;
                    (-(ring / 0.14).powi(2)).exp() * (1.0 - u)
                }
            };
            strength = strength.max(pulse);
        }
        return mix(config.base, sample(spec.stops, 0.0), strength);
    }

    let variant = crate::regions::select(spec.id, x, y);
    let geometry = variant.map(|v| v.geometry).unwrap_or(spec.geometry);
    let wavelength = variant.map(|v| v.wavelength).unwrap_or(spec.wavelength);
    let period_ms = variant.map(|v| v.period_ms).unwrap_or(spec.period_ms);
    let stops = variant.map(|v| v.stops).unwrap_or(spec.stops);
    let spatial = match geometry {
        1 => ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt() * 1.5,
        2 => y,
        3 => x * 0.55 + y * 0.45,
        _ => x + y * 0.12,
    };
    let ratio = if wavelength == 0 {
        0.0
    } else {
        (64.0 / f32::from(wavelength.unsigned_abs())).clamp(0.1, 20.0)
    };
    let direction = if wavelength < 0 { -1.0 } else { 1.0 };
    let clock = if period_ms == 0 {
        0.0
    } else {
        seconds.max(0.0) * config.speed * 1000.0 / period_ms as f32
    };
    let phase = (spatial * ratio * direction + clock).rem_euclid(1.0) * 255.0;
    let rgb = sample(stops, phase);
    if spec.id == "haze" {
        // Single-color graphics have additional aperture/background animation
        // in GG; this independent breath approximation is not a parity match.
        let fade = 0.4 + 0.6 * (0.5 + 0.5 * (clock * std::f32::consts::TAU).sin());
        return mix(from_hex(0), rgb, fade);
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effects, encode_direct_write};
    use std::collections::HashSet;

    #[test]
    fn all_31_historical_identifiers_have_unique_selectors() {
        assert_eq!(PRESETS.len(), 31);
        let mut seen = HashSet::new();
        for (i, spec) in PRESETS.iter().enumerate() {
            assert!(seen.insert(spec.id));
            assert_eq!(find(spec.cli), Some(i as u8));
            assert_eq!(by_index(i as u8).unwrap().id, spec.id);
            assert!(spec.variants_in_source >= 1);
        }
        assert_eq!(find("gg:rainbow"), Some(12));
        assert!(find("some-other-name").is_none());
        assert!(find("gg:missing").is_none());
    }

    #[test]
    fn factory_presets_encode_full_mini_feature_frames_offline() {
        for (idx, preset) in PRESETS.iter().enumerate() {
            let mode = effects::Preset::Factory(idx as u8);
            assert_eq!(mode.as_str(), preset.cli);
            assert_eq!(effects::Preset::from_name(preset.cli), Some(mode));
            for time in [0.0, 0.02, 0.2, 3.7] {
                let colors = effects::frame(mode, Config::default(), time, &[]);
                assert_eq!(colors.len(), 68);
                assert_eq!(encode_direct_write(&colors).unwrap()[2], 68);
            }
        }
    }

    #[test]
    fn original_factory_reactive_presets_respond_to_hits() {
        let config = Config { brightness: 1.0, ..Config::default() };
        for key in ["gg:reactiveLine", "gg:reactiveRipple", "gg:reactiveSingleKey", "gg:heatmap"] {
            let mode = effects::Preset::from_name(key).unwrap();
            let without = effects::frame(mode, config, 0.06, &[]);
            let with = effects::frame(mode, config, 0.06, &[KeyHit { hid: 4, when: 0.0 }]);
            assert_ne!(without.iter().map(|k| (k.rgb.red, k.rgb.green, k.rgb.blue)).collect::<Vec<_>>(),
                       with.iter().map(|k| (k.rgb.red, k.rgb.green, k.rgb.blue)).collect::<Vec<_>>(), "{key}");
        }
    }
    #[test]
    fn later_red_pulse_is_renamed_faze() {
        let first = find("gg:faze").unwrap();
        let later = find("gg:redPulse").unwrap();
        assert_eq!(PRESETS[first as usize].period_ms, PRESETS[later as usize].period_ms);
        assert_eq!(PRESETS[first as usize].stops, PRESETS[later as usize].stops);
    }
    #[test]
    fn source_derived_regional_palettes_affect_keyboard_rows() {
        let config = Config { brightness: 1.0, ..Config::default() };
        let idx = find("gg:pride").unwrap();
        let top = render(idx, 30, 0.24, 0.0, 0.0, config, &[]);
        let bottom = render(idx, 44, 0.24, 1.0, 0.0, config, &[]);
        assert_ne!(top, bottom);
    }
    #[test]
    fn palette_samples_are_bounded_at_edges_and_repeated_stops() {
        assert_eq!(sample(&[(0, 0xff0000), (255, 0x00ff00)], 0.0), from_hex(0xff0000));
        assert_eq!(sample(&[(0, 0xff0000), (255, 0x00ff00)], 255.0), from_hex(0x00ff00));
        assert_eq!(sample(&[(0, 0xff0000), (0, 0x00ff00)], 0.0), from_hex(0xff0000));
    }
}
