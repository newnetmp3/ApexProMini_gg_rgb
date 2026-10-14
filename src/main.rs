//! Read-only by default. Explicit confirmation is required for all HID writes.
//! Only supports wired SteelSeries Apex Pro Mini USB 1038:161e.
#[cfg(target_os = "linux")]
mod animate;
use apex_pro_mini::{
    encode_clear_direct_write, encode_direct_write, hid_from_key, parse_report_descriptor,
    rgb_from_hex, KeyColor, ReportKind, FEATURE_REPORT_BYTES, OUTPUT_REPORT_BYTES, PRODUCT_ID,
    VENDOR_ID,
};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ControlEndpoint {
    device: PathBuf,
}

fn parse_target_usb_id(uevent: &str) -> bool {
    for line in uevent.lines() {
        if let Some(value) = line.strip_prefix("HID_ID=") {
            let parts: Vec<_> = value.split(':').collect();
            if parts.len() == 3 {
                if let (Ok(vendor), Ok(product)) = (
                    u32::from_str_radix(parts[1], 16),
                    u32::from_str_radix(parts[2], 16),
                ) {
                    if vendor == VENDOR_ID as u32 && product == PRODUCT_ID as u32 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn discover_control() -> Result<ControlEndpoint, String> {
    let mut matches = Vec::new();
    let root = Path::new("/sys/class/hidraw");
    let entries = fs::read_dir(root).map_err(|e| format!("Cannot inspect {root:?}: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let node = entry.path();
        let uevent_path = node.join("device/uevent");
        let uevent = match fs::read_to_string(uevent_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !parse_target_usb_id(&uevent) {
            continue;
        }
        let raw = fs::read(node.join("device/report_descriptor"))
            .map_err(|e| format!("Unable to read Mini HID descriptor: {e}"))?;
        let reports = parse_report_descriptor(&raw)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = PathBuf::from(format!("/dev/{name}"));
        println!(
            "{path:?}: control_page={} feature={:?} output={:?}",
            reports.has_vendor_control_page,
            reports.report_bytes(0, ReportKind::Feature),
            reports.report_bytes(0, ReportKind::Output)
        );
        if reports.matches_mini() {
            matches.push(ControlEndpoint { device: path });
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(format!(
            "No Mini vendor HID collection advertising exactly {FEATURE_REPORT_BYTES}-byte Feature and {OUTPUT_REPORT_BYTES}-byte Output reports. Refusing device writes."
        )),
        _ => Err("Multiple matching Mini control collections found; refusing ambiguous device write.".into()),
    }
}

fn print_bytes(bytes: &[u8]) {
    print!("{} bytes:", bytes.len());
    for b in bytes.iter().take(32) {
        print!(" {b:02x}");
    }
    if bytes.len() > 32 {
        print!(" ... (remaining bytes not shown)");
    }
    println!();
}

fn usage() {
    eprintln!(
        "Usage:\n  apex-mini-rgb                          # read-only discovery\n  apex-mini-rgb --dry-run A ff0000       # no USB write\n  apex-mini-rgb --write-key A ff0000 --confirm-device-write\n  apex-mini-rgb --clear --confirm-device-write\n\nDevice writes are EXPERIMENTAL, always opt-in. The default command is read-only."
    );
}

fn open_guarded(endpoint: &ControlEndpoint) -> Result<File, String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(&endpoint.device)
        .map_err(|e| format!("Cannot open {:?} for HID write: {e}. Do not use sudo blindly; inspect udev permissions.", endpoint.device))
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: std::os::raw::c_ulong, ...) -> std::os::raw::c_int;
}

#[cfg(target_os = "linux")]
fn send_feature(file: &File, mut data: [u8; FEATURE_REPORT_BYTES], verbose: bool) -> Result<(), String> {
    use std::os::fd::AsRawFd;
    // Kernel HIDIOCSFEATURE(len) = _IOWR('H', 0x06, len).
    // Data[0] is report ID 0; this call is for the *feature* report, not write(2).
    let request: std::os::raw::c_ulong = ((3u64 << 30)
        | ((FEATURE_REPORT_BYTES as u64) << 16)
        | ((b'H' as u64) << 8)
        | 0x06) as _;
    let rc = unsafe { ioctl(file.as_raw_fd(), request, data.as_mut_ptr()) };
    if rc < 0 {
        return Err(format!("HIDIOCSFEATURE failed: {}", io::Error::last_os_error()));
    }
    if verbose {
        println!("Feature report write submitted (kernel return {rc}); verify visible effect on hardware.");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn send_feature(_file: &File, _data: [u8; FEATURE_REPORT_BYTES], _verbose: bool) -> Result<(), String> {
    Err("Live HID writes require Linux".into())
}

fn send_output(mut file: File, data: [u8; OUTPUT_REPORT_BYTES]) -> Result<(), String> {
    let written = file.write(&data).map_err(|e| format!("HID output write failed: {e}"))?;
    if written != data.len() {
        return Err(format!("Short HID output write: wrote {written} of {} bytes", data.len()));
    }
    println!("Clear-direct-mode Output report submitted; device reaction is not confirmed.");
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if args.first().is_some_and(|arg| matches!(arg.as_str(), "--list-inputs" | "--effect" | "--preview-effect")) {
        return animate::run_effect(args);
    }
    match args {
        [] => {
            let endpoint = discover_control()?;
            println!("Verified control endpoint {:?}. No USB writes performed.", endpoint.device);
            Ok(())
        }
        [dry, key, color] if dry == "--dry-run" => {
            let hid = hid_from_key(key)?;
            let rgb = rgb_from_hex(color)?;
            let packet = encode_direct_write(&[KeyColor { hid, rgb }])?;
            println!("Dry run for key {key} (HID {hid})");
            print_bytes(&packet);
            println!("No USB writes performed.");
            Ok(())
        }
        [write, key, color, confirm]
            if write == "--write-key" && confirm == "--confirm-device-write" =>
        {
            let hid = hid_from_key(key)?;
            let rgb = rgb_from_hex(color)?;
            let report = encode_direct_write(&[KeyColor { hid, rgb }])?;
            let endpoint = discover_control()?;
            let file = open_guarded(&endpoint)?;
            println!(
                "WARNING: Experimental static RGB feature report to {:?}, HID {hid}. This can change lighting until you --clear.",
                endpoint.device
            );
            send_feature(&file, report, true)
        }
        [clear, confirm] if clear == "--clear" && confirm == "--confirm-device-write" => {
            let endpoint = discover_control()?;
            let file = open_guarded(&endpoint)?;
            send_output(file, encode_clear_direct_write())
        }
        _ => {
            usage();
            Err("Invalid command or missing --confirm-device-write (nothing sent).".into())
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(err) = run(&args) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_target_usb_id;

    #[test]
    fn matches_only_exact_mini_vid_pid() {
        assert!(parse_target_usb_id("HID_ID=0003:00001038:0000161E"));
        assert!(!parse_target_usb_id("HID_ID=0003:00001038:00001628"));
        assert!(!parse_target_usb_id("HID_ID=0003:00001037:0000161E"));
    }
}
