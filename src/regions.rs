//! Representative GG multi-region palette masks, summarized independently
//! from owner-supplied GG 108.3 Prism migration files.
//!
//! GG addresses a six-row logical grid with x coordinates ranging 0..21.
//! Mini is five physical rows, so current mapping aligns physical rows with
//! GG 1..5 (top row 0 is an absent function row). Column matching is a
//! nearest-grid approximation requiring on-device validation.
//!
//! These are functional parameters only, NOT vendor source JSON/SQL.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub name: &'static str,
    pub source_rows: u8,
    pub columns: &'static [u8],
    pub period_ms: u32,
    pub wavelength: i16,
    pub geometry: u8,
    pub stops: &'static [(u8, u32)],
}

macro_rules! region {
    ($id:literal, $mask:expr, [$($column:literal),*], $time:expr, $wave:expr, $shape:expr, [$($at:literal => $color:literal),* $(,)?]) => {
        Region {
            name: $id,
            source_rows: $mask,
            columns: &[$($column),*],
            period_ms: $time,
            wavelength: $wave,
            geometry: $shape,
            stops: &[$(($at,$color)),*],
        }
    };
}
const fn row(id: u8) -> u8 { 1 << id }
const fn rows(a: u8,b: u8) -> u8 { row(a) | row(b) }
const fn range(a:u8,b:u8,c:u8) -> u8 { row(a) | row(b) | row(c) }

