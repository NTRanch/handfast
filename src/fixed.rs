//! Fixed-point arithmetic and trigonometry for the simulation.
//!
//! The sim runs on integer fixed point (8 fractional bits; 1 unit =
//! 1/256 px) so a draw list is byte-identical on native and wasm32 builds
//! of the same source. Contract: every helper is total — no panics, no
//! overflow in release or debug (wrapping is confined to `mul`'s i64
//! intermediate, which cannot overflow for canvas-scale values).

use std::sync::OnceLock;

/// Fixed-point value: `value / SCALE` pixels.
pub type Fx = i32;

/// Fractional bits in [`Fx`].
pub const SCALE_BITS: u32 = 8;
/// One pixel in fixed point.
pub const ONE: Fx = 1 << SCALE_BITS;
/// Canvas width and height in pixels (square, per the program contract).
pub const CANVAS_PX: i32 = 960;

/// Convert whole pixels to fixed point.
pub const fn px(value: i32) -> Fx {
    value * ONE
}

/// Convert fixed point back to whole pixels, rounding to nearest.
pub fn to_px(value: Fx) -> i32 {
    (value + ONE / 2) >> SCALE_BITS
}

/// Multiply two fixed-point values. Intermediates are i64; with
/// canvas-scale operands (|v| < 2^20) the product cannot overflow.
pub fn mul(a: Fx, b: Fx) -> Fx {
    ((i64::from(a) * i64::from(b)) >> SCALE_BITS) as i32
}

/// Divide fixed point by fixed point. A zero divisor yields 0: callers
/// treat "no extent" as "no movement" rather than a fault.
pub fn div(a: Fx, b: Fx) -> Fx {
    if b == 0 {
        return 0;
    }
    ((i64::from(a) << SCALE_BITS) / i64::from(b)) as i32
}

/// Linear interpolation; `t_milli` is clamped to 0..=1000.
pub fn lerp(a: Fx, b: Fx, t_milli: i32) -> Fx {
    let t = t_milli.clamp(0, 1000);
    a + ((i64::from(b - a) * i64::from(t)) / 1000) as i32
}

/// Move `current` toward `target` by at most `max_delta` (which the
/// caller passes as a non-negative rate). The expression-ramp workhorse:
/// monotone, never overshoots, never snaps.
pub fn approach(current: i32, target: i32, max_delta: i32) -> i32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + max_delta * delta.signum()
    }
}

/// Euclidean length of a fixed-point vector, in fixed point. Uses i64
/// squares; canvas-scale vectors cannot overflow.
pub fn length(x: Fx, y: Fx) -> Fx {
    let xx = i64::from(x) * i64::from(x);
    let yy = i64::from(y) * i64::from(y);
    // sqrt of the sum scaled back down: sum is in units of (1/256)^2 px^2
    // times 2^16 from the squares of Fx values; isqrt then rescale.
    let sum = (xx + yy) >> SCALE_BITS;
    (isqrt(sum.max(0)) << (SCALE_BITS / 2)) as i32
}

fn isqrt(value: i64) -> i64 {
    if value <= 0 {
        return 0;
    }
    let mut root = (value as f64).sqrt() as i64;
    while (root + 1) * (root + 1) <= value {
        root += 1;
    }
    while root * root > value {
        root -= 1;
    }
    root
}

/// Phase is a full turn in 0..65536. The sine table holds 1024 entries
/// in milli-units (-1000..=1000), built once from `f64::sin`; lookups
/// interpolate linearly so motion is smooth at 60 steps/s.
const SINE_TABLE_LEN: usize = 1024;

fn sine_table() -> &'static [i32; SINE_TABLE_LEN] {
    static TABLE: OnceLock<[i32; SINE_TABLE_LEN]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0i32; SINE_TABLE_LEN];
        for (index, slot) in table.iter_mut().enumerate() {
            let angle = (index as f64) * std::f64::consts::TAU / (SINE_TABLE_LEN as f64);
            *slot = (angle.sin() * 1000.0).round() as i32;
        }
        table
    })
}

/// Sine of a phase (0..65536 per turn), in milli-units (-1000..=1000).
pub fn sin_milli(phase: u32) -> i32 {
    let table = sine_table();
    let scaled = (u64::from(phase & 0xFFFF) * SINE_TABLE_LEN as u64) >> 16;
    let index = (scaled as usize) % SINE_TABLE_LEN;
    let next = (index + 1) % SINE_TABLE_LEN;
    let frac = (((u64::from(phase & 0xFFFF) * SINE_TABLE_LEN as u64) & 0xFFFF) >> 6) as i32;
    let a = table[index];
    let b = table[next];
    a + ((b - a) * frac) / 1024
}

/// Cosine of a phase, in milli-units.
pub fn cos_milli(phase: u32) -> i32 {
    sin_milli(phase.wrapping_add(16384))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_round_trips() {
        assert_eq!(to_px(px(960)), 960);
        assert_eq!(to_px(px(-12)), -12);
    }

    #[test]
    fn mul_div_are_inverse_on_canvas_values() {
        let a = px(123);
        let b = px(7);
        assert_eq!(div(mul(a, b), b), a);
    }

    #[test]
    fn div_by_zero_is_zero_not_panic() {
        assert_eq!(div(px(5), 0), 0);
    }

    #[test]
    fn approach_never_overshoots() {
        assert_eq!(approach(0, 100, 30), 30);
        assert_eq!(approach(90, 100, 30), 100);
        assert_eq!(approach(100, 0, 30), 70);
        assert_eq!(approach(100, 100, 30), 100);
    }

    #[test]
    fn sine_cardinal_points() {
        assert_eq!(sin_milli(0), 0);
        assert!(sin_milli(16384) > 990);
        assert!(sin_milli(32768).abs() < 10);
        assert!(sin_milli(49152) < -990);
    }

    #[test]
    fn cos_is_quarter_turn_offset() {
        assert!((cos_milli(0) - 1000).abs() < 10);
        assert!(cos_milli(16384).abs() < 10);
    }

    #[test]
    fn length_of_unit_diagonal() {
        let len = length(px(3), px(4));
        assert!((to_px(len) - 5).abs() <= 1, "len={len}");
    }
}
