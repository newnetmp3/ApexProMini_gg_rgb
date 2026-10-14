//! Independent, minimal wired Apex Pro Mini RGB command serializer.
//! Based on the device's official GG 108.3 decrypted specifications, inspected
//! locally. No vendor source or firmware is bundled or executed.
//!
//! Byte layout verified by static specification; the wired Mini A-key red
//! color response was confirmed by an owner hardware test on 2026-10-14.
//! A-key red, S-key green, and 0x41 clear have been observed on hardware.
//! Simultaneous multi-key effects are the next verification milestone.

pub mod effects;
#[cfg(target_os = "linux")]
pub mod input;

pub const VENDOR_ID: u16 = 0x1038;
pub const PRODUCT_ID: u16 = 0x161e;
pub const CONTROL_USAGE_PAGE: u32 = 0xffc0;
pub const OUTPUT_REPORT_BYTES: usize = 65;
pub const FEATURE_REPORT_BYTES: usize = 513;
pub const FIRMWARE_KEY_SLOTS: usize = 70;
pub const ADDRESSABLE_KEY_COUNT: usize = 68;
pub const DIRECT_WRITE_COMMAND: u8 = 0x40;
pub const CLEAR_DIRECT_WRITE_COMMAND: u8 = 0x41;

// Logical HID usage codes referenced by the wired Mini device definition.
// The firmware has 70 indexed slots, but two of those slots are unused.
// These are valid logical codes, not an assertion about the current layout.
pub const ADDRESSABLE_HID_CODES: [u8; ADDRESSABLE_KEY_COUNT] = [
    41, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 45, 46, 137,
    43, 20, 26, 8, 21, 23, 28, 24, 12, 18, 19, 47, 48, 49,
    57, 4, 22, 7, 9, 10, 11, 13, 14, 15, 51, 52, 50, 40,
    225, 100, 29, 27, 6, 25, 5, 17, 16, 54, 55, 56, 135, 229,
    224, 227, 226, 139, 44, 138, 136, 230, 231, 240, 228, 42,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyColor {
    pub hid: u8,
    pub rgb: Rgb,
}

pub fn rgb_from_hex(s: &str) -> Result<Rgb, String> {
    let trimmed = s.strip_prefix('#').unwrap_or(s);
    if trimmed.len() != 6 || !trimmed.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("RGB must be exactly six hexadecimal digits: {s}"));
    }
    let rgb = u32::from_str_radix(trimmed, 16).map_err(|e| e.to_string())?;
    Ok(Rgb {
        red: (rgb >> 16) as u8,
        green: (rgb >> 8) as u8,
        blue: rgb as u8,
    })
}

pub fn hid_from_key(key: &str) -> Result<u8, String> {
    let up = key.to_ascii_uppercase();
    let value = match up.as_str() {
        "SPACE" => 44,
        "ENTER" | "RETURN" => 40,
        "ESC" | "ESCAPE" => 41,
        "TAB" => 43,
        "BACKSPACE" => 42,
        _ if up.len() == 1 => {
            let b = up.as_bytes()[0];
            match b {
                b'A'..=b'Z' => b - b'A' + 4,
                b'1'..=b'9' => b - b'1' + 30,
                b'0' => 39,
                _ => return Err(format!("unsupported key: {key}")),
            }
        }
        _ => return Err(format!("unsupported key: {key}")),
    };
    if !ADDRESSABLE_HID_CODES.contains(&value) {
        return Err(format!("key {key} (HID {value}) is not in the wired Mini key table"));
    }
    Ok(value)
}

