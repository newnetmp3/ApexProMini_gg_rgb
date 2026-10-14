//! Pure, independently modeled keypress-reactive factory graphics.
//!
//! The preset names and trigger durations below are informed by the user's
//! authorized SteelSeries GG 108.3 migrations, but the exact vendor aperture,
//! propagation and fade curves are NOT known. This module does not claim
//! SteelSeries shader equivalence and never interacts with the keyboard.
use crate::effects::{Config, KeyHit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A circular pulse travels radially from the pressed key.
    Ripple,
    /// A horizontally expanding pulse remains confined to the key row.
    Line,
    /// Only the actual pressed key is lit, followed by a timed fade.
    SingleKey,
}

pub fn kind_for_factory(id: &str) -> Option<Kind> {
    match id {
        "reactiveRipple" => Some(Kind::Ripple),
        "reactiveLine" => Some(Kind::Line),
        "reactiveSingleKey" => Some(Kind::SingleKey),
        _ => None,
    }
}

/// GG supplies the nominal period (250 / 500 / 1250 ms respectively).
/// Speed scales the effect length, but never changes a key's identity or
/// makes it travel backward. Values are bounded by profile validation.
pub fn duration_seconds(period_ms: u32, speed: f32) -> f32 {
    (period_ms as f32 / 1000.0).max(0.05) / speed.clamp(0.1, 5.0)
}

/// Evaluate one event at one normalized physical Mini key coordinate.
///
/// Unlike generic native 'ripple', this uses the *historic factory duration*
/// and isolates three different source-defined trigger families:
/// row-constrained line, circular pulse, and exact pressed-key fade.
///
/// The constants governing wavefront and glow width are independently
/// designed approximations. No vendor shader code is present.
pub fn event_strength(
    kind: Kind,
    hit: KeyHit,
    hid: u8,
    x: f32,
    y: f32,
    now: f32,
    duration: f32,
) -> f32 {
    let age = now - hit.when;
    if !age.is_finite() || age < 0.0 || age >= duration {
        return 0.0;
    }
    let progress = (age / duration).clamp(0.0, 1.0);
    let fade = 1.0 - progress;
    if kind == Kind::SingleKey {
        return if hit.hid == hid { fade } else { 0.0 };
    }
    let Some((origin_x, origin_y)) = crate::effects::position(hit.hid) else {
        return 0.0;
    };
    let dx = (x - origin_x).abs();
    let dy = (y - origin_y).abs();
    // The farthest corners of the normalized board are around sqrt(2) apart.
    // 1.55 reaches them before the GG-derived trigger lifetime ends.
    let front = progress * 1.55;
    let gap = match kind {
        Kind::Line => dx - front,
        Kind::Ripple => (dx * dx + dy * dy).sqrt() - front,
        Kind::SingleKey => unreachable!("single key handled above"),
    };
    // Smooth bounded wave front, deliberately broad enough for a 24 FPS
    // keyboard animation even with the short 250 ms factory ripple duration.
    let wave = (-(gap / 0.19).powi(2)).exp();
    let row_mask = if kind == Kind::Line {
        (-(dy / 0.14).powi(2)).exp()
    } else {
        1.0
    };
    (wave * row_mask * fade).clamp(0.0, 1.0)
}

/// Preserve distinct simultaneous/recent key events without making
/// overlapping pulses overflow channels. This is intentionally an
/// approximation, not GG's original graph blend policy.
pub fn intensity(
    kind: Kind,
    hid: u8,
    x: f32,
    y: f32,
    seconds: f32,
    config: Config,
    period_ms: u32,
    hits: &[KeyHit],
) -> f32 {
    let duration = duration_seconds(period_ms, config.speed);
    hits.iter().map(|&hit| event_strength(kind, hit, hid, x, y, seconds, duration))
        .fold(0.0_f32, f32::max)
}

