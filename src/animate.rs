//! Runtime animation loop for the wired Apex Pro Mini. All writes require opt-in.
//! Ripple and Reactive read *only* the verified USB keyboard's evdev device;
//! no key strings, timestamps, or input events are persisted.
use apex_pro_mini::effects::{self, Config, KeyHit, Preset, PRESET_NAMES};
use apex_pro_mini::{encode_clear_direct_write, encode_direct_write, rgb_from_hex, Rgb};
use apex_pro_mini::input;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::{discover_control, open_guarded, print_bytes, send_feature};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn stop_requested(_signal: i32) {
    RUNNING.store(false, Ordering::Relaxed);
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn signal(signal_number: i32, handler: extern "C" fn(i32)) -> usize;
}

fn register_exit_handlers() {
    // Signals only flip an atomic flag. Restore is performed outside the handler.
    #[cfg(target_os = "linux")]
    unsafe {
        signal(2, stop_requested);  // SIGINT (Ctrl+C)
        signal(15, stop_requested); // SIGTERM
    }
}

#[derive(Clone, Debug)]
struct Options {
    preset: Preset,
    config: Config,
    fps: u32,
    input: Option<PathBuf>,
    confirmed: bool,
    preview: bool,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let preview = args.first().is_some_and(|arg| arg == "--preview-effect");
    if args.len() < 2 || (!preview && args[0] != "--effect") {
        return Err(format!(
            "Use --effect PRESET or --preview-effect PRESET. Available presets: {PRESET_NAMES}"
        ));
    }
    let preset = Preset::from_name(&args[1])
        .ok_or_else(|| format!("Unknown effect {:?}; choose from {PRESET_NAMES}", args[1]))?;
    let mut options = Options {
        preset,
        config: Config::default(),
        fps: 24,
        input: None,
        confirmed: false,
        preview,
    };
    let mut index = 2;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--confirm-device-write" {
            if options.confirmed { return Err("Duplicate confirmation flag".into()); }
            options.confirmed = true;
            index += 1;
            continue;
        }
        let Some(value) = args.get(index + 1) else {
            return Err(format!("Missing value for {flag}"));
        };
        match flag.as_str() {
            "--color" => options.config.base = rgb_from_hex(value)?,
            "--accent" => options.config.accent = rgb_from_hex(value)?,
            "--fps" => {
                options.fps = value.parse::<u32>()
                    .map_err(|_| format!("Invalid FPS: {value}"))?;
                if !(5..=40).contains(&options.fps) {
                    return Err("FPS must be between 5 and 40".into());
                }
            }
            "--speed" => {
                options.config.speed = value.parse::<f32>()
                    .map_err(|_| format!("Invalid speed: {value}"))?;
                if !options.config.speed.is_finite()
                    || !(0.1..=5.0).contains(&options.config.speed) {
                    return Err("Speed must be finite and between 0.1 and 5".into());
                }
            }
            "--brightness" => {
                options.config.brightness = value.parse::<f32>()
                    .map_err(|_| format!("Invalid brightness: {value}"))?;
                if !options.config.brightness.is_finite()
                    || !(0.0..=1.0).contains(&options.config.brightness) {
                    return Err("Brightness must be finite and between 0 and 1".into());
                }
            }
            "--input" => options.input = Some(PathBuf::from(value)),
            _ => return Err(format!("Unknown flag {flag}")),
        }
        index += 2;
    }
    if options.preview && options.confirmed {
        return Err("Preview is always read-only; omit --confirm-device-write".into());
    }
    if !options.preview && !options.confirmed {
        return Err("Device writes require --confirm-device-write (nothing sent).".into());
    }
    if !options.preview && options.preset.needs_key_events() && options.input.is_none() {
        return Err("Ripple/reactive requires --input /dev/input/eventN; find the correct Mini device with --list-inputs".into());
    }
    if !options.preset.needs_key_events() && options.input.is_some() {
        return Err("--input is only valid for ripple and reactive effects".into());
    }
    Ok(options)
}

fn static_effect(preset: Preset) -> bool {
    matches!(preset, Preset::Solid | Preset::Gradient | Preset::Rainbow)
}

