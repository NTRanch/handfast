//! The per-frame draw list: the core's only rendering output.
//!
//! Ops are serialised as text lines into a bounded buffer the shell (or
//! the capture harness) parses: `r x y w h c`, `c x y r c`,
//! `l x1 y1 x2 y2 w c`, `p x1 y1 x2 y2 x3 y3 c`, `t x y size c TEXT`.
//! Coordinates are whole pixels on the 960x960 logical canvas; `c` is a
//! palette index. The op count is capped ([`MAX_OPS`]); past the cap,
//! ops are dropped and `overflowed` is set — a frame degrades, it never
//! corrupts the buffer or panics.

use std::fmt::Write as _;

/// Maximum ops per frame. The richest scene (two figures + UI) uses
/// well under 300; the cap is headroom, not a budget to spend.
pub const MAX_OPS: usize = 1024;

/// Maximum characters in one text op.
pub const MAX_TEXT_CHARS: usize = 48;

/// One frame's draw list.
pub struct DrawList {
    buffer: String,
    ops: usize,
    /// Set when ops were dropped at the cap (adversarial-test surface).
    pub overflowed: bool,
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawList {
    /// An empty draw list.
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(16 * 1024),
            ops: 0,
            overflowed: false,
        }
    }

    /// Reset for the next frame.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.ops = 0;
        self.overflowed = false;
    }

    /// Ops emitted so far this frame.
    pub fn op_count(&self) -> usize {
        self.ops
    }

    /// The serialised frame.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// FNV-1a hash of the serialised frame (determinism tests).
    pub fn hash(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in self.buffer.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    fn room(&mut self) -> bool {
        if self.ops >= MAX_OPS {
            self.overflowed = true;
            return false;
        }
        self.ops += 1;
        true
    }

    /// Filled rectangle.
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if self.room() {
            let _ = writeln!(self.buffer, "r {x} {y} {w} {h} {color}");
        }
    }

    /// Filled circle.
    pub fn circle(&mut self, x: i32, y: i32, r: i32, color: u8) {
        if self.room() {
            let _ = writeln!(self.buffer, "c {x} {y} {r} {color}");
        }
    }

    /// Thick line segment.
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, w: i32, color: u8) {
        if self.room() {
            let _ = writeln!(self.buffer, "l {x1} {y1} {x2} {y2} {w} {color}");
        }
    }

    /// Filled triangle. The argument list mirrors the wire op
    /// (`p x1 y1 x2 y2 x3 y3 c`) — a protocol shape, kept flat.
    #[allow(clippy::too_many_arguments)]
    pub fn tri(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: u8) {
        if self.room() {
            let _ = writeln!(self.buffer, "p {x1} {y1} {x2} {y2} {x3} {y3} {color}");
        }
    }

    /// Bitmap text. The input is uppercased and stripped to the raster
    /// font's set (A-Z 0-9 and `.,!?:'-/+()% `); anything else becomes
    /// `?`. Length is capped at [`MAX_TEXT_CHARS`].
    pub fn text(&mut self, x: i32, y: i32, size: i32, color: u8, text: &str) {
        if !self.room() {
            return;
        }
        let mut clean = String::with_capacity(MAX_TEXT_CHARS);
        for ch in text.chars().take(MAX_TEXT_CHARS) {
            let up = if ch.is_whitespace() {
                ' '
            } else {
                ch.to_ascii_uppercase()
            };
            if up.is_ascii_alphanumeric() || " .,!?:'-/+()%".contains(up) {
                clean.push(up);
            } else {
                clean.push('?');
            }
        }
        let _ = writeln!(self.buffer, "t {x} {y} {size} {color} {clean}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops_serialise_in_order() {
        let mut list = DrawList::new();
        list.rect(1, 2, 3, 4, 5);
        list.circle(10, 20, 6, 7);
        assert_eq!(list.as_str(), "r 1 2 3 4 5\nc 10 20 6 7\n");
        assert_eq!(list.op_count(), 2);
    }

    #[test]
    fn text_is_sanitised_and_capped() {
        let mut list = DrawList::new();
        list.text(
            0,
            0,
            14,
            1,
            "héllo\nworld this is a very long string beyond cap!!",
        );
        let line = list.as_str();
        assert!(line.starts_with("t 0 0 14 1 H?LLO WORLD"));
        assert!(!line.contains('\n') || line.matches('\n').count() == 1);
    }

    #[test]
    fn cap_drops_ops_and_flags_overflow() {
        let mut list = DrawList::new();
        for _ in 0..(MAX_OPS + 50) {
            list.circle(0, 0, 1, 0);
        }
        assert_eq!(list.op_count(), MAX_OPS);
        assert!(list.overflowed);
    }
}
