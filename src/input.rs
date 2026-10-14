//! Linux evdev keypress source for reactive keyboard lighting.
//! This module NEVER grabs input, persists keystrokes, or transmits them.
//! A reader is permitted only for a matching wired Apex Pro Mini (1038:161e).
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::{ADDRESSABLE_HID_CODES, PRODUCT_ID, VENDOR_ID};

pub const EVENT_BYTES: usize = 2 * std::mem::size_of::<isize>() + 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMessage {
    /// Physical key-down mapped to a supported Mini HID key usage.
    Press { hid: u8 },
    /// Only sent for explicit diagnostics. Not retained or logged by the daemon.
    UnsupportedKey,
    /// A reader ended (e.g., the keyboard was unplugged).
    ReaderStopped { device: PathBuf, reason: String },
}

/// Linux input-event-codes.h KEY_* -> USB keyboard HID usages.
pub fn linux_key_to_hid(key: u16) -> Option<u8> {
    match key {
        1 => Some(41),
        2..=10 => Some((key - 2) as u8 + 30),
        11 => Some(39), // KEY_0 is USB HID 39 (not 39+1)
        12 => Some(45), 13 => Some(46), 14 => Some(42), 15 => Some(43),
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

/// Probe the lowest 64 EV_KEY codes, where KEY_A = 30. Do not assume the
/// keyboard is event0 or that the first USB HID interface emits typing events.
fn has_letter_key_capability(path: &Path) -> bool {
    let Ok(bitmap) = fs::read_to_string(path.join("device/capabilities/key")) else {
        return false;
    };
    let Some(lowest_word) = bitmap.split_whitespace().last() else {
        return false;
    };
    u64::from_str_radix(lowest_word, 16)
        .is_ok_and(|word| word & (1u64 << 30) != 0)
}

pub fn matching_event_devices() -> Vec<PathBuf> {
    let mut matches = Vec::new();
    let Ok(nodes) = fs::read_dir("/sys/class/input") else {
        return matches;
    };
    for node in nodes.flatten() {
        let name = node.file_name().to_string_lossy().into_owned();
        if !validate_event_name(&name) {
            continue;
        }
        let Ok(full_path) = fs::canonicalize(node.path()) else {
            continue;
        };
        if is_target_usb_ancestor(&full_path) && has_letter_key_capability(&node.path()) {
            matches.push(PathBuf::from("/dev/input").join(name));
        }
    }
    matches.sort();
    matches
}

fn validate_selected_device(path: &Path) -> Result<(), String> {
    let name = path.file_name().and_then(|n| n.to_str())
        .ok_or("Input must be an exact /dev/input/eventN path")?;
    if !validate_event_name(name) || path != Path::new("/dev/input").join(name) {
        return Err("Input must be an exact /dev/input/eventN path".into());
    }
    if !matching_event_devices().iter().any(|candidate| candidate == path) {
        return Err(format!("{path:?} is not a verified wired Apex Pro Mini keyboard input event node; inspect --list-inputs"));
    }
    Ok(())
}

/// Extract an EV_KEY press from the native Linux input_event structure.
/// Repeats (2), releases (0), other event types and unsupported HID usages
/// return None, avoiding key bounce, auto-repeat storms, and non-key events.
pub fn decode_keydown(bytes: &[u8]) -> Option<u8> {
    if bytes.len() != EVENT_BYTES {
        return None;
    }
    let offset = 2 * std::mem::size_of::<isize>();
    let event_type = u16::from_ne_bytes([bytes[offset], bytes[offset + 1]]);
    let code = u16::from_ne_bytes([bytes[offset + 2], bytes[offset + 3]]);
    let value = i32::from_ne_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
    if event_type != 1 || value != 1 {
        return None;
    }
    let hid = linux_key_to_hid(code)?;
    ADDRESSABLE_HID_CODES.contains(&hid).then_some(hid)
}

/// The process reads every validated wired-Mini keyboard event node rather
/// than guessing which one is active. No evdev grabs, no key capture logs.
pub fn start_readers(selection: Option<&Path>) -> Result<(Receiver<InputMessage>, usize), String> {
    let paths = if let Some(path) = selection {
        validate_selected_device(path)?;
        vec![path.to_path_buf()]
    } else {
        matching_event_devices()
    };
    if paths.is_empty() {
        return Err("No wired Mini input event device matched KEY_A capability. Run --list-inputs; check USB connection and /sys/class/input.".into());
    }
    let (sender, receiver) = mpsc::channel();
    let mut active = 0usize;
    let mut errors = Vec::new();
    for path in paths {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                errors.push(format!("{}: {err}", path.display()));
                continue;
            }
        };
        spawn_reader(file, path, sender.clone());
        active += 1;
    }
    drop(sender);
    if active == 0 {
        return Err(format!("Could not read any verified wired Mini input event nodes: {}. Check seat/udev ACLs (do not run as root blindly).", errors.join("; ")));
    }
    if !errors.is_empty() {
        eprintln!("Warning: some Mini input nodes are unreadable: {}", errors.join("; "));
    }
    Ok((receiver, active))
}

