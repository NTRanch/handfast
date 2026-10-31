//! One staged figure: breathing layer, dual gaze channel, continuous
//! expression state, and lagging hair strands.
//!
//! Laws carried here (from the fathom recipe, docs/RECIPE-5447477.md):
//! - Breathing is a perpetual additive layer: it advances in every
//!   state, is never gated, and adds to chest/head on top of all else.
//! - Expression values are continuous (milli-units) and only ever move
//!   via bounded ramps — no pose snapping.
//! - Secondary motion (hair strands) is spring-lagged behind the head;
//!   the spring settles in ~1 step at 60 steps/s (the <=17 ms budget).
//! - The glowing iris is a switchable layer flag, never a property of
//!   the figure: the game turns it on/off per scene and per setting.

use crate::draw::DrawList;
use crate::fixed::{self, Fx, approach, px, sin_milli, to_px};
use crate::palette as pal;
use crate::rng::Rng;

/// Head centre height above the ground origin, px.
const HEAD_Y: i32 = -400;
/// Head radius, px.
const HEAD_R: i32 = 62;
/// Strand count per figure.
const STRANDS: usize = 3;

/// Per-figure colour assignment.
#[derive(Clone, Copy)]
pub struct Style {
    /// Hair fill.
    pub hair: u8,
    /// Skin fill.
    pub skin: u8,
    /// Cloth fill.
    pub cloth: u8,
    /// Accent (style-of-presiding) colour.
    pub accent: u8,
}

/// Expression state: current values and ramp targets, all milli-units.
#[derive(Clone, Copy, Default)]
pub struct Expression {
    /// Lid openness 0 (closed) .. 1000 (open).
    pub lid: i32,
    /// Mouth openness 0..1000.
    pub mouth: i32,
    /// Blush intensity 0..1000.
    pub blush: i32,
    /// Brow raise -1000..1000.
    pub brow: i32,
    /// Lid target.
    pub lid_t: i32,
    /// Mouth target.
    pub mouth_t: i32,
    /// Blush target.
    pub blush_t: i32,
    /// Brow target.
    pub brow_t: i32,
}

/// One hair strand: a spring tip chasing a scalp anchor.
#[derive(Clone, Copy)]
struct Strand {
    tip_x: Fx,
    tip_y: Fx,
    vel_x: Fx,
    vel_y: Fx,
    anchor_dx: i32,
    len_px: i32,
}

/// A staged figure. All positions are fixed point; the origin is the
/// ground point under the figure.
pub struct Figure {
    /// Ground origin, fixed point.
    pub origin_x: Fx,
    /// Ground origin y, fixed point.
    pub origin_y: Fx,
    /// +1 faces right, -1 faces left.
    pub facing: i32,
    /// Colour assignment.
    pub style: Style,
    /// Expression amplitude multiplier, milli (the receiving figure
    /// carries the higher activity — recipe §5).
    pub activity: i32,
    /// Iris glow layer on/off (switchable; game/settings own this).
    pub glow: bool,
    /// Calm mode scales secondary motion down (accessibility).
    pub calm: bool,
    /// Current gaze, milli -1000..=1000 per axis.
    pub gaze_x: i32,
    /// Current gaze y, milli.
    pub gaze_y: i32,
    /// Expression state.
    pub expr: Expression,
    breath_phase: u32,
    breath_rate: u32,
    sway_phase: u32,
    blink_in: u32,
    blinking: u32,
    gaze_tx: i32,
    gaze_ty: i32,
    strands: [Strand; STRANDS],
}

impl Figure {
    /// Create a figure at a ground position. `breath_rate` is the phase
    /// increment per step (65536 = one breath cycle per N steps).
    pub fn new(
        origin_x_px: i32,
        origin_y_px: i32,
        facing: i32,
        style: Style,
        breath_rate: u32,
    ) -> Self {
        // Side locks only: no strand crosses the face.
        let anchors = [-56, -47, 54];
        let lens = [138, 118, 130];
        let mut strands = [Strand {
            tip_x: 0,
            tip_y: 0,
            vel_x: 0,
            vel_y: 0,
            anchor_dx: 0,
            len_px: 0,
        }; STRANDS];
        for (i, strand) in strands.iter_mut().enumerate() {
            strand.anchor_dx = anchors[i];
            strand.len_px = lens[i];
            strand.tip_x = px(origin_x_px + anchors[i]);
            strand.tip_y = px(origin_y_px + HEAD_Y + 10 + lens[i]);
        }
        Self {
            origin_x: px(origin_x_px),
            origin_y: px(origin_y_px),
            facing,
            style,
            activity: 1000,
            glow: false,
            calm: false,
            gaze_x: 0,
            gaze_y: 0,
            expr: Expression {
                lid: 850,
                lid_t: 850,
                ..Expression::default()
            },
            breath_phase: 0,
            breath_rate,
            sway_phase: 0,
            blink_in: 200,
            blinking: 0,
            gaze_tx: 0,
            gaze_ty: 0,
            strands,
        }
    }

