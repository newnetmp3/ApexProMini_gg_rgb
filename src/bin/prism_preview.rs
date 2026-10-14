//! Deterministic, read-only visual-regression gallery for factory-style Prism
//! rendering. Generates SVG without opening hidraw, evdev, GUI or the network.
//! Each panel shows the same 68-key Rust frame as the real Mini encoder.
use apex_pro_mini::effects::{self, Config, KeyHit, Preset};
use apex_pro_mini::factory;
use apex_pro_mini::{KeyColor, Rgb};
use std::fmt::Write as _;
use std::path::PathBuf;

const TILE_W: f32 = 794.0;
const TILE_H: f32 = 150.0;
const COLUMNS: usize = 2;
const SPACING: f32 = 18.0;
const MARGIN: f32 = 24.0;
const HEADER: f32 = 122.0;

fn rgb_hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue)
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&apos;")
}

/// Keep reactive snapshots proportional to the source trigger lifetime.
/// A 1250ms Single Key should not be previewed only at 30–180ms, and a
/// 250ms ripple must show its propagation before its source period ends.
fn preview_time(index: u8, phase: usize) -> f32 {
    let spec = &factory::PRESETS[index as usize];
    if factory::is_reactive(index) {
        if spec.period_ms == 0 {
            [0.03, 0.6, 1.5][phase]
        } else {
            spec.period_ms as f32 / 1000.0 * [0.08, 0.4, 0.8][phase]
        }
    } else if spec.period_ms > 0 {
        spec.period_ms as f32 / 1000.0 * [0.0, 0.33, 0.67][phase]
    } else {
        0.0
    }
}

/// Generate a static gallery with three effect phases per historical selector.
/// Reactive presets receive SYNTHETIC key A events; no input device is opened.
pub fn build_svg() -> String {
    let count = factory::PRESETS.len();
    let rows = count.div_ceil(COLUMNS);
    let width = MARGIN * 2.0 + TILE_W * COLUMNS as f32 + SPACING * (COLUMNS - 1) as f32;
    let height = HEADER + MARGIN + TILE_H * rows as f32 + SPACING * (rows - 1) as f32;
    let mut svg = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}" viewBox="0 0 {width:.0} {height:.0}">