/// Additional region-specific historical source variants. The unmasked base
/// remains in factory::PRESETS; this table maps the alternate masked layouts.
pub static VARIANTS: &[Region] = &[
    // 6-row rainbows: y0 is not present on this wired Mini's 5-row layout.
    region!("rainbow",row(1),[],6120,89,0,[0=>0xff00e1,30=>0x000000,61=>0xff0400]),
    region!("rainbow",row(2),[],6120,89,0,[0=>0xff0000,30=>0x000000,61=>0xffea00]),
    region!("rainbow",row(3),[],6120,89,0,[0=>0xffea00,30=>0x000000,61=>0x26ff00]),
    region!("rainbow",row(4),[],6120,89,0,[0=>0x1eff00,30=>0x000000,61=>0x001eff]),
    region!("rainbow",row(5),[],6120,89,0,[0=>0x0000ff,30=>0x000000,61=>0x7b00ff]),

    // The source also has a y0 variation; retain it for future layouts.
    region!("rainbow",row(0),[],6120,89,0,[0=>0x9500ff,30=>0x000000,61=>0xff00ff]),

    region!("colorFusion",range(0,1,2),[],1679,64,0,[3=>0xff0000,105=>0xff00e6,218=>0xffea00]),
    region!("colorFusion",range(3,4,5),[],1679,-64,0,[0=>0xff0000,127=>0xea00ff,220=>0xffea00]),

    region!("aqua",row(1),[],4000,-64,0,[0=>0x009dff,51=>0x0026ff,89=>0x005e80,122=>0x00bbff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),
    region!("aqua",row(2),[],4000,-64,1,[0=>0x009dff,63=>0x0026ff,76=>0x000000,102=>0x000000,160=>0x00bbff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),
    region!("aqua",row(3),[],4000,-64,1,[0=>0x009dff,63=>0x0026ff,76=>0x000000,102=>0x000000,160=>0x00bbff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),
    region!("aqua",row(4),[],4000,-64,1,[0=>0x009dff,63=>0x0026ff,76=>0x000000,102=>0x000000,160=>0x00bbff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),
    region!("aqua",row(5),[],4000,-64,1,[0=>0x009dff,63=>0x0026ff,76=>0x000000,102=>0x000000,160=>0x00bbff,183=>0x00b1f2,201=>0x000000,249=>0x000000]),

    region!("freeway",row(0),[],2000,64,0,[80=>0x000000,125=>0xff0000,170=>0x000000]),
    region!("freeway",row(1),[],2000,-64,0,[80=>0x000000,125=>0xffff00,170=>0x000000]),
    region!("freeway",row(2),[],2000,64,0,[80=>0x000000,125=>0x00ff00,170=>0x000000]),
    region!("freeway",row(3),[],2000,-64,0,[80=>0x000000,125=>0x00ffff,170=>0x000000]),
    region!("freeway",row(4),[],1679,64,0,[80=>0x000000,125=>0x0000ff,170=>0x000000]),
    region!("freeway",row(5),[],1679,-64,0,[80=>0x000000,125=>0xb700ff,170=>0x000000]),

    region!("comet",row(1),[],1679,-64,0,[190=>0x000000,210=>0xe890fc,230=>0xffffff,255=>0x000000]),
    // Vendor variant 2 lists a stop out of order; normalize for interpolation.
    region!("comet",row(2),[],1679,64,0,[150=>0x000000,164=>0x000000,199=>0x00ffe9,230=>0xffffff]),
    region!("comet",row(3),[],1679,64,0,[20=>0x000000,40=>0x909bff,80=>0xffffff,120=>0x000000]),
    region!("comet",row(4),[],1679,64,0,[80=>0x000000,110=>0xffa81f,130=>0xffd796,170=>0x000000]),
    region!("comet",row(5),[],1679,-64,0,[130=>0x000000,155=>0x96ffd7,190=>0x9dd75a,207=>0x000000]),

    region!("pride",row(5),[],4690,64,0,[0=>0x7f00ff,77=>0xff0000,170=>0x7f00ff]),
    region!("pride",row(4),[],4690,64,0,[0=>0x0000ff,83=>0x7f00ff,168=>0x0000ff]),
    region!("pride",row(3),[],4690,64,0,[14=>0x00ff00,84=>0x0000ff,181=>0x00ff00]),
    region!("pride",row(2),[],4690,64,0,[0=>0xffff00,96=>0x00ff00,177=>0xffff00]),
    region!("pride",row(0),[],4690,64,0,[0=>0xff0000,93=>0xff6600,181=>0xff0000]),
    region!("pride",row(1),[],4690,64,0,[0=>0xff6600,85=>0xffea00,172=>0xff6600]),

    region!("twilight",range(0,1,2),[],4690,64,0,[0=>0xd60071,87=>0x9b59b6,166=>0xd60071]),
    region!("twilight",row(3),[],4690,64,0,[0=>0x9b59b6,80=>0x0038a8,171=>0x9b59b6]),
    region!("twilight",rows(4,5),[],4690,64,0,[0=>0x0038a8,79=>0xd60071,172=>0x0038a8]),

    // Wabash and Lake: sparse column masks extending through rows 0..5.
    region!("wabashAndLake",0x3f,[1,6,8,10,13,16],2100,-16,3,[0=>0x326400,40=>0x081000,51=>0x4e9b00,102=>0x80ff00,127=>0x000000,153=>0x408000,204=>0x80ff00,242=>0x0b1600]),
    region!("wabashAndLake",0x3f,[2,7,11,14,17],2100,-19,3,[0=>0x050a00,51=>0x081000,64=>0x50a000,77=>0x6fde00,102=>0x1c3700,153=>0x408000,204=>0x64c800,242=>0x000000]),
    region!("wabashAndLake",0x3f,[3,5,9,12,19],2100,-6,3,[0=>0x000000,51=>0x081000,64=>0x50a000,77=>0x285000,102=>0x6edc00,153=>0x408000,204=>0x000000,242=>0x80ff00]),
];

/// GG uses six logical rows; our wired Mini 5-row keyboard maps to y1..5.
/// Exact physical spacing and mask alignment require visual/hardware QA.
pub fn approx_source_row(y: f32) -> u8 {
    ((y.clamp(0.0, 1.0) * 4.0).round() as u8 + 1).min(5)
}

pub fn select(id: &str, x: f32, y: f32) -> Option<&'static Region> {
    let source_y = approx_source_row(y);
    let source_x = (x.clamp(0.0, 1.0) * 21.0).round() as u8;
    VARIANTS.iter().find(|v| {
        v.name == id
            && v.source_rows & row(source_y) != 0
            && (v.columns.is_empty() || v.columns.contains(&source_x))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_region_presets_have_additional_variants() {
        for name in ["rainbow","colorFusion","aqua","freeway","comet","pride","twilight","wabashAndLake"] {
            assert!(VARIANTS.iter().any(|v| v.name == name), "{name}");
        }
        assert_eq!(VARIANTS.len(), 36);
    }
    #[test]
    fn row_mapping_is_bounded() {
        assert_eq!(approx_source_row(0.0), 1);
        assert_eq!(approx_source_row(1.0), 5);
        assert_eq!(approx_source_row(-1.0), 1);
        assert_eq!(approx_source_row(2.0), 5);
    }
    #[test]
    fn region_variants_select_different_palette_rows() {
        let upper = select("pride",0.25,0.0).unwrap();
        let lower = select("pride",0.25,1.0).unwrap();
        assert_ne!(upper.stops, lower.stops);
        assert!(select("prism",0.2,0.1).is_none());
    }
    #[test]
    fn sparse_columns_restrict_wabash_layouts() {
        let selected = select("wabashAndLake",6.0/21.0,0.25).unwrap();
        assert_eq!(selected.wavelength,-16);
        assert!(selected.columns.contains(&6));
        assert!(select("wabashAndLake",0.0,0.25).is_none());
    }
}
