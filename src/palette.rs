//! The game palette. Ops carry indices into this table; the shell, the
//! capture rasteriser, and the tests all read the same table from the
//! core, so web and video can never drift apart.
//!
//! Colours: NTRanch dark ground + hot pink family, with the reference
//! recipe's measured families (plum shadow, magenta mid, rose skin).

/// Palette index of the background / ground colour.
pub const BG: u8 = 0;
/// Panel / card fill.
pub const PANEL: u8 = 1;
/// Deep plum shadow (reference family `#540e46`).
pub const PLUM: u8 = 2;
/// Magenta mid (reference family `#b31348`).
pub const MAGENTA: u8 = 3;
/// Hot pink, the NTRanch accent (`#ff2e88`).
pub const PINK: u8 = 4;
/// Rose skin highlight (reference family `#ec8584`).
pub const ROSE: u8 = 5;
/// Dimmer rose (reference family `#da6869`).
pub const ROSE_DIM: u8 = 6;
/// Cream — text and sclera.
pub const CREAM: u8 = 7;
/// Dim violet — secondary linework.
pub const VIOLET_DIM: u8 = 8;
/// Appraisal gold — the Brush's colour.
pub const GOLD: u8 = 9;
/// Pale glow — the emissive-iris layer and peak highlights.
pub const GLOW: u8 = 10;
/// Near-black outline.
pub const OUTLINE: u8 = 11;
/// Violet — Sloane's presiding-style accent.
pub const VIOLET: u8 = 12;

/// The palette as `#rrggbb` strings, in index order.
pub const HEX: [&str; 13] = [
    "#0d0208", "#1c0714", "#540e46", "#b31348", "#ff2e88", "#ec8584", "#da6869", "#f7e8f1",
    "#8a5a7d", "#ffc24d", "#ffd7ec", "#2a0a1e", "#9b5de5",
];

/// The palette serialised as a JSON array (shell bootstrap).
pub fn json() -> String {
    let mut out = String::from("[");
    for (index, hex) in HEX.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(hex);
        out.push('"');
    }
    out.push(']');
    out
}