/// Construct the Mini's exact 513-byte feature-report envelope.
/// The official descriptor declares a key-count field plus 68 fixed slots
/// of (HID, R, G, B). Remaining report bytes are zero-padding.
/// Never silently truncate or invent HID indices.
pub fn encode_direct_write(keys: &[KeyColor]) -> Result<[u8; FEATURE_REPORT_BYTES], String> {
    if keys.len() > ADDRESSABLE_KEY_COUNT {
        return Err(format!("too many key colors: {} (max {ADDRESSABLE_KEY_COUNT})", keys.len()));
    }
    let mut used = [false; 256];
    let mut data = [0u8; FEATURE_REPORT_BYTES];
    data[0] = 0; // report ID
    data[1] = DIRECT_WRITE_COMMAND;
    data[2] = u8::try_from(keys.len()).expect("68 <= u8::MAX");
    for (index, entry) in keys.iter().enumerate() {
        if !ADDRESSABLE_HID_CODES.contains(&entry.hid) {
            return Err(format!("HID {} is not a supported Mini key", entry.hid));
        }
        if used[entry.hid as usize] {
            return Err(format!("duplicate HID key {}", entry.hid));
        }
        used[entry.hid as usize] = true;
        let offset = 3 + index * 4;
        data[offset] = entry.hid;
        data[offset + 1] = entry.rgb.red;
        data[offset + 2] = entry.rgb.green;
        data[offset + 3] = entry.rgb.blue;
    }
    Ok(data)
}

/// Produce a complete 68-key solid-color update for the Mini's official key list.
/// This is independent of physical region; optional ISO/JIS keys are included.
pub fn uniform_keys(rgb: Rgb) -> Vec<KeyColor> {
    ADDRESSABLE_HID_CODES.iter().copied().map(|hid| KeyColor { hid, rgb }).collect()
}

