//! Width-aware, provisional 60% keyboard geometry for factory-style Prism
//! presets. This is an independent approximation based on conventional
//! ANSI-style key-unit widths and the known 68 HID usages of wired Mini.
//!
//! Unlike a uniformly spaced grid, long keys have physical centers: Space
//! is near the bottom-row center; Shift, Backspace and Enter use wider spans.
//! The real vendor's exact LED coordinates / ANSI/ISO choices still need
//! visual validation on owner hardware. The *owner-confirmed native ripple*
//! intentionally continues to use effects::position without changing it.
use crate::ADDRESSABLE_HID_CODES;

/// Position of an addressable RGB usage, in the normalized five-row board.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyPosition {
    pub x: f32,
    pub y: f32,
    /// 0-based five-row index (GG historical logical row = row + 1).
    pub row: u8,
    /// Key's approximate physical width in 1u keyboard units.
    pub width_units: f32,
}

const BOARD_WIDTH_UNITS: f32 = 15.0;
const TOP: &[(u8, f32)] = &[
    (41,1.0), (30,1.0), (31,1.0), (32,1.0), (33,1.0),
    (34,1.0), (35,1.0), (36,1.0), (37,1.0), (38,1.0),
    (39,1.0), (45,1.0), (46,1.0), (42,2.0),
];
const QWERTY: &[(u8, f32)] = &[
    (43,1.5), (20,1.0), (26,1.0), (8,1.0), (21,1.0),
    (23,1.0), (28,1.0), (24,1.0), (12,1.0), (18,1.0),
    (19,1.0), (47,1.0), (48,1.0), (49,1.5),
];
const HOME: &[(u8, f32)] = &[
    (57,1.75), (4,1.0), (22,1.0), (7,1.0), (9,1.0),
    (10,1.0), (11,1.0), (13,1.0), (14,1.0), (15,1.0),
    (51,1.0), (52,1.0), (40,2.25),
];
const SHIFT: &[(u8, f32)] = &[
    (225,2.25), (29,1.0), (27,1.0), (6,1.0), (25,1.0),
    (5,1.0), (17,1.0), (16,1.0), (54,1.0), (55,1.0),
    (56,1.0), (229,2.75),
];
// Bottom row includes optional/nonstandard layout usage IDs. Their centers
// are not manufacturer-verified and should be calibrated on-device.
const BOTTOM: &[(u8, f32)] = &[
    (224,1.25), (227,1.25), (226,1.25), (139,0.75),
    (44,5.5), (138,0.75), (136,0.75),
    (230,1.0), (231,1.0), (240,0.75), (228,0.75),
];

const ROWS: [&[(u8, f32)]; 5] = [TOP, QWERTY, HOME, SHIFT, BOTTOM];

fn center_of(hid: u8) -> Option<KeyPosition> {
    for (row, keys) in ROWS.iter().enumerate() {
        let mut offset = 0.0_f32;
        for &(key, width) in *keys {
            if hid == key {
                return Some(KeyPosition {
                    x: (offset + width * 0.5) / BOARD_WIDTH_UNITS,
                    y: row as f32 / 4.0,
                    row: row as u8,
                    width_units: width,
                });
            }
            offset += width;
        }
    }
    None
}

/// Four additional usages describe regional or firmware variants not present
/// simultaneously on conventional ANSI physical rows. Keep them addressable
/// without inventing new keys outside the 15u case.
pub fn position(hid: u8) -> Option<KeyPosition> {
    if let Some(primary) = center_of(hid) {
        return Some(primary);
    }
    match hid {
        // Alternate row-0 firmware slot near the wide Backspace.
        137 => center_of(42),
        // Non-US row-2 symbol key: near the Enter aperture. Provisional.
        50 => Some(KeyPosition {
            x: 13.25 / BOARD_WIDTH_UNITS, y: 0.5, row: 2, width_units: 1.0,
        }),
        // ISO additional key adjacent to shortened left Shift.
        100 => Some(KeyPosition {
            x: 1.75 / BOARD_WIDTH_UNITS, y: 0.75, row: 3, width_units: 1.0,
        }),
        // Alternate row-3 firmware slot near the wide right Shift.
        135 => center_of(229),
        _ => None,
    }
}

/// Map a specific Mini usage onto the historical 6-row by 22-column
/// Prism source grid. Five physical rows use source rows 1..5.
/// The column is an approximate 0..21 geometric lookup, not the true GG
/// coordinate definition. Sparse-region masks can now use *per-key* lookup.
pub fn source_grid(hid: u8) -> Option<(u8, u8)> {
    let pos = position(hid)?;
    let column = (pos.x * 21.0).round().clamp(0.0, 21.0) as u8;
    Some((pos.row + 1, column))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_known_hid_usages_are_unique_and_located() {
        let mut mapped = HashSet::new();
        for row in ROWS {
            let mut units = 0.0_f32;
            for &(hid, width) in row {
                assert!(mapped.insert(hid), "duplicate HID {hid}");
                assert!(width > 0.0);
                units += width;
            }
            assert!((units - 15.0).abs() < 1e-5, "row width was {units}");
        }
        for alt in [137, 50, 100, 135] {
            assert!(mapped.insert(alt));
        }
        let source = ADDRESSABLE_HID_CODES.iter().copied().collect::<HashSet<_>>();
        assert_eq!(mapped, source);
        for hid in source {
            let pos = position(hid).expect("missing 68-slot factory geometry");
            assert!(pos.x >= 0.0 && pos.x <= 1.0);
            assert!(pos.y >= 0.0 && pos.y <= 1.0);
            assert!(pos.width_units > 0.0);
            let (source_row, col) = source_grid(hid).unwrap();
            assert_eq!(source_row, pos.row + 1);
            assert!(source_row >= 1 && source_row <= 5 && col <= 21);
        }
        assert!(position(0).is_none());
    }

    #[test]
    fn wide_key_centers_are_not_equispaced() {
        let space = position(44).unwrap();
        let left_shift = position(225).unwrap();
        let right_shift = position(229).unwrap();
        let backspace = position(42).unwrap();
        let enter = position(40).unwrap();
        assert!((0.4..=0.6).contains(&space.x), "space should be centered");
        assert!(space.width_units >= 5.0);
        assert!(left_shift.width_units > 2.0);
        assert!(right_shift.width_units > 2.0);
        assert_eq!(backspace.width_units, 2.0);
        assert!(enter.width_units >= 2.0);
    }

    #[test]
    fn rows_and_neighbors_match_basic_hid_layout() {
        let a = position(4).unwrap();
        let s = position(22).unwrap();
        let w = position(26).unwrap();
        assert_eq!(a.row, s.row);
        assert_ne!(a.row, w.row);
        assert!(a.x < s.x);
        let z = position(29).unwrap();
        let x = position(27).unwrap();
        assert_eq!(z.row, x.row);
        assert!(z.x < x.x);
        assert_eq!(source_grid(4).unwrap().0, 3);
        assert_eq!(source_grid(44).unwrap().0, 5);
    }

    #[test]
    fn iso_alternates_remain_inside_keyboard() {
        assert_eq!(position(137), position(42));
        assert_eq!(position(135), position(229));
        assert_eq!(position(50).unwrap().row, 2);
        assert_eq!(position(100).unwrap().row, 3);
    }
}
