//! Small, independent graphic-layer compositor for GG-style foreground,
//! background and aperture recipes. No vendor code/JSON is embedded.
//!
//! Parameters come from the owner's authorized GG Prism migrations; the
//! phase/masking and compositing math is our own approximate implementation.
//! Exact Prism GPU shader equivalence still requires external visual QA.
use crate::effects::Config;
use crate::Rgb;

#[derive(Debug, Clone, Copy)]
struct Motion {
    period_ms: u32,
    wave: i16,
    geometry: u8,
    stops: &'static [(u8,u32)],
}
#[derive(Debug, Clone, Copy)]
struct Scene {
    foreground: Motion,
    background: Option<Motion>,
    aperture: Option<Motion>,
}

fn unpack(rgb: u32) -> Rgb {
    Rgb { red: (rgb >> 16) as u8, green: (rgb >> 8) as u8, blue: rgb as u8 }
}
fn mix(a: Rgb, b: Rgb, alpha: f32) -> Rgb {
    let w = alpha.clamp(0.0, 1.0);
    let f = |x:u8,y:u8| (x as f32 * (1.0-w) + y as f32 * w).round() as u8;
    Rgb { red:f(a.red,b.red), green:f(a.green,b.green), blue:f(a.blue,b.blue) }
}
fn palette(stops: &[(u8,u32)], position: f32) -> Rgb {
    let Some(&(initial, color)) = stops.first() else {
        return unpack(0);
    };
    if position <= initial as f32 { return unpack(color); }
    for slice in stops.windows(2) {
        let (a,from) = slice[0];
        let (b,to) = slice[1];
        if position <= b as f32 {
            let width = (b-a).max(1) as f32;
            return mix(unpack(from), unpack(to), (position-a as f32)/width);
        }
    }
    unpack(stops.last().unwrap().1)
}
fn phase(motion: Motion, x: f32, y: f32, seconds: f32, speed: f32) -> f32 {
    let radial = ((x-0.5).powi(2) + (y-0.5).powi(2)).sqrt();
    let spatial = match motion.geometry {
        1 => radial * 1.5,
        2 => y,
        3 => x*0.55 + y*0.45,
        _ => x + y*0.12,
    };
    let space = if motion.wave == 0 { 0.0 }
        else {
            let sign = if motion.wave < 0 { -1.0 } else { 1.0 };
            sign * (64.0/f32::from(motion.wave.unsigned_abs())).clamp(0.1,20.0)*spatial
        };
    let clock = if motion.period_ms == 0 {0.0}
        else { seconds.max(0.0)*speed*1000.0/motion.period_ms as f32 };
    (space + clock).rem_euclid(1.0)
}
fn render(scene: Scene, x: f32, y: f32, t: f32, config: Config) -> Rgb {
    let fore_phase = phase(scene.foreground,x,y,t,config.speed);
    let foreground = palette(scene.foreground.stops,fore_phase*255.0);
    let Some(aperture) = scene.aperture else {
        return foreground;
    };
    // Original GG aperture data has *no gradient stops*. The independent
    // sinusoidal coverage function below approximates a smooth spatial gate
    // and must NOT be represented as the official aperture implementation.
    let aperture_phase = phase(aperture,x,y,t,config.speed);
    let coverage = 0.5 + 0.5*(aperture_phase*std::f32::consts::TAU).cos();
    let background = scene.background
        .map(|back| palette(back.stops,phase(back,x,y,t,config.speed)*255.0))
        .unwrap_or(unpack(0));
    mix(background,foreground,coverage)
}

const WEST_COAST: Scene = Scene {
    foreground: Motion {
        period_ms:0,wave:-24,geometry:1,
        stops:&[(0,0xff2000),(230,0xffaf00),(245,0x05b1ff),(255,0x05b1ff)],
    },
    background:Some(Motion {
        period_ms:0,wave:-24,geometry:1,
        stops:&[(0,0x3250ff),(230,0x00baff),(245,0xff2800),(255,0xff2800)],
    }),
    aperture:Some(Motion {period_ms:0,wave:-24,geometry:1,stops:&[]}),
};
const HAZE: Scene = Scene {
    foreground: Motion {
        period_ms:5000,wave:0,geometry:0,stops:&[(0,0xff00ff)],
    },
    background:Some(Motion {
        period_ms:4000,wave:24,geometry:0,
        stops:&[(0,0xff0000),(85,0x0000ff),(170,0x00ffff),(255,0xff0000)],
    }),
    aperture:Some(Motion {period_ms:9000,wave:96,geometry:1,stops:&[]}),
};
const WARP_DRIVE: Scene = Scene {
    foreground: Motion {
        period_ms:1200,wave:6,geometry:1,
        stops:&[(0,0x000000),(38,0xffffff),(77,0xffffff),(255,0x000000)],
    },
    background:None,
    aperture:Some(Motion {period_ms:1200,wave:6,geometry:1,stops:&[]}),
};

/// Try a specialized multi-graphic scene; return None for presets rendered
/// by the factory gradient pipeline.
pub fn special_scene(id: &str, x: f32, y: f32, seconds: f32, config: Config) -> Option<Rgb> {
    let scene = match id {
        "westCoast" => WEST_COAST,
        "haze" => HAZE,
        "warpDrive" => WARP_DRIVE,
        _ => return None,
    };
    Some(render(scene,x,y,seconds,config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effects,encode_direct_write};
    #[test]
    fn source_multigraphic_scenes_are_spatially_distinct() {
        let config=Config {brightness:1.0,..Config::default()};
        for name in ["westCoast","haze","warpDrive"] {
            let mode=effects::Preset::from_name(&format!("gg:{name}")).unwrap();
            let frame=effects::frame(mode,config,0.125,&[]);
            assert_eq!(frame.len(),68);
            assert_eq!(encode_direct_write(&frame).unwrap()[2],68);
            let different=frame.iter().any(|key| key.rgb!=frame[0].rgb);
            assert!(different, "scene {name} should not collapse to one static RGB value");
        }
    }
    #[test]
    fn haze_uses_both_temporal_layers_and_aperture() {
        let c=Config {brightness:1.0,..Config::default()};
        let a=special_scene("haze",0.4,0.3,0.1,c).unwrap();
        let b=special_scene("haze",0.4,0.3,1.4,c).unwrap();
        assert_ne!(a,b);
    }
    #[test]
    fn unknown_scenes_use_factory_fallback() {
        assert!(special_scene("aqua",0.0,0.0,0.0,Config::default()).is_none());
    }
}