/// Direct mode release; **not** an APPLY/SAVE command.
pub fn encode_clear_direct_write() -> [u8; OUTPUT_REPORT_BYTES] {
    let mut data = [0u8; OUTPUT_REPORT_BYTES];
    data[1] = CLEAR_DIRECT_WRITE_COMMAND;
    data
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReportKind {
    Output,
    Feature,
}

#[derive(Debug, Default, Clone)]
pub struct ReportDescriptor {
    pub has_vendor_control_page: bool,
    pub reports: std::collections::BTreeMap<(u32, ReportKind), u32>,
}

impl ReportDescriptor {
    pub fn report_bytes(&self, id: u32, kind: ReportKind) -> Option<usize> {
        self.reports.get(&(id, kind)).map(|bits| (*bits as usize).div_ceil(8) + 1)
    }

    pub fn matches_mini(&self) -> bool {
        self.has_vendor_control_page
            && self.report_bytes(0, ReportKind::Output) == Some(OUTPUT_REPORT_BYTES)
            && self.report_bytes(0, ReportKind::Feature) == Some(FEATURE_REPORT_BYTES)
    }
}

/// Decode HID descriptor's Global report items, respecting PUSH/POP.
/// Reports are measured in payload bits; report-ID byte is added by report_bytes().
pub fn parse_report_descriptor(raw: &[u8]) -> Result<ReportDescriptor, String> {
    let mut result = ReportDescriptor::default();
    let mut offset = 0usize;
    let mut page = 0u32;
    let mut size_bits = 0u32;
    let mut count = 0u32;
    let mut report_id = 0u32;
    let mut stack = Vec::new();

    while offset < raw.len() {
        let prefix = raw[offset];
        offset += 1;
        if prefix == 0xfe {
            if offset + 2 > raw.len() {
                return Err("truncated long HID item header".into());
            }
            let len = raw[offset] as usize;
            offset += 2;
            if offset + len > raw.len() {
                return Err("truncated long HID item data".into());
            }
            offset += len;
            continue;
        }
        let len = [0usize, 1, 2, 4][(prefix & 0x03) as usize];
        if offset + len > raw.len() {
            return Err("truncated short HID item".into());
        }
        let mut value = 0u32;
        for index in 0..len {
            value |= (raw[offset + index] as u32) << (index * 8);
        }
        offset += len;
        let item_type = (prefix >> 2) & 3;
        let tag = prefix >> 4;
        if item_type == 1 {
            match tag {
                0 => {
                    page = value;
                    if page == CONTROL_USAGE_PAGE {
                        result.has_vendor_control_page = true;
                    }
                }
                7 => size_bits = value,
                8 => report_id = value,
                9 => count = value,
                10 => stack.push((page, size_bits, count, report_id)),
                11 => {
                    (page, size_bits, count, report_id) =
                        stack.pop().ok_or("unmatched HID global POP")?;
                }
                _ => {}
            }
        } else if item_type == 0 && (tag == 9 || tag == 11) {
            let kind = if tag == 9 { ReportKind::Output } else { ReportKind::Feature };
            let bits = size_bits.checked_mul(count).ok_or("HID report multiplication overflow")?;
            let current = result.reports.entry((report_id, kind)).or_insert(0);
            *current = current.checked_add(bits).ok_or("HID report size overflow")?;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_reports_are_not_tkl_reports() {
        assert_eq!(FEATURE_REPORT_BYTES, 513);
        assert_eq!(OUTPUT_REPORT_BYTES, 65);
        assert_eq!(ADDRESSABLE_KEY_COUNT, 68);
        assert_eq!(FIRMWARE_KEY_SLOTS, 70);
        assert_eq!(ADDRESSABLE_HID_CODES.iter().filter(|&&n| n == 4).count(), 1);
    }

    #[test]
    fn single_a_frame_matches_specification() {
        let frame = encode_direct_write(&[KeyColor {
            hid: 4,
            rgb: Rgb { red: 255, green: 0, blue: 0 },
        }]).unwrap();
        assert_eq!(frame.len(), 513);
        assert_eq!(&frame[..7], &[0x00, 0x40, 0x01, 0x04, 0xff, 0x00, 0x00]);
        assert!(frame[7..].iter().all(|&b| b == 0));
    }

    #[test]
    fn full_frame_fits_report_with_padding() {
        let rgb = Rgb { red: 1, green: 2, blue: 3 };
        let keys: Vec<_> = ADDRESSABLE_HID_CODES.iter().copied().map(|hid| KeyColor { hid, rgb }).collect();
        let frame = encode_direct_write(&keys).unwrap();
        assert_eq!(frame[2], 68);
        assert_eq!(&frame[3..7], &[41, 1, 2, 3]);
        assert_eq!(&frame[3 + 67 * 4..3 + 68 * 4], &[42, 1, 2, 3]);
        assert!(frame[3 + 68 * 4..].iter().all(|&b| b == 0));
    }

    #[test]
    fn reject_unsupported_or_duplicate_keys() {
        let red = Rgb { red: 255, green: 0, blue: 0 };
        assert!(encode_direct_write(&[KeyColor { hid: 0, rgb: red }]).is_err());
        assert!(encode_direct_write(&[KeyColor { hid: 4, rgb: red }, KeyColor { hid: 4, rgb: red }]).is_err());
    }

    #[test]
    fn uniform_colors_include_each_key_once() {
        let rgb = Rgb { red: 3, green: 22, blue: 189 };
        let keys = uniform_keys(rgb);
        assert_eq!(keys.len(), ADDRESSABLE_KEY_COUNT);
        assert_eq!(keys[0].hid, ADDRESSABLE_HID_CODES[0]);
        assert!(keys.iter().all(|key| key.rgb == rgb));
        assert_eq!(encode_direct_write(&keys).unwrap()[2], 68);
    }

    #[test]
    fn clear_is_65_byte_output_report() {
        let packet = encode_clear_direct_write();
        assert_eq!(&packet[..3], &[0, 0x41, 0]);
        assert!(packet[2..].iter().all(|&x| x == 0));
    }

    #[test]
    fn parse_descriptor_with_large_feature_and_short_output() {
        let raw = [
            0x06, 0xc0, 0xff, 0x75, 0x08,
            0x96, 0x00, 0x02, 0xb1, 0x02,
            0x95, 0x40, 0x91, 0x02,
        ];
        let desc = parse_report_descriptor(&raw).unwrap();
        assert!(desc.matches_mini());
        assert_eq!(desc.report_bytes(0, ReportKind::Feature), Some(513));
        assert_eq!(desc.report_bytes(0, ReportKind::Output), Some(65));
    }

    #[test]
    fn parser_rejects_truncated_data() {
        assert!(parse_report_descriptor(&[0x96, 0xff]).is_err());
    }

    #[test]
    fn parse_rgb_and_key_names() {
        assert_eq!(rgb_from_hex("#ff0033").unwrap(), Rgb { red: 255, green: 0, blue: 51 });
        assert_eq!(hid_from_key("A").unwrap(), 4);
        assert_eq!(hid_from_key("space").unwrap(), 44);
        assert!(rgb_from_hex("red").is_err());
        assert!(hid_from_key("F24").is_err());
    }
}
