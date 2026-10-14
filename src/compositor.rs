//! Reusable, pure lighting-compositor primitives for Prism-style graphics.
//!
//! Layer order, opacity, blend modes, transforms and masks are independent
//! models; this is NOT SteelSeries' proprietary compositor or shader.
//! No HID, evdev, timers, I/O or process-wide state.
use crate::Rgb;
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}

#[derive(Clone, Copy, Debug)]
pub enum Geometry {
    Horizontal,
    Vertical,
    Radial,
    Diagonal,
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    /// Reference coordinate for phase origin, in normalized board units.
    pub origin: [f32; 2],
    /// Separate axis scales for stretched and compressed graphics.
    pub scale: [f32; 2],
    /// Positive angles rotate the graphic clockwise in screen coordinates.
    pub rotation_radians: f32,
    /// Repeat the spatial field by this factor without repeating time.
    pub repeats: f32,
    /// A full cycle is 1.0. This offset shifts the animation in space/time.
    pub offset: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        origin: [0.0, 0.0],
        scale: [1.0, 1.0],
        rotation_radians: 0.0,
        repeats: 1.0,
        offset: 0.0,
    };
    pub const CENTERED: Self = Self {
        origin: [0.5, 0.5],
        scale: [1.0, 1.0],
        rotation_radians: 0.0,
        repeats: 1.0,
        offset: 0.0,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub period_ms: u32,
    pub wavelength: i16,
    pub geometry: Geometry,
    pub transform: Transform,
}