/// Bounded in-memory event lifetime for animation and overlay modes.
/// This must cover the longest factory fade when speed=0.1 (12.5 seconds
/// for Single Key), not the old universal 2.5s pruning window.
/// Native ripple and other existing modes remain on the original 2.5s.
pub fn retention_seconds(kind: Kind, period_ms: u32, speed: f32) -> f32 {
    let _ = kind; // all three use their recovered nominal periods
    duration_seconds(period_ms, speed).max(2.5).min(20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(kind: Kind, hid: u8, when: f32, now: f32, duration: f32) -> f32 {
        let (x, y) = crate::effects::position(hid).unwrap();
        event_strength(kind, KeyHit { hid: 4, when }, hid, x, y, now, duration)
    }

    #[test]
    fn official_periods_and_speed_scaling_are_observed_as_metadata() {
        assert!((duration_seconds(250, 1.0) - 0.25).abs() < 1e-6);
        assert!((duration_seconds(500, 1.0) - 0.5).abs() < 1e-6);
        assert!((duration_seconds(1250, 0.1) - 12.5).abs() < 1e-6);
        assert_eq!(retention_seconds(Kind::SingleKey, 1250, 0.1), 12.5);
        assert_eq!(retention_seconds(Kind::Ripple, 250, 1.0), 2.5);
    }

    #[test]
    fn single_key_never_bleeds_into_neighbors_and_fades_exactly() {
        let origin = sample(Kind::SingleKey, 4, 0.0, 0.0, 1.25);
        let midway = sample(Kind::SingleKey, 4, 0.0, 0.625, 1.25);
        let expired = sample(Kind::SingleKey, 4, 0.0, 1.25, 1.25);
        let other = sample(Kind::SingleKey, 22, 0.0, 0.1, 1.25);
        assert_eq!(origin, 1.0);
        assert!((midway - 0.5).abs() < 1e-6);
        assert_eq!(expired, 0.0);
        assert_eq!(other, 0.0);
    }

    #[test]
    fn ripple_starts_near_origin_and_propagates_outward() {
        let start = sample(Kind::Ripple, 4, 0.0, 0.0, 0.25);
        let far_early = sample(Kind::Ripple, 39, 0.0, 0.0, 0.25);
        assert!(start > far_early);
        let (x, y) = crate::effects::position(39).unwrap();
        let hit = KeyHit { hid: 4, when: 0.0 };
        let later = event_strength(Kind::Ripple, hit, 39, x, y, 0.16, 0.25);
        assert!(later > far_early);
        assert_eq!(event_strength(Kind::Ripple, hit, 4, x, y, 0.25, 0.25), 0.0);
    }

    #[test]
    fn line_is_row_constrained_and_spreads_both_directions() {
        let hit = KeyHit { hid: 4, when: 0.0 };
        let row = crate::effects::position(22).unwrap(); // S, same row as A
        let other = crate::effects::position(26).unwrap(); // W, other row
        let same_row = event_strength(Kind::Line, hit, 22, row.0, row.1, 0.08, 0.5);
        let other_row = event_strength(Kind::Line, hit, 26, other.0, other.1, 0.08, 0.5);
        assert!(same_row > other_row);
    }

    #[test]
    fn overlapping_hits_choose_brightest_without_overflow() {
        let config = Config { speed: 1.0, ..Config::default() };
        let (x, y) = crate::effects::position(4).unwrap();
        let hit1 = KeyHit { hid: 4, when: 0.0 };
        let hit2 = KeyHit { hid: 22, when: 0.02 };
        let one = intensity(Kind::Ripple, 4, x, y, 0.05, config, 250, &[hit1]);
        let two = intensity(Kind::Ripple, 4, x, y, 0.05, config, 250, &[hit1, hit2]);
        assert!(two >= one);
        assert!((0.0..=1.0).contains(&two));
    }

    #[test]
    fn future_unknown_and_expired_events_are_ignored() {
        let a = crate::effects::position(4).unwrap();
        assert_eq!(event_strength(Kind::Ripple, KeyHit { hid: 4, when: 0.5 },
            4, a.0, a.1, 0.0, 0.25), 0.0);
        assert_eq!(event_strength(Kind::Ripple, KeyHit { hid: 255, when: 0.0 },
            4, a.0, a.1, 0.0, 0.25), 0.0);
        assert_eq!(event_strength(Kind::Ripple, KeyHit { hid: 4, when: 0.0 },
            4, a.0, a.1, 10.0, 0.25), 0.0);
        assert_eq!(event_strength(Kind::Ripple, KeyHit { hid: 4, when: f32::NAN },
            4, a.0, a.1, 0.0, 0.25), 0.0);
    }
}