fn spawn_reader(mut file: File, path: PathBuf, sender: Sender<InputMessage>) {
    std::thread::spawn(move || {
        let mut bytes = [0u8; EVENT_BYTES];
        loop {
            match file.read_exact(&mut bytes) {
                Ok(()) => {
                    if let Some(hid) = decode_keydown(&bytes) {
                        if sender.send(InputMessage::Press { hid }).is_err() {
                            return;
                        }
                    }
                }
                Err(err) => {
                    let _ = sender.send(InputMessage::ReaderStopped {
                        device: path,
                        reason: err.to_string(),
                    });
                    return;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_event(kind: u16, key: u16, value: i32) -> [u8; EVENT_BYTES] {
        let mut bytes = [0u8; EVENT_BYTES];
        let pos = 2 * std::mem::size_of::<isize>();
        bytes[pos..pos + 2].copy_from_slice(&kind.to_ne_bytes());
        bytes[pos + 2..pos + 4].copy_from_slice(&key.to_ne_bytes());
        bytes[pos + 4..pos + 8].copy_from_slice(&value.to_ne_bytes());
        bytes
    }

    #[test]
    fn decodes_native_evdev_keydown_and_filters_repeats() {
        assert_eq!(decode_keydown(&raw_event(1, 30, 1)), Some(4)); // A
        assert_eq!(decode_keydown(&raw_event(1, 31, 1)), Some(22)); // S
        assert_eq!(decode_keydown(&raw_event(1, 11, 1)), Some(39)); // 0 regression
        assert_eq!(decode_keydown(&raw_event(1, 30, 0)), None);
        assert_eq!(decode_keydown(&raw_event(1, 30, 2)), None);
        assert_eq!(decode_keydown(&raw_event(0, 30, 1)), None);
        assert_eq!(decode_keydown(&raw_event(1, 41, 1)), None); // not in Mini layout
        assert_eq!(decode_keydown(&[]), None);
    }
    #[test]
    fn validates_event_node_name_strictly() {
        assert!(validate_event_name("event12"));
        assert!(!validate_event_name("event0/evil"));
        assert!(!validate_event_name("event"));
        assert!(!validate_event_name("mice"));
    }
    #[test]
    fn expected_linux_hid_mappings() {
        assert_eq!(linux_key_to_hid(30), Some(4));
        assert_eq!(linux_key_to_hid(31), Some(22));
        assert_eq!(linux_key_to_hid(17), Some(26));
        assert_eq!(linux_key_to_hid(57), Some(44));
        assert_eq!(linux_key_to_hid(11), Some(39));
        assert_eq!(linux_key_to_hid(999), None);
    }
}