<rect width="100%" height="100%" fill="#0a1020"/>
<text x="{MARGIN}" y="50" fill="#f7fafc" font-family="sans-serif" font-size="30" font-weight="bold">Apex Mini • historical Prism preset previews</text>
<text x="{MARGIN}" y="79" fill="#abb9d2" font-family="sans-serif" font-size="17">31 historical identifiers • 30 source definitions + redPulse/faze rename • three synthetic animation phases</text>
<text x="{MARGIN}" y="105" fill="#ffc267" font-family="sans-serif" font-size="16">Functional parameter-derived approximations; NOT visually verified copies of official SteelSeries GG presets</text>
"##);
    let config = Config {
        brightness: 1.0, speed: 1.0, ..Config::default()
    };
    for (index, spec) in factory::PRESETS.iter().enumerate() {
        let col = index % COLUMNS;
        let row = index / COLUMNS;
        let origin_x = MARGIN + col as f32 * (TILE_W + SPACING);
        let origin_y = HEADER + row as f32 * (TILE_H + SPACING);
        let _ = write!(svg, r##"<rect x="{origin_x:.1}" y="{origin_y:.1}" width="{TILE_W}" height="{TILE_H}" rx="14" fill="#172238" stroke="#33445f" stroke-width="1"/>
<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="17" font-weight="bold" fill="#f6f8ff">{}</text>
<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="12" fill="#93a5bd">{} • {} source variant(s) • period {} ms</text>
"##,
            origin_x + 16.0, origin_y + 26.0, esc(spec.cli),
            origin_x + 16.0, origin_y + 44.0,
            if factory::is_reactive(index as u8) { "keypress-based" } else { "base/idle" },
            spec.variants_in_source, spec.period_ms);
        for phase in 0..3 {
            let time = preview_time(index as u8, phase);
            let hit = [KeyHit { hid: 4, when: 0.0 }];
            let hits: &[KeyHit] = if factory::is_reactive(index as u8) {
                &hit
            } else { &[] };
            let colors = effects::frame(Preset::Factory(index as u8), config, time, hits);
            render_keyboard(&mut svg, &colors, origin_x + 13.0 + phase as f32 * 255.0,
                origin_y + 68.0);
            let _ = write!(svg,
                r##"<text x="{:.1}" y="{:.1}" fill="#a8b7ce" font-family="sans-serif" font-size="11">phase {} • {:.2} s</text>"##,
                origin_x + 18.0 + phase as f32 * 255.0, origin_y + 138.0,
                phase + 1, time);
        }
    }
    svg.push_str("</svg>\n");
    svg
}
fn render_keyboard(svg: &mut String, colors: &[KeyColor], ox: f32, oy: f32) {
    let pitch_x = 15.0;
    let pitch_y = 11.5;
    for key in colors {
        let Some((x, y)) = effects::position(key.hid) else { continue };
        let px = ox + x * 14.0 * pitch_x;
        let py = oy + y * 4.0 * pitch_y;
        let w = if key.hid == 44 { pitch_x * 2.2 } else { 12.6 };
        let fill = rgb_hex(key.rgb);
        let _ = write!(svg,
            r##"<rect x="{px:.1}" y="{py:.1}" width="{w:.1}" height="9.5" rx="2.1" fill="{fill}" stroke="#c4d2e5" stroke-width=".4"><title>HID {} : {fill}</title></rect>"##,
            key.hid);
    }
}

fn parse_args() -> Result<PathBuf, String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => Ok(PathBuf::from("prism-presets-preview.svg")),
        [flag, path] if flag == "--output" => Ok(PathBuf::from(path)),
        _ => Err("Usage: prism-preview [--output path/to/prism-presets-preview.svg]".into()),
    }
}

fn main() {
    let output = match parse_args() {
        Ok(output) => output,
        Err(message) => { eprintln!("{message}"); std::process::exit(2); }
    };
    if let Some(parent) = output.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            eprintln!("Cannot create output folder: {error}");
            std::process::exit(1);
        }
    }
    let svg = build_svg();
    if let Err(error) = std::fs::write(&output, svg) {
        eprintln!("Cannot write gallery: {error}");
        std::process::exit(1);
    }
    println!(
        "Created {} (31 historical factory-style previews). Read-only: no USB device, input events, or network accessed.",
        output.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_modes_have_three_preview_frames_and_svg_closes() {
        let svg = build_svg();
        assert_eq!(svg.matches("source variant(s)").count(), 31);
        assert_eq!(svg.matches("phase 1").count(), 31);
        assert_eq!(svg.matches("phase 2").count(), 31);
        assert_eq!(svg.matches("phase 3").count(), 31);
        assert!(svg.contains("gg:prism"));
        assert!(svg.contains("gg:redPulse"));
        assert!(svg.ends_with("</svg>\n"));
    }
    #[test]
    fn reactive_previews_scale_to_factory_trigger_period() {
        let ripple = factory::find("gg:reactiveRipple").unwrap();
        let line = factory::find("gg:reactiveLine").unwrap();
        let single = factory::find("gg:reactiveSingleKey").unwrap();
        assert!((preview_time(ripple, 2) - 0.2).abs() < 1e-6);
        assert!((preview_time(line, 2) - 0.4).abs() < 1e-6);
        assert!((preview_time(single, 2) - 1.0).abs() < 1e-6);
        assert!(preview_time(single, 2) > preview_time(ripple, 2));
    }

    #[test]
    fn svg_escapes_user_content() {
        assert_eq!(esc("<test & hi>"), "&lt;test &amp; hi&gt;");
    }
}