    /// Current breathing value, 0..1000. Public for gates/tests: the
    /// layer must be visibly alive in every state.
    pub fn breath_milli(&self) -> i32 {
        (sin_milli(self.breath_phase) + 1000) / 2
    }

    /// World position of the head centre, px.
    pub fn head_px(&self) -> (i32, i32) {
        let breath = self.breath_milli();
        let sway = self.sway_px();
        (
            to_px(self.origin_x) + sway,
            to_px(self.origin_y) + HEAD_Y - breath * 3 / 1000,
        )
    }

    /// World position of an eye centre, px (`side` -1 left, +1 right).
    pub fn eye_px(&self, side: i32) -> (i32, i32) {
        let (hx, hy) = self.head_px();
        (hx + side * 24, hy - 8)
    }

    /// Horizontal body sway offset, px (secondary motion).
    pub fn sway_px(&self) -> i32 {
        let amp = if self.calm { 2 } else { 5 };
        sin_milli(self.sway_phase) * amp / 1000
    }

    /// Aim the gaze channel at a world point (px). The channel itself
    /// ramps in `step`; this only sets the target.
    pub fn look_at(&mut self, world_x: i32, world_y: i32) {
        let (ex, ey) = self.eye_px(0);
        let dx = px(world_x - ex);
        let dy = px(world_y - ey);
        let len = fixed::length(dx, dy);
        if len > px(4) {
            self.gaze_tx = (fixed::div(dx, len) * 1000 / fixed::ONE).clamp(-1000, 1000);
            self.gaze_ty = (fixed::div(dy, len) * 1000 / fixed::ONE).clamp(-1000, 1000);
        }
    }

    /// Advance one fixed step: breathing always, gaze ramp, expression
    /// ramps, blink, hair springs.
    pub fn step(&mut self, rng: &mut Rng) {
        self.breath_phase = self.breath_phase.wrapping_add(self.breath_rate);
        self.sway_phase = self.sway_phase.wrapping_add(self.breath_rate / 3 + 7);
        self.gaze_x = approach(self.gaze_x, self.gaze_tx, 30);
        self.gaze_y = approach(self.gaze_y, self.gaze_ty, 30);
        let e = &mut self.expr;
        e.lid = approach(e.lid, e.lid_t, 14);
        e.mouth = approach(e.mouth, e.mouth_t, 9);
        e.blush = approach(e.blush, e.blush_t, 5);
        e.brow = approach(e.brow, e.brow_t, 11);
        if self.blink_in == 0 {
            self.blinking = 9;
            self.blink_in = 170 + rng.below(160) as u32;
        } else {
            self.blink_in -= 1;
        }
        if self.blinking > 0 {
            self.blinking -= 1;
        }
        self.step_strands();
    }

    fn step_strands(&mut self) {
        let (hx, hy) = self.head_px();
        for strand in &mut self.strands {
            let target_x = px(hx + strand.anchor_dx);
            let target_y = px(hy + 10 + strand.len_px);
            // Spring with stiffness 1/6 and damping: the tip trails the
            // scalp by design; the lag stays inside one step at speed.
            strand.vel_x += (target_x - strand.tip_x) / 6;
            strand.vel_y += (target_y - strand.tip_y) / 6;
            strand.vel_x = strand.vel_x * 3 / 4;
            strand.vel_y = strand.vel_y * 3 / 4;
            strand.tip_x += strand.vel_x;
            strand.tip_y += strand.vel_y;
        }
    }

    /// Effective lid openness after the blink override, 0..1000.
    fn lid_effective(&self) -> i32 {
        if self.blinking > 0 {
            let depth = (if self.blinking > 4 {
                9 - self.blinking
            } else {
                self.blinking
            }) as i32;
            (self.expr.lid * depth / 5).min(self.expr.lid)
        } else {
            self.expr.lid
        }
    }

    /// Draw the figure (not the guided arm — the game owns that).
    pub fn draw(&self, list: &mut DrawList) {
        let ox = to_px(self.origin_x) + self.sway_px();
        let oy = to_px(self.origin_y);
        let breath = self.breath_milli();
        let s = self.style;
        // Hair mass behind the head, then the body, then the head.
        list.circle(ox + self.facing * 4, oy + HEAD_Y - 10, HEAD_R + 10, s.hair);
        let chest_r = 76 + breath * 4 / 1000;
        list.circle(ox, oy - 130, 74, s.cloth);
        list.tri(ox - 92, oy - 150, ox + 92, oy - 150, ox, oy + 4, s.cloth);
        list.circle(ox, oy - 208, 62, s.skin);
        list.circle(ox, oy - 292, chest_r, s.skin);
        list.tri(ox - 52, oy - 336, ox + 52, oy - 336, ox, oy - 258, s.cloth);
        // Resting far arm: shoulder to a hand at the hip.
        let sh_x = ox - self.facing * 62;
        let sh_y = oy - 300;
        list.line(sh_x, sh_y, sh_x - self.facing * 18, oy - 190, 13, s.skin);
        list.line(
            sh_x - self.facing * 18,
            oy - 190,
            ox - self.facing * 30,
            oy - 128,
            13,
            s.skin,
        );
        list.circle(ox - self.facing * 30, oy - 124, 11, s.skin);
        // Head and face.
        let (hx, hy) = self.head_px();
        list.circle(hx, hy, HEAD_R, s.skin);
        list.circle(hx + self.facing * 8, hy - HEAD_R + 8, HEAD_R - 14, s.hair);
        self.draw_strands(list);
        self.draw_face(list, hx, hy);
    }

