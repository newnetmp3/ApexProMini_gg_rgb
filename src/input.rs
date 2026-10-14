//! Linux evdev keyboard input for reactive illumination.
//! No keyboard grab: ordinary typing continues uninterrupted.
//! Reads only an explicitly selected event node belonging to USB 1038:161e.
//! Processes key-down *codes* in memory; no typed text, files, or network access.
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use crate::{PRODUCT_ID, VENDOR_ID};

/// Map Linux input-event-codes.h KEY_* values to USB keyboard HID usages.
/// Unmapped keys (mouse/media/unknown usages) do not trigger lighting.
pub fn linux_key_to_hid(key: u16) -> Option<u8> {
    match key {
        1 => Some(41),  // Escape
        2..=11 => Some((key - 2) as u8 + 30), // numbers 1..0
        12 => Some(45), 13 => Some(46), 14 => Some(42),
        15 => Some(43),
        16 => Some(20), 17 => Some(26), 18 => Some(8), 19 => Some(21), 20 => Some(23),
        21 => Some(28), 22 => Some(24), 23 => Some(12), 24 => Some(18), 25 => Some(19),
        26 => Some(47), 27 => Some(48), 28 => Some(40), 29 => Some(224),
        30 => Some(4), 31 => Some(22), 32 => Some(7), 33 => Some(9), 34 => Some(10),
        35 => Some(11), 36 => Some(13), 37 => Some(14), 38 => Some(15), 39 => Some(51),
        40 => Some(52), 41 => Some(53), 42 => Some(225), 43 => Some(49),
        44 => Some(29), 45 => Some(27), 46 => Some(6), 47 => Some(25), 48 => Some(5),
        49 => Some(17), 50 => Some(16), 51 => Some(54), 52 => Some(55), 53 => Some(56),
        54 => Some(229), 56 => Some(226), 57 => Some(44), 58 => Some(57),
        86 => Some(100), 97 => Some(228), 100 => Some(230),
        125 => Some(227), 126 => Some(231),
        _ => None,
    }
}

fn validate_event_name(name: &str) -> bool {
    name.strip_prefix("event")
        .is_some_and(|num| !num.is_empty() && num.bytes().all(|b| b.is_ascii_digit()))
}

fn is_target_usb_ancestor(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        let vendor = fs::read_to_string(ancestor.join("idVendor")).ok();
        let product = fs::read_to_string(ancestor.join("idProduct")).ok();
        if let (Some(vendor), Some(product)) = (vendor, product) {
            if u16::from_str_radix(vendor.trim(), 16) == Ok(VENDOR_ID)
                && u16::from_str_radix(product.trim(), 16) == Ok(PRODUCT_ID)
            {
                return true;
            }
        }
    }
    false
}

fn has_letter_key_capability(path: &Path) -> bool {
    let text = match fs::read_to_string(path.join("device/capabilities/key")) {
        Ok(text) => text,
        Err(_) => return false,
    };
    // A KEY_A press is key code 30. Key capability bitmap is printed in hex
    // chunks, most-significant chunk first. The last chunk covers lowest codes.
    text.split_whitespace().last()
        .and_then(|part| u64::from_str_radix(part, 16).ok())
        .is_some_and(|word| (word & (1_u64 << 30)) != 0)
}

pub fn matching_event_devices() -> Vec<PathBuf> {
    let mut matches = Vec::new();
    let Ok(nodes) = fs::read_dir("/sys/class/input") else { return matches; };
    for node in nodes.flatten() {
        let name = node.file_name().to_string_lossy().into_owned();
        if !validate_event_name(&name) { continue; }
        let Ok(full_path) = fs::canonicalize(node.path()) else { continue; };
        if is_target_usb_ancestor(&full_path) && has_letter_key_capability(&node.path()) {
            matches.push(PathBuf::from("/dev/input").join(name));
        }
    }
    matches.sort();
    matches
}

/// Spawn a reader for keydown events. The caller MUST choose an exact event path
/// for the wired Mini's own keyboard interface. No arbitrary-device capture.
pub fn start_reader(path: &Path) -> Result<Receiver<u8>, String> {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("Input must be /dev/input/eventN")?;
    if !validate_event_name(name) || path != Path::new("/dev/input").join(name) {
        return Err("Expected an explicit /dev/input/eventN device path".into());
    }
    if !matching_event_devices().iter().any(|candidate| candidate == path) {
        return Err(format!("{path:?} is not a recognized wired Apex Pro Mini keyboard input device; use --list-inputs"));
    }
    let mut input = File::open(path).map_err(|e| format!("Cannot read {path:?}: {e}. Check seat ACL/udev permissions; do not run as root by default."))?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // input_event: struct timeval (two native longs), u16 type,
        // u16 code, i32 value. Linux x86_64 uses 24 bytes.
        let timeval_len = 2 * std::mem::size_of::<isize>();
        let mut bytes = vec![0u8; timeval_len + 8];
        while input.read_exact(&mut bytes).is_ok() {
            let kind = u16::from_ne_bytes([bytes[timeval_len], bytes[timeval_len + 1]]);
            let code = u16::from_ne_bytes([bytes[timeval_len + 2], bytes[timeval_len + 3]]);
            let value = i32::from_ne_bytes(bytes[timeval_len + 4..timeval_len + 8].try_into().unwrap());
            if kind == 1 && value == 1 {
                if let Some(hid) = linux_key_to_hid(code) {
                    if tx.send(hid).is_err() { break; }
                }
            }
        }
    });
    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_wired_mini_letter_keypresses() {
        assert_eq!(linux_key_to_hid(30), Some(4));  // A
        assert_eq!(linux_key_to_hid(31), Some(22)); // S
        assert_eq!(linux_key_to_hid(17), Some(26)); // W
        assert_eq!(linux_key_to_hid(57), Some(44)); // Space
        assert_eq!(linux_key_to_hid(999), None);
    }
    #[test]
    fn input_node_name_is_strictly_validated() {
        assert!(validate_event_name("event12"));
        assert!(!validate_event_name("event0/evil"));
        assert!(!validate_event_name("event"));
        assert!(!validate_event_name("mice"));
    }
}