#[derive(Clone, Copy, Debug)]
pub enum ApertureShape {
    /// Smooth periodic coverage, a local replacement for an unavailable shader.
    Cosine,
    /// Flat-topped moving band with a linearly feathered edge.
    Band { half_width: f32, feather: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Aperture {
    pub motion: Motion,
    pub shape: ApertureShape,
    pub invert: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Graphic {
    pub stops: &'static [(u8, u32)],
    pub motion: Motion,
    pub opacity: f32,
    pub blend: BlendMode,
    pub aperture: Option<Aperture>,
}

/// Layers are evaluated strictly in array order: first is back, last is front.
/// For safety, every value is clipped to valid 8-bit RGB output. Scene can
/// borrow compile-time arrays; no allocations occur per key frame.
#[derive(Clone, Copy, Debug)]
pub struct Scene<'a> {
    pub clear: Rgb,
    pub layers: &'a [Graphic],
}

pub const BLACK: Rgb = Rgb {
    red: 0,
    green: 0,
    blue: 0,
};

fn from_hex(value: u32) -> Rgb {
    Rgb {
        red: (value >> 16) as u8,
        green: (value >> 8) as u8,
        blue: value as u8,
    }
}

/// Straight sRGB channel interpolation. Source GG gradient color space is
/// not yet verified; this explicit behavior is deterministic and testable.
fn interpolate(a: Rgb, b: Rgb, factor: f32) -> Rgb {
    let t = factor.clamp(0.0, 1.0);
    let blend = |x: u8, y: u8| (f32::from(x) * (1.0 - t) + f32::from(y) * t).round() as u8;
    Rgb {
        red: blend(a.red, b.red),
        green: blend(a.green, b.green),
        blue: blend(a.blue, b.blue),
    }
}

/// Sorted source-color stops, encoded as U8 positions 0..255.
/// Duplicate/empty stops have deterministic fallback semantics.
pub fn palette(stops: &[(u8, u32)], phase: f32) -> Rgb {
    let Some(&(first, color)) = stops.first() else {
        return BLACK;
    };
    let normalized = if phase.is_finite() { phase.clamp(0.0, 255.0) } else { 0.0 };
    if normalized <= f32::from(first) {
        return from_hex(color);
    }
    for window in stops.windows(2) {
        let (start, first_color) = window[0];
        let (end, second_color) = window[1];
        if normalized <= f32::from(end) {
            if end <= start {
                return from_hex(second_color);
            }
            let factor = (normalized - f32::from(start)) / f32::from(end - start);
            return interpolate(from_hex(first_color), from_hex(second_color), factor);
        }
    }
    from_hex(stops[stops.len() - 1].1)
}

/// Normalized cyclic motion phase. Wavelength signs preserve travel
/// direction; the values do not assert vendor shader/spatial equivalence.
pub fn phase(motion: Motion, x: f32, y: f32, time: f32, speed: f32) -> f32 {
    let tr = motion.transform;
    let u = (x - tr.origin[0]) * tr.scale[0];
    let v = (y - tr.origin[1]) * tr.scale[1];
    let (sine, cosine) = tr.rotation_radians.sin_cos();
    let rx = u * cosine - v * sine;
    let ry = u * sine + v * cosine;
    let axis = match motion.geometry {
        Geometry::Horizontal => rx + ry * 0.12,
        Geometry::Vertical => ry,
        Geometry::Radial => (rx * rx + ry * ry).sqrt() * 1.5,
        Geometry::Diagonal => rx * 0.55 + ry * 0.45,
    };
    let spatial = if motion.wavelength == 0 {
        0.0
    } else {
        let ratio = (64.0 / f32::from(motion.wavelength.unsigned_abs())).clamp(0.1, 20.0);
        ratio * axis * tr.repeats * motion.wavelength.signum() as f32
    };
    let clock = if motion.period_ms == 0 {
        0.0
    } else {
        time.max(0.0) * speed.max(0.0) * 1000.0 / motion.period_ms as f32
    };
    (spatial + clock + tr.offset).rem_euclid(1.0)
}

fn mask_coverage(aperture: Aperture, x: f32, y: f32, time: f32, speed: f32) -> f32 {
    let progress = phase(aperture.motion, x, y, time, speed);
    let coverage = match aperture.shape {
        ApertureShape::Cosine => 0.5 + 0.5 * (progress * TAU).cos(),
        ApertureShape::Band { half_width, feather } => {
            let distance = progress.min(1.0 - progress);
            let edge = (half_width - distance) / feather.max(0.0001);
            (0.5 + edge * 0.5).clamp(0.0, 1.0)
        }
    };
    if aperture.invert { 1.0 - coverage } else { coverage }
}

pub fn blend_over(background: Rgb, foreground: Rgb, opacity: f32, mode: BlendMode) -> Rgb {
    let calc = |back: u8, front: u8| {
        match mode {
            BlendMode::Normal => front,
            BlendMode::Add => back.saturating_add(front),
            BlendMode::Multiply => ((u16::from(back) * u16::from(front) + 127) / 255) as u8,
            BlendMode::Screen => {
                (255 - ((u16::from(255 - back) * u16::from(255 - front) + 127) / 255)) as u8
            }
        }
    };
    let target = Rgb {
        red: calc(background.red, foreground.red),
        green: calc(background.green, foreground.green),
        blue: calc(background.blue, foreground.blue),
    };
    interpolate(background, target, opacity)
}

impl Scene<'_> {
    pub fn sample(&self, x: f32, y: f32, time: f32, speed: f32) -> Rgb {
        self.layers.iter().fold(self.clear, |background, graphic| {
            let t = phase(graphic.motion, x, y, time, speed);
            let foreground = palette(graphic.stops, t * 255.0);
            let alpha = graphic.opacity.clamp(0.0, 1.0)
                * graphic.aperture.map_or(1.0, |mask| mask_coverage(mask, x, y, time, speed));
            blend_over(background, foreground, alpha, graphic.blend)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb { red: 255, green: 0, blue: 0 };
    const BLUE: Rgb = Rgb { red: 0, green: 0, blue: 255 };
    const UNIFORM: Motion = Motion {
        period_ms: 0,
        wavelength: 0,
        geometry: Geometry::Horizontal,
        transform: Transform::IDENTITY,
    };
    const RED_LAYER: Graphic = Graphic {
        stops: &[(0, 0xff0000)],
        motion: UNIFORM,
        opacity: 1.0,
        blend: BlendMode::Normal,
        aperture: None,
    };
    const BLUE_LAYER: Graphic = Graphic {
        stops: &[(0, 0x0000ff)],
        motion: UNIFORM,
        opacity: 0.5,
        blend: BlendMode::Normal,
        aperture: None,
    };

    #[test]
    fn layer_order_and_opacity_are_explicit() {
        let back_first = Scene { clear: BLACK, layers: &[RED_LAYER, BLUE_LAYER] };
        let front_first = Scene { clear: BLACK, layers: &[BLUE_LAYER, RED_LAYER] };
        assert_eq!(back_first.sample(0.3, 0.4, 0.0, 1.0), Rgb { red: 128, green: 0, blue: 128 });
        assert_eq!(front_first.sample(0.3, 0.4, 0.0, 1.0), RED);
    }
    #[test]
    fn blending_endpoints_and_modes_are_bounded() {
        let dark = Rgb { red: 40, green: 90, blue: 250 };
        let light = Rgb { red: 220, green: 200, blue: 80 };
        for mode in [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen] {
            assert_eq!(blend_over(dark, light, 0.0, mode), dark);
            let end = blend_over(dark, light, 1.0, mode);
            assert_eq!(end, blend_over(dark, light, 1.0, mode));
            assert_ne!(blend_over(dark, light, 0.5, mode), dark);
        }
        assert_eq!(blend_over(BLUE, RED, 1.0, BlendMode::Add), Rgb { red: 255, green: 0, blue: 255 });
        assert_eq!(blend_over(BLUE, RED, 1.0, BlendMode::Multiply), BLACK);
        assert_eq!(blend_over(BLUE, RED, 1.0, BlendMode::Screen), Rgb { red: 255, green: 0, blue: 255 });
    }
    #[test]
    fn transforms_affect_position_not_clock() {
        let map = Motion {
            period_ms: 1000, wavelength: 64,
            geometry: Geometry::Horizontal, transform: Transform::IDENTITY,
        };
        let origin_shifted = Motion {
            transform: Transform { origin: [0.2, 0.0], ..Transform::IDENTITY },
            ..map
        };
        let rotated = Motion {
            transform: Transform { rotation_radians: std::f32::consts::FRAC_PI_2, ..Transform::IDENTITY },
            ..map
        };
        assert_ne!(phase(map, 0.25, 0.0, 0.0, 1.0), phase(origin_shifted, 0.25, 0.0, 0.0, 1.0));
        assert_ne!(phase(map, 0.25, 0.0, 0.0, 1.0), phase(rotated, 0.25, 0.0, 0.0, 1.0));
        assert!((phase(map, 0.0, 0.0, 1.0, 1.0) - phase(map, 0.0, 0.0, 0.0, 1.0)).abs() < 1e-6);
    }
    #[test]
    fn positive_and_negative_waves_travel_in_opposite_directions() {
        let forward = Motion { period_ms: 1000, wavelength: 64,
            geometry: Geometry::Horizontal, transform: Transform::IDENTITY };
        let backward = Motion { wavelength: -64, ..forward };
        let f = phase(forward, 0.25, 0.0, 0.1, 1.0);
        let b = phase(backward, 0.25, 0.0, 0.1, 1.0);
        assert_ne!(f, b);
    }
    #[test]
    fn repeating_aperture_changes_coverage_and_is_bounded() {
        let aperture = Aperture {
            motion: Motion {
                period_ms: 1000, wavelength: 64,
                geometry: Geometry::Horizontal, transform: Transform::IDENTITY,
            },
            shape: ApertureShape::Cosine, invert: false,
        };
        let repeated = Aperture {
            motion: Motion {
                transform: Transform { repeats: 2.0, ..Transform::IDENTITY },
                ..aperture.motion
            },
            ..aperture
        };
        let base = mask_coverage(aperture, 0.27, 0.0, 0.0, 1.0);
        let rep = mask_coverage(repeated, 0.27, 0.0, 0.0, 1.0);
        assert!((0.0..=1.0).contains(&base));
        assert!((0.0..=1.0).contains(&rep));
        assert_ne!(base, rep);
        let inverse = mask_coverage(Aperture { invert: true, ..aperture }, 0.27, 0.0, 0.0, 1.0);
        assert!((base + inverse - 1.0).abs() < 1e-6);
    }
    #[test]
    fn palettes_clamp_to_source_color_endpoints() {
        assert_eq!(palette(&[], 124.0), BLACK);
        assert_eq!(palette(&[(20, 0xff0000),(200, 0x0000ff)], 0.0), RED);
        assert_eq!(palette(&[(20, 0xff0000),(200, 0x0000ff)], 255.0), BLUE);
        assert_eq!(palette(&[(0, 0xff0000),(255, 0x0000ff)], 127.5), Rgb { red: 128, green: 0, blue: 128 });
        assert_eq!(palette(&[(0, 0xff0000)], f32::NAN), RED);
    }
    #[test]
    fn masked_scene_does_not_modify_unrelated_layers() {
        let aperture = Aperture {
            motion: UNIFORM,
            shape: ApertureShape::Band { half_width: 0.0, feather: 0.01 },
            invert: false,
        };
        let masked = Graphic {
            aperture: Some(aperture), ..BLUE_LAYER
        };
        let scene = Scene { clear: BLACK, layers: &[RED_LAYER, masked] };
        assert_eq!(scene.sample(0.1, 0.2, 0.0, 1.0).green, 0);
        assert!(scene.sample(0.1, 0.2, 0.0, 1.0).red > 0);
    }
}