    fn draw_strands(&self, list: &mut DrawList) {
        let (hx, hy) = self.head_px();
        for strand in &self.strands {
            let sx = hx + strand.anchor_dx;
            let sy = hy + 10;
            list.line(
                sx,
                sy,
                to_px(strand.tip_x),
                to_px(strand.tip_y),
                7,
                self.style.hair,
            );
        }
    }

    fn draw_face(&self, list: &mut DrawList, hx: i32, hy: i32) {
        let lid = self.lid_effective();
        for side in [-1, 1] {
            let (ex, ey) = (hx + side * 24, hy - 8);
            // Sclera, then the gaze-offset iris; glow layer behind iris.
            list.circle(ex, ey, 13, pal::CREAM);
            let ix = ex + self.gaze_x * 5 / 1000;
            let iy = ey + self.gaze_y * 4 / 1000;
            if self.glow {
                list.circle(ix, iy, 12, pal::PLUM);
                list.circle(ix, iy, 10, pal::GLOW);
            }
            list.circle(ix, iy, 7, self.style.accent);
            list.circle(ix, iy, 3, pal::OUTLINE);
            // Lid: a skin rect closing from the top as lid -> 0.
            let closed = (1000 - lid) * 26 / 1000;
            if closed > 0 {
                list.rect(ex - 14, ey - 14, 28, closed, self.style.skin);
            }
            // Brow: a short line whose height follows the brow channel.
            let brow_y = ey - 24 - self.expr.brow * 5 / 1000;
            list.line(ex - 11, brow_y + 3, ex + 11, brow_y, 3, pal::OUTLINE);
        }
        // Blush: size stands in for intensity (the op set has no alpha).
        if self.expr.blush > 300 {
            let r = 6 + (self.expr.blush - 300) * 8 / 700;
            list.circle(hx - 38, hy + 14, r, pal::ROSE_DIM);
            list.circle(hx + 38, hy + 14, r, pal::ROSE_DIM);
        }
        // Mouth: a shallow V whose depth is the openness channel.
        let my = hy + 30;
        let depth = self.expr.mouth * 9 / 1000;
        list.line(hx - 10, my, hx, my + depth, 3, pal::OUTLINE);
        list.line(hx, my + depth, hx + 10, my, 3, pal::OUTLINE);
        if self.expr.mouth > 720 {
            list.circle(hx, my + 5, 5, pal::MAGENTA);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> Style {
        Style {
            hair: pal::PLUM,
            skin: pal::ROSE,
            cloth: pal::MAGENTA,
            accent: pal::PINK,
        }
    }

    #[test]
    fn breathing_never_parks() {
        let mut fig = Figure::new(300, 700, 1, style(), 273);
        let mut rng = Rng::new(1);
        let mut seen_min = i32::MAX;
        let mut seen_max = i32::MIN;
        for _ in 0..600 {
            fig.step(&mut rng);
            seen_min = seen_min.min(fig.breath_milli());
            seen_max = seen_max.max(fig.breath_milli());
        }
        assert!(
            seen_min < 200 && seen_max > 800,
            "breath range {seen_min}..{seen_max}"
        );
    }

    #[test]
    fn expression_ramps_are_monotone() {
        let mut fig = Figure::new(300, 700, 1, style(), 273);
        let mut rng = Rng::new(1);
        fig.expr.blush_t = 1000;
        let mut prev = fig.expr.blush;
        for _ in 0..300 {
            fig.step(&mut rng);
            assert!(fig.expr.blush >= prev, "blush snapped or reversed");
            assert!(fig.expr.blush <= 1000);
            prev = fig.expr.blush;
        }
        assert_eq!(fig.expr.blush, 1000);
    }

    #[test]
    fn gaze_stays_in_unit_disc() {
        let mut fig = Figure::new(300, 700, 1, style(), 273);
        let mut rng = Rng::new(1);
        fig.look_at(4000, -3000);
        for _ in 0..120 {
            fig.step(&mut rng);
            assert!(fig.gaze_x.abs() <= 1000 && fig.gaze_y.abs() <= 1000);
        }
        assert!(fig.gaze_x > 500, "gaze should track the far target");
    }
}
