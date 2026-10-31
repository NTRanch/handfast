//! Level data: the escalating stages of the encounter.
//!
//! Anchors and path waypoints are offsets in px from Sloane's ground
//! origin (she faces -x; her near side is negative x). A level is a
//! chain: plant anchor 1, be guided along segment 1 to anchor 2, …,
//! and finish with the held appraisal at the last anchor.

/// A plantable anchor on Sloane's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorKind {
    /// Cheek — the final, face-led plant (the reference's after-phase).
    Cheek,
    /// Heart — chest.
    Heart,
    /// Hip — the first, easiest plant.
    Hip,
    /// Shoulder.
    Shoulder,
}

impl AnchorKind {
    /// Offset from Sloane's ground origin, px.
    pub fn offset(self) -> (i32, i32) {
        match self {
            AnchorKind::Cheek => (-34, -382),
            AnchorKind::Heart => (-16, -262),
            AnchorKind::Hip => (-58, -140),
            AnchorKind::Shoulder => (-56, -300),
        }
    }

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            AnchorKind::Cheek => "CHEEK",
            AnchorKind::Heart => "HEART",
            AnchorKind::Hip => "HIP",
            AnchorKind::Shoulder => "SHOULDER",
        }
    }
}

/// One level's tuning and geometry.
pub struct Level {
    /// Display name.
    pub name: &'static str,
    /// Anchor chain, in planting order.
    pub anchors: &'static [AnchorKind],
    /// Guidance polylines: `segments[i]` runs from `anchors[i]` to
    /// `anchors[i + 1]`, endpoints inclusive.
    pub segments: &'static [&'static [(i32, i32)]],
    /// Plant capture radius, px.
    pub plant_radius_px: i32,
    /// Strain distance at which a plant tears, px.
    pub tear_radius_px: i32,
    /// Appraisal hold the regard meter must fill within ~this many
    /// steps of presence, steps (60/s).
    pub hold_steps: u32,
    /// Appraisal hard deadline, steps.
    pub appraisal_deadline_steps: u32,
    /// Brush travel speed along the path, milli-px per step.
    pub brush_speed_milli_px: i32,
    /// Breathing-rate multiplier, milli.
    pub breath_scale_milli: i32,
    /// The iris-glow layer switches on at the appraisal peak.
    pub glow_at_peak: bool,
}

/// The three levels.
pub const LEVELS: [Level; 3] = [
    Level {
        name: "FIRST TOUCH",
        anchors: &[AnchorKind::Hip, AnchorKind::Heart],
        segments: &[&[(-58, -140), (-64, -200), (-40, -240), (-16, -262)]],
        plant_radius_px: 46,
        tear_radius_px: 130,
        hold_steps: 240,
        appraisal_deadline_steps: 900,
        brush_speed_milli_px: 550,
        breath_scale_milli: 1000,
        glow_at_peak: false,
    },
    Level {
        name: "THE BRUSH'S PATH",
        anchors: &[AnchorKind::Hip, AnchorKind::Shoulder, AnchorKind::Cheek],
        segments: &[
            &[(-58, -140), (-70, -210), (-62, -262), (-56, -300)],
            &[(-56, -300), (-52, -336), (-34, -382)],
        ],
        plant_radius_px: 38,
        tear_radius_px: 105,
        hold_steps: 360,
        appraisal_deadline_steps: 1200,
        brush_speed_milli_px: 500,
        breath_scale_milli: 1150,
        glow_at_peak: false,
    },
    Level {
        name: "HELD APPRAISAL",
        anchors: &[AnchorKind::Shoulder, AnchorKind::Heart, AnchorKind::Cheek],
        segments: &[
            &[(-56, -300), (-40, -286), (-16, -262)],
            &[(-16, -262), (-30, -320), (-34, -382)],
        ],
        plant_radius_px: 32,
        tear_radius_px: 90,
        hold_steps: 480,
        appraisal_deadline_steps: 1500,
        brush_speed_milli_px: 450,
        breath_scale_milli: 1300,
        glow_at_peak: true,
    },
];

/// Number of levels.
pub const LEVEL_COUNT: usize = LEVELS.len();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_connect_the_anchor_chain() {
        for level in &LEVELS {
            assert_eq!(level.segments.len() + 1, level.anchors.len());
            for (i, segment) in level.segments.iter().enumerate() {
                assert_eq!(segment.first(), Some(&level.anchors[i].offset()));
                assert_eq!(segment.last(), Some(&level.anchors[i + 1].offset()));
            }
        }
    }

    #[test]
    fn levels_escalate() {
        assert!(LEVELS[0].plant_radius_px > LEVELS[2].plant_radius_px);
        assert!(LEVELS[0].hold_steps < LEVELS[2].hold_steps);
        assert!(LEVELS[0].anchors.len() < LEVELS[2].anchors.len());
    }
}
