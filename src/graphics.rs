//! Historical Prism multi-graphic scene definitions.
//!
//! Functional gradient/period/wavelength parameters are summarized from
//! the owner's GG 108.3 preset migrations. All rendering is performed by
//! the independently designed compositor.rs. The original vendor aperture
//! shader and opacity/mask semantics are NOT available or reproduced.
use crate::compositor::{
    Aperture, ApertureShape, BlendMode, Geometry, Graphic, Motion, Scene, Transform, BLACK,
};
use crate::effects::Config;
use crate::Rgb;

const fn radial(period_ms: u32, wave: i16) -> Motion {
    Motion {
        period_ms,
        wavelength: wave,
        geometry: Geometry::Radial,
        transform: Transform::CENTERED,
    }
}
const fn horizontal(period_ms: u32, wave: i16) -> Motion {
    Motion {
        period_ms,
        wavelength: wave,
        geometry: Geometry::Horizontal,
        transform: Transform::IDENTITY,
    }
}

/// Original GG source aperture fields have no RGB stops, so an independent
/// smooth coverage function is used. This is expressly NOT an exact GG mask.
const fn aperture(motion: Motion) -> Option<Aperture> {
    Some(Aperture {
        motion,
        shape: ApertureShape::Cosine,
        invert: false,
    })
}

/// West Coast: two distinct static source gradients plus a radial mask.
const WEST_COAST: Scene<'static> = Scene {
    clear: BLACK,
    layers: &[
        Graphic {
            stops: &[(0, 0x3250ff), (230, 0x00baff), (245, 0xff2800), (255, 0xff2800)],
            motion: radial(0, -24),
            opacity: 1.0,
            blend: BlendMode::Normal,
            aperture: None,
        },
        Graphic {
            stops: &[(0, 0xff2000), (230, 0xffaf00), (245, 0x05b1ff), (255, 0x05b1ff)],
            motion: radial(0, -24),
            opacity: 1.0,
            blend: BlendMode::Normal,
            aperture: aperture(radial(0, -24)),
        },
    ],
};

/// Haze: a time-varying rainbow-like background, a single magenta foreground,
/// and a separate slow spatial aperture.
const HAZE: Scene<'static> = Scene {
    clear: BLACK,
    layers: &[
        Graphic {
            stops: &[(0, 0xff0000), (85, 0x0000ff), (170, 0x00ffff), (255, 0xff0000)],
            motion: horizontal(4000, 24),
            opacity: 1.0,
            blend: BlendMode::Normal,
            aperture: None,
        },
        Graphic {
            stops: &[(0, 0xff00ff)],
            motion: horizontal(5000, 0),
            opacity: 1.0,
            blend: BlendMode::Normal,
            aperture: aperture(radial(9000, 96)),
        },
    ],
};

/// Warp Drive: radial foreground with an aperture against dark background.
const WARP_DRIVE: Scene<'static> = Scene {
    clear: BLACK,
    layers: &[Graphic {
        stops: &[(0, 0x000000), (38, 0xffffff), (77, 0xffffff), (255, 0x000000)],
        motion: radial(1200, 6),
        opacity: 1.0,
        blend: BlendMode::Normal,
        aperture: aperture(radial(1200, 6)),
    }],
};

/// Specialized historic graphics scenes, with the rest falling through to
/// the factory palette pipeline. No USB or physical keypress access.
pub fn special_scene(
    id: &str,
    x: f32,
    y: f32,
    seconds: f32,
    config: Config,
) -> Option<Rgb> {
    let scene = match id {
        "westCoast" => WEST_COAST,
        "haze" => HAZE,
        "warpDrive" => WARP_DRIVE,
        _ => return None,
    };
    Some(scene.sample(x, y, seconds, config.speed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effects, encode_direct_write};

    #[test]
    fn all_three_compound_presets_remain_full_68_key_frames() {
        let settings = Config { brightness: 1.0, ..Config::default() };
        for name in ["westCoast", "haze", "warpDrive"] {
            let mode = effects::Preset::from_name(&format!("gg:{name}")).unwrap();
            let frame = effects::frame(mode, settings, 0.125, &[]);
            assert_eq!(frame.len(), 68);
            assert_eq!(encode_direct_write(&frame).unwrap().len(), 513);
            assert_eq!(encode_direct_write(&frame).unwrap()[2], 68);
            assert!(frame.iter().any(|key| key.rgb != frame[0].rgb));
        }
    }
    #[test]
    fn haze_foreground_background_and_aperture_are_independent_in_time() {
        let config = Config { brightness: 1.0, ..Config::default() };
        let a = special_scene("haze", 0.4, 0.3, 0.1, config).unwrap();
        let b = special_scene("haze", 0.4, 0.3, 1.4, config).unwrap();
        assert_ne!(a, b);
    }
    #[test]
    fn west_coast_has_two_ordered_graphics_layers() {
        assert_eq!(WEST_COAST.layers.len(), 2);
        assert!(WEST_COAST.layers[0].aperture.is_none());
        assert!(WEST_COAST.layers[1].aperture.is_some());
        assert_eq!(HAZE.layers.len(), 2);
        assert_eq!(WARP_DRIVE.layers.len(), 1);
    }
    #[test]
    fn unknown_preset_uses_existing_factory_fallback() {
        assert!(special_scene("aqua", 0.0, 0.0, 0.0, Config::default()).is_none());
    }
}