fn run_animation(options: Options) -> Result<(), String> {
    // Open evdev input BEFORE writing RGB so missing permissions cannot
    // strand the keyboard in direct lighting mode.
    let input_rx = if options.preset.needs_key_events() {
        Some(input::start_reader(options.input.as_ref().expect("validated input"))?)
    } else {
        None
    };
    let endpoint = discover_control()?;
    let file = open_guarded(&endpoint)?;
    let clock = Instant::now();
    let mut hits = Vec::<KeyHit>::new();
    let mut previous = None;
    let interval = Duration::from_secs_f64(1.0 / f64::from(options.fps));
    RUNNING.store(true, Ordering::Relaxed);
    register_exit_handlers();
    println!(
        "Running {:?} on {:?} ({} fps, speed {:.2}). Ctrl+C restores onboard lighting.",
        options.preset, endpoint.device, options.fps, options.config.speed
    );
    if options.preset.needs_key_events() {
        println!("Passive keypress illumination; normal key typing is not intercepted or recorded.");
    }

    let mut next_frame = Instant::now();
    let outcome = loop {
        if !RUNNING.load(Ordering::Relaxed) { break Ok(()); }
        let elapsed = clock.elapsed().as_secs_f32();
        if let Some(receiver) = &input_rx {
            while let Ok(hid) = receiver.try_recv() {
                hits.push(KeyHit { hid, when: elapsed });
                if hits.len() > 32 { hits.remove(0); }
            }
            hits.retain(|hit| elapsed - hit.when < 2.5);
        }
        let colors = effects::frame(options.preset, options.config, elapsed, &hits);
        let packet = match encode_direct_write(&colors) {
            Ok(report) => report,
            Err(e) => break Err(e),
        };
        let must_send = !static_effect(options.preset)
            || previous.as_ref() != Some(&packet);
        if must_send {
            if let Err(error) = send_feature(&file, packet, false) {
                break Err(error);
            }
            previous = Some(packet);
        }
        next_frame += interval;
        let now = Instant::now();
        if let Some(remaining) = next_frame.checked_duration_since(now) {
            std::thread::sleep(remaining);
        } else {
            next_frame = now;
        }
    };

    // Even on a failed frame send, attempt to return control to onboard lighting.
    use crate::send_output;
    let clear = send_output(file, encode_clear_direct_write());
    match (outcome, clear) {
        (Err(error), Err(clear_error)) =>
            Err(format!("Effect error: {error}; ALSO failed to restore lighting: {clear_error}")),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(format!("Effect ended; failed to restore onboard lighting: {error}")),
        (Ok(_), Ok(_)) => Ok(()),
    }
}

pub fn run_effect(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| arg == "--list-inputs") {
        if args.len() != 1 {
            return Err("--list-inputs accepts no other arguments".into());
        }
        let options = input::matching_event_devices();
        if options.is_empty() {
            println!("No matching Apex Pro Mini keyboard event devices detected.");
            println!("Check that it is connected via USB; the control interface may still be available.");
        } else {
            println!("USB 1038:161e keyboard event devices (no reads or writes performed):");
            for path in options {
                println!("  {}", path.display());
            }
        }
        return Ok(());
    }
    let options = parse_args(args)?;
    if options.preview {
        let colors = effects::frame(options.preset, options.config, 0.5, &[]);
        let packet = encode_direct_write(&colors)?;
        println!("Preview {:?} at t=0.5 s ({} keys)", options.preset, colors.len());
        print_bytes(&packet);
        println!("Read-only preview: no keyboard or event device opened.");
        return Ok(());
    }
    run_animation(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(tokens: &[&str]) -> Result<Options, String> {
        parse_args(&tokens.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
    }
    #[test]
    fn effects_cannot_send_without_explicit_confirmation() {
        assert!(parse(&["--effect", "wave"]).is_err());
        assert!(parse(&["--effect", "ripple", "--input", "/dev/input/event3"]).is_err());
        assert!(parse(&["--preview-effect", "rainbow"]).is_ok());
        assert!(parse(&["--preview-effect", "rainbow", "--confirm-device-write"]).is_err());
    }
    #[test]
    fn reactive_effect_requires_scoped_input_and_bounds_fps() {
        assert!(parse(&["--effect", "ripple", "--confirm-device-write"]).is_err());
        assert!(parse(&["--effect", "ripple", "--input", "/dev/input/event3", "--confirm-device-write"]).is_ok());
        assert!(parse(&["--effect", "wave", "--fps", "999", "--confirm-device-write"]).is_err());
        assert!(parse(&["--effect", "wave", "--speed", "NaN", "--confirm-device-write"]).is_err());
        assert!(parse(&["--effect", "wave", "--input", "/dev/input/event2", "--confirm-device-write"]).is_err());
    }
}
