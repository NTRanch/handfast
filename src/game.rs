//! The game: screens, the four-beat structure, coupled anchors, the
//! Appraisal Brush, dual gaze, and the demo policy.
//!
//! Beat structure (recipe §5): approach → contact → guidance → held
//! appraisal. Contact is the default state once planted: a planted hand
//! is *carried by the partner* — its world position is derived from
//! Sloane's body every step, so it cannot slide, float, or clip apart.
//! Contact breaks only on a sustained strain (a yank), which is the
//! separation attack the adversarial tests exercise.

use crate::draw::DrawList;
use crate::figure::{Figure, Style};
use crate::fixed::{self, Fx, px, to_px};
use crate::levels::{AnchorKind, LEVEL_COUNT, LEVELS};
use crate::palette as pal;
use crate::rng::Rng;
use crate::save::{SaveData, SaveError};

/// Frames of contact settle between a plant and the next beat.
const CONTACT_SETTLE_STEPS: u32 = 50;
/// Consecutive over-strain steps that tear a plant.
const TEAR_SUSTAIN_STEPS: u32 = 30;
/// Strikes (torn plants / broken gazes) that end a run.
const MAX_STRIKES: u32 = 3;
/// Upper bound on steps per `advance` call (the max-dt clamp).
pub const MAX_ADVANCE_STEPS: u32 = 600;
/// Ground line y, px.
const GROUND_Y: i32 = 724;

/// Event flags (state line / shell audio cues).
pub const EV_PLANT: u32 = 1;
/// Event: a plant tore.
pub const EV_BREAK: u32 = 2;
/// Event: victory.
pub const EV_VICTORY: u32 = 4;
/// Event: menu selection.
pub const EV_SELECT: u32 = 8;
/// Event: a guidance segment completed.
pub const EV_GUIDE_DONE: u32 = 16;

/// One step of player input. Coordinates are logical px and may be
/// hostile (the sim clamps before use).
#[derive(Clone, Copy, Default)]
pub struct Input {
    /// Pointer x, px.
    pub pointer_x: i32,
    /// Pointer y, px.
    pub pointer_y: i32,
    /// Pointer is present (moved at least once / demo policy active).
    pub pointer_present: bool,
    /// Pointer held down.
    pub pointer_down: bool,
    /// Confirm pressed (edge-detected by the game).
    pub confirm: bool,
    /// Back/pause pressed.
    pub back: bool,
    /// Menu up.
    pub up: bool,
    /// Menu down.
    pub down: bool,
}

/// Screens (18+ gate is the shell's DOM layer, before the core starts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Title + level select.
    Title,
    /// How to play.
    HowTo,
    /// Settings.
    Settings,
    /// Gameplay.
    Game,
    /// Pause.
    Pause,
    /// Victory.
    Victory,
    /// Game over.
    GameOver,
}

/// The beat within a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beat {
    /// Guide the free hand to the target anchor and plant it.
    Approach,
    /// Settle after a plant.
    Contact,
    /// The Brush travels the path; keep the planted hand with it.
    Guidance,
    /// Hold the mutual gaze at the final anchor.
    Appraisal,
}

/// The full game state.
pub struct Game {
    /// Persisted record (settings, unlocks, best scores).
    pub save: SaveData,
    /// Current screen.
    pub screen: Screen,
    /// Current beat (meaningful on the Game screen).
    pub beat: Beat,
    /// Level being played, 0-based.
    pub level_index: usize,
    /// Demo mode: the scripted policy drives the pointer.
    pub demo: bool,
    /// Events raised by the latest step (EV_* bitmask).
    pub events: u32,
    /// Set when persisted state changed; the shell saves on this flag.
    pub save_dirty: bool,
    prima: Figure,
    sloane: Figure,
    rng: Rng,
    seed: u32,
    frame: u64,
    beat_step: u32,
    target_anchor: usize,
    hand_x: Fx,
    hand_y: Fx,
    hand_render_x: Fx,
    hand_render_y: Fx,
    hand_planted: bool,
    brush_s: Fx,
    hand_s: Fx,
    dev_sum: i64,
    dev_count: u32,
    guidance_scores: [i32; 2],
    guidance_count: usize,
    strain_steps: u32,
    strikes: u32,
    regard_fx: Fx,
    appraisal_steps: u32,
    appraisal_inside: u32,
    score: i32,
    sel: usize,
    prev: Input,
    return_screen: Screen,
    demo_x: Fx,
    demo_y: Fx,
}

impl Game {
    /// A fresh game at the Title screen.
    pub fn new(seed: u32) -> Self {
        let mut game = Self::blank(seed);
        game.screen = Screen::Title;
        game
    }

    /// A game already inside a level, driven by the demo policy.
    pub fn new_demo(seed: u32, level: usize) -> Self {
        let mut game = Self::blank(seed);
        game.demo = true;
        game.start_level(level.min(LEVEL_COUNT - 1), seed);
        game
    }

    fn blank(seed: u32) -> Self {
        let prima = Figure::new(
            318,
            GROUND_Y,
            1,
            Style {
                hair: pal::MAGENTA,
                skin: pal::ROSE,
                cloth: pal::PLUM,
                accent: pal::PINK,
            },
            240,
        );
        let mut sloane = Figure::new(
            642,
            GROUND_Y,
            -1,
            Style {
                hair: pal::PLUM,
                skin: pal::ROSE_DIM,
                cloth: pal::VIOLET,
                accent: pal::VIOLET,
            },
            262,
        );
        sloane.activity = 1300;
        Self {
            save: SaveData::default(),
            screen: Screen::Title,
            beat: Beat::Approach,
            level_index: 0,
            demo: false,
            events: 0,
            save_dirty: false,
            prima,
            sloane,
            rng: Rng::new(seed),
            seed,
            frame: 0,
            beat_step: 0,
            target_anchor: 0,
            hand_x: px(370),
            hand_y: px(560),
            hand_render_x: px(370),
            hand_render_y: px(560),
            hand_planted: false,
            brush_s: 0,
            hand_s: 0,
            dev_sum: 0,
            dev_count: 0,
            guidance_scores: [0; 2],
            guidance_count: 0,
            strain_steps: 0,
            strikes: 0,
            regard_fx: 0,
            appraisal_steps: 0,
            appraisal_inside: 0,
            score: 0,
            sel: 0,
            prev: Input::default(),
            return_screen: Screen::Title,
            demo_x: px(480),
            demo_y: px(500),
        }
    }

    /// Apply a parsed save: settings reach the figures immediately;
    /// unlocks clamp to the level count.
    pub fn apply_save(&mut self, data: SaveData) {
        self.save = data;
        self.sync_settings();
    }

    /// Parse and apply a save string; errors leave state untouched.
    pub fn load_save(&mut self, text: &str) -> Result<(), SaveError> {
        let data = SaveData::parse(text)?;
        self.apply_save(data);
        Ok(())
    }

    fn sync_settings(&mut self) {
        let calm = self.save.settings.calm_mode;
        self.prima.calm = calm;
        self.sloane.calm = calm;
    }

    /// Enter a level: reset the beat machinery, reseed the run, and set
    /// the breathing rates from the level's escalation scale.
    pub fn start_level(&mut self, level: usize, seed: u32) {
        let level = level.min(LEVEL_COUNT - 1);
        self.level_index = level;
        self.seed = seed;
        self.rng = Rng::new(seed ^ 0x4841_4E44 ^ (level as u32));
        self.screen = Screen::Game;
        self.beat = Beat::Approach;
        self.beat_step = 0;
        self.target_anchor = 0;
        self.hand_planted = false;
        self.hand_x = px(370);
        self.hand_y = px(560);
        self.brush_s = 0;
        self.hand_s = 0;
        self.dev_sum = 0;
        self.dev_count = 0;
        self.guidance_count = 0;
        self.strain_steps = 0;
        self.strikes = 0;
        self.regard_fx = 0;
        self.appraisal_steps = 0;
        self.appraisal_inside = 0;
        self.score = 0;
        let scale = LEVELS[level].breath_scale_milli;
        self.prima = Figure::new(
            318,
            GROUND_Y,
            1,
            self.prima.style,
            (240 * scale / 1000) as u32,
        );
        self.sloane = Figure::new(
            642,
            GROUND_Y,
            -1,
            self.sloane.style,
            (262 * scale / 1000) as u32,
        );
        self.sloane.activity = 1300;
        self.sync_settings();
    }

    /// World position of an anchor on Sloane, px. The planted hand is
    /// derived from this every step — partner-carried by construction.
    pub fn anchor_world(&self, kind: AnchorKind) -> (i32, i32) {
        let (dx, dy) = kind.offset();
        let breath = self.sloane.breath_milli();
        let lift = if dy < -200 {
            breath * 3 / 1000
        } else {
            breath / 1000
        };
        (
            to_px(self.sloane.origin_x) + self.sloane.sway_px() + dx,
            GROUND_Y + dy - lift,
        )
    }

    /// The held-gaze point: midway between the two faces, px.
    pub fn regard_point(&self) -> (i32, i32) {
        let (px_, py) = self.prima.eye_px(1);
        let (sx, sy) = self.sloane.eye_px(-1);
        ((px_ + sx) / 2, (py + sy) / 2)
    }

    /// The anchor kind currently targeted (Approach/Guidance start).
    pub fn target_anchor_kind(&self) -> AnchorKind {
        LEVELS[self.level_index].anchors[self.target_anchor]
    }

    /// World position of the Appraisal Brush, px (Guidance beat).
    pub fn brush_world(&self) -> (i32, i32) {
        let segment = LEVELS[self.level_index].segments[self
            .target_anchor
            .min(LEVELS[self.level_index].segments.len() - 1)];
        let pos = segment_point(segment, self.brush_s, self);
        (to_px(pos.0), to_px(pos.1))
    }

    /// Current regard, 0..=1000.
    pub fn regard_milli(&self) -> i32 {
        (self.regard_fx / fixed::ONE).clamp(0, 1000)
    }

    /// Strikes so far this run.
    pub fn strikes(&self) -> u32 {
        self.strikes
    }

    /// Score of the finished run (valid on the Victory screen).
    pub fn score(&self) -> i32 {
        self.score
    }

    /// Is the guided hand currently planted?
    pub fn hand_planted(&self) -> bool {
        self.hand_planted
    }

    /// Distance from the rendered hand to its anchor while planted, px
    /// (the contact invariant the tests assert; includes the render
    /// spring's lag, which is the <=17 ms secondary-motion budget).
    pub fn planted_hand_error_px(&self) -> i32 {
        if !self.hand_planted {
            return 0;
        }
        if self.beat == Beat::Guidance {
            // Mid-guidance the hand slides along the body path: the
            // invariant is distance to the path, not to the start
            // anchor it left.
            let segment = LEVELS[self.level_index].segments[self.target_anchor];
            return distance_to_segment(segment, self.hand_render_x, self.hand_render_y, self);
        }
        let kind = LEVELS[self.level_index].anchors[self.target_anchor];
        let (ax, ay) = self.anchor_world(kind);
        let dx = self.hand_render_x - px(ax);
        let dy = self.hand_render_y - px(ay);
        to_px(fixed::length(dx, dy))
    }

    /// Advance `steps` fixed steps under one input snapshot. The count
    /// clamps to [`MAX_ADVANCE_STEPS`] — a max-dt burst cannot fast-
    /// forward the sim past its budget.
    pub fn advance(&mut self, steps: u32, input: &Input) {
        let steps = steps.min(MAX_ADVANCE_STEPS);
        for _ in 0..steps {
            self.step(input);
        }
    }

    /// One fixed step (1/60 s).
    pub fn step(&mut self, raw_input: &Input) {
        self.events = 0;
        self.frame += 1;
        let input = sanitise(raw_input);
        let effective = if self.demo { self.demo_policy() } else { input };
        match self.screen {
            Screen::Title
            | Screen::HowTo
            | Screen::Settings
            | Screen::Pause
            | Screen::Victory
            | Screen::GameOver => self.step_menu(&effective),
            Screen::Game => self.step_game(&effective),
        }
        if self.screen != Screen::Pause {
            self.prima.step(&mut self.rng);
            self.sloane.step(&mut self.rng);
        }
        // The rendered hand springs toward the logical hand while
        // planted: contact shows a breath of give, never a snap. Free,
        // the hand is the pointer and tracks it exactly.
        if self.hand_planted {
            self.hand_render_x += (self.hand_x - self.hand_render_x) / 3;
            self.hand_render_y += (self.hand_y - self.hand_render_y) / 3;
        } else {
            self.hand_render_x = self.hand_x;
            self.hand_render_y = self.hand_y;
        }
        self.prev = effective;
    }

    // ---- screens -------------------------------------------------

    fn menu_rows(&self) -> Vec<(&'static str, bool)> {
        match self.screen {
            Screen::Title => {
                let mut rows: Vec<(&'static str, bool)> = Vec::new();
                for (i, level) in LEVELS.iter().enumerate() {
                    rows.push((level.name, i <= self.save.unlocked));
                }
                rows.push(("HOW TO PLAY", true));
                rows.push(("SETTINGS", true));
                rows
            }
            Screen::HowTo => vec![("BACK", true)],
            Screen::Settings => vec![
                ("CALM MODE", true),
                ("IRIS GLOW", true),
                ("SOUND", true),
                ("BACK", true),
            ],
            Screen::Pause => vec![
                ("RESUME", true),
                ("SETTINGS", true),
                ("QUIT TO TITLE", true),
            ],
            Screen::Victory => {
                let mut rows = vec![];
                if self.level_index + 1 < LEVEL_COUNT {
                    rows.push(("NEXT LEVEL", true));
                }
                rows.push(("REPLAY", true));
                rows.push(("TITLE", true));
                rows
            }
            Screen::GameOver => vec![("TRY AGAIN", true), ("TITLE", true)],
            Screen::Game => vec![],
        }
    }

    /// Row rectangle (x, y, w, h) for menu row `i`.
    fn row_rect(i: usize) -> (i32, i32, i32, i32) {
        (280, 330 + (i as i32) * 62, 400, 50)
    }

    fn step_menu(&mut self, input: &Input) {
        let rows = self.menu_rows();
        if rows.is_empty() {
            return;
        }
        if edge(input.down, self.prev.down) {
            self.sel = (self.sel + 1) % rows.len();
        }
        if edge(input.up, self.prev.up) {
            self.sel = (self.sel + rows.len() - 1) % rows.len();
        }
        if input.pointer_present {
            for (i, _) in rows.iter().enumerate() {
                let (x, y, w, h) = Self::row_rect(i);
                if inside(input.pointer_x, input.pointer_y, x, y, w, h) {
                    self.sel = i;
                    if edge(input.pointer_down, self.prev.pointer_down) {
                        self.activate(self.sel);
                        return;
                    }
                }
            }
        }
        if edge(input.confirm, self.prev.confirm) {
            self.activate(self.sel);
        }
        if self.screen == Screen::Pause && edge(input.back, self.prev.back) {
            self.screen = Screen::Game;
        }
    }

    fn activate(&mut self, row: usize) {
        self.events |= EV_SELECT;
        match self.screen {
            Screen::Title => {
                if row < LEVEL_COUNT {
                    if row <= self.save.unlocked {
                        let seed = self.seed.wrapping_add(self.frame as u32);
                        self.start_level(row, seed);
                    }
                } else if row == LEVEL_COUNT {
                    self.goto(Screen::HowTo);
                } else {
                    self.return_screen = Screen::Title;
                    self.goto(Screen::Settings);
                }
            }
            Screen::HowTo => self.goto(Screen::Title),
            Screen::Settings => match row {
                0 => self.toggle_setting(Setting::Calm),
                1 => self.toggle_setting(Setting::Glow),
                2 => self.toggle_setting(Setting::Sound),
                _ => {
                    let back = self.return_screen;
                    self.goto(back);
                }
            },
            Screen::Pause => match row {
                0 => self.screen = Screen::Game,
                1 => {
                    self.return_screen = Screen::Pause;
                    self.goto(Screen::Settings);
                }
                _ => self.goto(Screen::Title),
            },
            Screen::Victory => {
                let base = if self.level_index + 1 < LEVEL_COUNT {
                    1
                } else {
                    0
                };
                match row {
                    0 if base == 1 => {
                        let seed = self.seed.wrapping_add(1);
                        self.start_level(self.level_index + 1, seed);
                    }
                    r if r == base => {
                        let seed = self.seed.wrapping_add(1);
                        self.start_level(self.level_index, seed);
                    }
                    _ => self.goto(Screen::Title),
                }
            }
            Screen::GameOver => match row {
                0 => {
                    let seed = self.seed.wrapping_add(1);
                    self.start_level(self.level_index, seed);
                }
                _ => self.goto(Screen::Title),
            },
            Screen::Game => {}
        }
    }

    fn goto(&mut self, screen: Screen) {
        self.screen = screen;
        self.sel = 0;
    }

    fn toggle_setting(&mut self, which: Setting) {
        match which {
            Setting::Calm => self.save.settings.calm_mode = !self.save.settings.calm_mode,
            Setting::Glow => self.save.settings.glow_default = !self.save.settings.glow_default,
            Setting::Sound => self.save.settings.sound_on = !self.save.settings.sound_on,
        }
        self.sync_settings();
        self.save_dirty = true;
    }

    // ---- gameplay ------------------------------------------------

    fn step_game(&mut self, input: &Input) {
        if edge(input.back, self.prev.back) && !self.demo {
            self.screen = Screen::Pause;
            self.sel = 0;
            return;
        }
        self.beat_step += 1;
        self.direct_gaze_and_expression();
        match self.beat {
            Beat::Approach => self.step_approach(input),
            Beat::Contact => self.step_contact(),
            Beat::Guidance => self.step_guidance(input),
            Beat::Appraisal => self.step_appraisal(input),
        }
    }

    fn step_approach(&mut self, input: &Input) {
        if input.pointer_present {
            self.hand_x = px(input.pointer_x);
            self.hand_y = px(input.pointer_y);
        }
        let kind = LEVELS[self.level_index].anchors[self.target_anchor];
        let (ax, ay) = self.anchor_world(kind);
        let dist = dist_px(self.hand_x, self.hand_y, px(ax), px(ay));
        let level = &LEVELS[self.level_index];
        if (input.pointer_down || input.confirm) && dist <= level.plant_radius_px {
            self.hand_planted = true;
            self.hand_x = px(ax);
            self.hand_y = px(ay);
            // The plant is a deliberate contact event: the render hand
            // arrives with it. The spring's give applies to the
            // carried motion that follows, not to this instant.
            self.hand_render_x = self.hand_x;
            self.hand_render_y = self.hand_y;
            self.beat = Beat::Contact;
            self.beat_step = 0;
            self.events |= EV_PLANT;
        }
    }

    fn step_contact(&mut self) {
        self.carry_hand();
        if self.beat_step >= CONTACT_SETTLE_STEPS {
            self.beat_step = 0;
            if self.target_anchor + 1 >= LEVELS[self.level_index].anchors.len() {
                self.beat = Beat::Appraisal;
                self.regard_fx = 0;
                self.appraisal_steps = 0;
                self.appraisal_inside = 0;
            } else {
                self.beat = Beat::Guidance;
                self.brush_s = 0;
                self.hand_s = 0;
                self.dev_sum = 0;
                self.dev_count = 0;
            }
        }
    }

    fn step_guidance(&mut self, input: &Input) {
        let level = &LEVELS[self.level_index];
        let segment = level.segments[self.target_anchor];
        let seg_len = segment_len(segment, self);
        let brush_pos = segment_point(segment, self.brush_s, self);
        let hand_pos = segment_point(segment, self.hand_s, self);
        // Strain: how far the player's intent is from the planted hand.
        let strain = dist_px(
            px(input.pointer_x),
            px(input.pointer_y),
            hand_pos.0,
            hand_pos.1,
        );
        if strain > level.tear_radius_px {
            self.strain_steps += 1;
            if self.strain_steps > TEAR_SUSTAIN_STEPS {
                self.break_contact();
                return;
            }
        } else {
            self.strain_steps = 0;
        }
        // Deviation from the Brush scores the guidance.
        self.dev_sum += i64::from(dist_px(
            px(input.pointer_x),
            px(input.pointer_y),
            brush_pos.0,
            brush_pos.1,
        ));
        self.dev_count += 1;
        // The Brush advances while the hand stays with it.
        if self.hand_s + px(60) >= self.brush_s {
            self.brush_s += (level.brush_speed_milli_px * fixed::ONE) / 1000;
            if self.brush_s > seg_len {
                self.brush_s = seg_len;
            }
        }
        // The planted hand slides along the body toward the player's
        // projected path position — contact is never broken by sliding.
        let target_s = segment_project(segment, px(input.pointer_x), px(input.pointer_y), self);
        let max_slide = px(2) + fixed::ONE / 2;
        if target_s > self.hand_s {
            self.hand_s = (self.hand_s + max_slide).min(target_s);
        } else {
            self.hand_s = (self.hand_s - max_slide).max(target_s);
        }
        self.hand_s = self.hand_s.clamp(0, seg_len);
        let (hx, hy) = segment_point(segment, self.hand_s, self);
        self.hand_x = hx;
        self.hand_y = hy;
        if self.brush_s >= seg_len && self.hand_s >= seg_len - px(2) {
            // Arrived at the next anchor: it plants by continuity.
            self.target_anchor += 1;
            let kind = LEVELS[self.level_index].anchors[self.target_anchor];
            let (ax, ay) = self.anchor_world(kind);
            self.hand_x = px(ax);
            self.hand_y = px(ay);
            let mean_dev = if self.dev_count > 0 {
                (self.dev_sum / i64::from(self.dev_count)) as i32
            } else {
                0
            };
            let steadiness = (1000 - mean_dev * 5).clamp(0, 1000);
            if self.guidance_count < self.guidance_scores.len() {
                self.guidance_scores[self.guidance_count] = steadiness;
                self.guidance_count += 1;
            }
            self.beat = Beat::Contact;
            self.beat_step = 0;
            self.strain_steps = 0;
            self.events |= EV_GUIDE_DONE | EV_PLANT;
        }
    }

    fn step_appraisal(&mut self, input: &Input) {
        self.carry_hand();
        let level = &LEVELS[self.level_index];
        self.appraisal_steps += 1;
        let (gx, gy) = self.regard_point();
        let dist = dist_px(px(input.pointer_x), px(input.pointer_y), px(gx), px(gy));
        let fill = (1000 * fixed::ONE) / (level.hold_steps as i32);
        if dist <= level.plant_radius_px + 26 {
            self.regard_fx = (self.regard_fx + fill).min(1000 * fixed::ONE);
            self.appraisal_inside += 1;
            self.strain_steps = 0;
        } else {
            self.regard_fx = (self.regard_fx - fill * 2).max(0);
            if dist > (level.plant_radius_px + 26) * 2 {
                self.strain_steps += 1;
                if self.strain_steps > TEAR_SUSTAIN_STEPS {
                    self.break_contact();
                    return;
                }
            } else {
                self.strain_steps = 0;
            }
        }
        if self.regard_milli() >= 1000 {
            self.win_level();
        } else if self.appraisal_steps > level.appraisal_deadline_steps {
            self.break_contact();
        }
    }

    /// The planted hand rides Sloane's body: derived, never integrated.
    fn carry_hand(&mut self) {
        let kind = LEVELS[self.level_index].anchors[self.target_anchor];
        let (ax, ay) = self.anchor_world(kind);
        self.hand_x = px(ax);
        self.hand_y = px(ay);
    }

    fn break_contact(&mut self) {
        self.hand_planted = false;
        self.strain_steps = 0;
        self.regard_fx = 0;
        self.strikes += 1;
        self.events |= EV_BREAK;
        if self.strikes >= MAX_STRIKES {
            self.screen = Screen::GameOver;
            self.sel = 0;
        } else {
            self.beat = Beat::Approach;
            self.beat_step = 0;
        }
    }

    fn win_level(&mut self) {
        let guidance_mean = if self.guidance_count > 0 {
            self.guidance_scores[..self.guidance_count]
                .iter()
                .sum::<i32>()
                / self.guidance_count as i32
        } else {
            1000
        };
        let smoothness = if self.appraisal_steps > 0 {
            (i64::from(self.appraisal_inside) * 1000 / i64::from(self.appraisal_steps)) as i32
        } else {
            0
        };
        self.score = (guidance_mean * 3 + smoothness * 2) / 5;
        if self.score > i32::from(self.save.best[self.level_index]) {
            self.save.best[self.level_index] = self.score as u16;
        }
        if self.level_index >= self.save.unlocked && self.level_index + 1 < LEVEL_COUNT {
            self.save.unlocked = self.level_index + 1;
        }
        self.save_dirty = true;
        self.events |= EV_VICTORY;
        self.screen = Screen::Victory;
        self.sel = 0;
    }

    // ---- direction (gaze + expression + glow) ---------------------

    fn direct_gaze_and_expression(&mut self) {
        let sloane_face = self.sloane.eye_px(-1);
        let prima_face = self.prima.eye_px(1);
        match self.beat {
            Beat::Approach => {
                self.sloane.look_at(to_px(self.hand_x), to_px(self.hand_y));
                self.prima.look_at(sloane_face.0, sloane_face.1);
                set_expr(&mut self.sloane, 800, 120, 300, 120);
                set_expr(&mut self.prima, 900, 140, 180, 60);
            }
            Beat::Contact => {
                self.sloane.look_at(prima_face.0, prima_face.1);
                self.prima.look_at(sloane_face.0, sloane_face.1);
                set_expr(&mut self.sloane, 760, 220, 470, 160);
                set_expr(&mut self.prima, 860, 200, 320, 100);
            }
            Beat::Guidance => {
                let segment = LEVELS[self.level_index].segments[self.target_anchor];
                let brush = segment_point(segment, self.brush_s, self);
                self.sloane.look_at(to_px(self.hand_x), to_px(self.hand_y));
                self.prima.look_at(to_px(brush.0), to_px(brush.1));
                let progress = if self.guidance_count > 0 { 650 } else { 520 };
                set_expr(&mut self.sloane, 700, 340, progress, 200);
                set_expr(&mut self.prima, 840, 240, 420, 120);
            }
            Beat::Appraisal => {
                self.sloane.look_at(prima_face.0, prima_face.1);
                self.prima.look_at(sloane_face.0, sloane_face.1);
                let regard = self.regard_milli();
                set_expr(&mut self.sloane, 620, 480, 850, 240);
                set_expr(&mut self.prima, 800, 320, 560, 160);
                let glow = self.save.settings.glow_default && regard > 500;
                self.sloane.glow = glow;
                self.prima.glow = glow;
                return;
            }
        }
        self.sloane.glow = false;
        self.prima.glow = false;
    }

    // ---- demo policy ----------------------------------------------

    fn demo_policy(&mut self) -> Input {
        let level = &LEVELS[self.level_index];
        let (tx, ty) = match self.beat {
            Beat::Approach | Beat::Contact => self.anchor_world(level.anchors[self.target_anchor]),
            Beat::Guidance => {
                let segment = level.segments[self.target_anchor];
                let pos = segment_point(segment, self.brush_s, self);
                (to_px(pos.0), to_px(pos.1))
            }
            Beat::Appraisal => self.regard_point(),
        };
        // Pursuit at a bounded speed, with a small deterministic wobble.
        let wobble_x = fixed::sin_milli((self.frame * 977 % 65536) as u32) * 5 / 1000;
        let wobble_y = fixed::cos_milli((self.frame * 613 % 65536) as u32) * 4 / 1000;
        let goal_x = px(tx + wobble_x);
        let goal_y = px(ty + wobble_y);
        let dx = goal_x - self.demo_x;
        let dy = goal_y - self.demo_y;
        let dist = fixed::length(dx, dy);
        let speed = px(3) + fixed::ONE / 4;
        if dist > speed {
            self.demo_x += fixed::div(fixed::mul(dx, speed), dist.max(1));
            self.demo_y += fixed::div(fixed::mul(dy, speed), dist.max(1));
        } else {
            self.demo_x = goal_x;
            self.demo_y = goal_y;
        }
        let near_anchor = self.beat == Beat::Approach
            && dist_px(self.demo_x, self.demo_y, px(tx), px(ty)) <= level.plant_radius_px / 2;
        Input {
            pointer_x: to_px(self.demo_x),
            pointer_y: to_px(self.demo_y),
            pointer_present: true,
            pointer_down: near_anchor,
            confirm: false,
            back: false,
            up: false,
            down: false,
        }
    }

    // ---- drawing ---------------------------------------------------

    /// Render the current state into a draw list (the frame the shell
    /// and the capture harness both consume).
    pub fn draw(&self, list: &mut DrawList) {
        list.clear();
        self.draw_scene(list);
        match self.screen {
            Screen::Game | Screen::Pause => self.draw_game_ui(list),
            _ => {}
        }
        if self.screen != Screen::Game {
            self.draw_menu(list);
        }
    }

    fn draw_scene(&self, list: &mut DrawList) {
        list.rect(0, 0, 960, 960, pal::BG);
        list.rect(0, GROUND_Y + 26, 960, 960 - GROUND_Y - 26, pal::PANEL);
        list.line(0, GROUND_Y + 26, 960, GROUND_Y + 26, 2, pal::PLUM);
        self.draw_motes(list);
        self.sloane.draw(list);
        self.prima.draw(list);
        if self.screen == Screen::Game || self.screen == Screen::Pause {
            self.draw_arm(list);
        }
    }

    fn draw_motes(&self, list: &mut DrawList) {
        let mut rng = Rng::new(self.seed ^ 0x4D4F_5445);
        for _ in 0..22 {
            let base_x = rng.below(940) + 10;
            let base_y = rng.below(700) + 30;
            let drift = fixed::sin_milli(
                (((self.frame * 41) % 65536) as u32 + (base_x as u32) * 131) % 65536,
            );
            let x = base_x + drift * 9 / 1000;
            let y = base_y + ((self.frame / 3 + base_x as u64) % 60) as i32 - 30;
            list.circle(x, y, 2, pal::PLUM);
        }
    }

    fn draw_arm(&self, list: &mut DrawList) {
        // Prima's guided arm: shoulder rides her breathing; the hand is
        // the sim object; the elbow sags between them.
        let breath = self.prima.breath_milli();
        let sh_x = to_px(self.prima.origin_x) + 54;
        let sh_y = GROUND_Y - 302 - breath * 3 / 1000;
        let hx = to_px(self.hand_render_x);
        let hy = to_px(self.hand_render_y);
        let ex = (sh_x + hx) / 2 + 8;
        let ey = (sh_y + hy) / 2 + 30;
        list.line(sh_x, sh_y, ex, ey, 13, pal::ROSE);
        list.line(ex, ey, hx, hy, 12, pal::ROSE);
        if self.hand_planted {
            list.circle(hx, hy, 17, pal::GOLD);
        }
        list.circle(hx, hy, 12, pal::ROSE);
        list.circle(hx - 3, hy - 3, 4, pal::ROSE);
    }

    fn draw_game_ui(&self, list: &mut DrawList) {
        let level = &LEVELS[self.level_index];
        list.text(28, 26, 20, pal::CREAM, level.name);
        list.text(
            28,
            56,
            14,
            pal::VIOLET_DIM,
            match self.beat {
                Beat::Approach => "BEAT: APPROACH",
                Beat::Contact => "BEAT: CONTACT",
                Beat::Guidance => "BEAT: GUIDANCE",
                Beat::Appraisal => "BEAT: HELD APPRAISAL",
            },
        );
        let breaks = format!("BREAKS {}/{}", self.strikes, MAX_STRIKES);
        list.text(800, 26, 14, pal::MAGENTA, &breaks);
        match self.beat {
            Beat::Approach => {
                let kind = level.anchors[self.target_anchor];
                let (ax, ay) = self.anchor_world(kind);
                let pulse =
                    6 + fixed::sin_milli((self.frame * 1093 % 65536) as u32).abs() * 6 / 1000;
                list.circle(ax, ay, level.plant_radius_px + pulse, pal::PLUM);
                list.circle(ax, ay, level.plant_radius_px + pulse - 4, pal::BG);
                list.circle(ax, ay, 7, pal::GOLD);
                list.text(
                    ax - 30,
                    ay - level.plant_radius_px - 30,
                    14,
                    pal::GOLD,
                    kind.name(),
                );
                list.text(
                    250,
                    880,
                    16,
                    pal::CREAM,
                    "GUIDE THE HAND TO THE MARK AND HOLD",
                );
            }
            Beat::Contact => {
                list.text(
                    330,
                    880,
                    16,
                    pal::CREAM,
                    "CONTACT. SHE CARRIES THE HAND NOW",
                );
            }
            Beat::Guidance => {
                let segment = level.segments[self.target_anchor];
                for pair in segment.windows(2) {
                    let (ax, ay) = self.anchor_offset_world(pair[0]);
                    let (bx, by) = self.anchor_offset_world(pair[1]);
                    list.line(ax, ay, bx, by, 3, pal::VIOLET_DIM);
                }
                let brush = segment_point(segment, self.brush_s, self);
                list.circle(to_px(brush.0), to_px(brush.1), 13, pal::PLUM);
                list.circle(to_px(brush.0), to_px(brush.1), 9, pal::GOLD);
                list.text(
                    235,
                    880,
                    16,
                    pal::CREAM,
                    "KEEP THE HAND WITH THE BRUSH. DO NOT YANK",
                );
            }
            Beat::Appraisal => {
                let (gx, gy) = self.regard_point();
                list.circle(gx, gy, 26, pal::PLUM);
                list.circle(gx, gy, 21, pal::BG);
                let filled = self.regard_milli() * 24 / 1000;
                for i in 0..filled {
                    let phase = (i * 65536 / 24) as u32;
                    let dx = fixed::cos_milli(phase) * 23 / 1000;
                    let dy = fixed::sin_milli(phase) * 23 / 1000;
                    list.circle(gx + dx, gy + dy, 3, pal::GOLD);
                }
                list.text(300, 880, 16, pal::CREAM, "HOLD THE GAZE. REST ON THE RING");
            }
        }
        if self.screen == Screen::Pause {
            list.rect(240, 240, 480, 470, pal::PANEL);
        }
    }

    /// Anchor-offset (path waypoint) to world px.
    fn anchor_offset_world(&self, offset: (i32, i32)) -> (i32, i32) {
        let breath = self.sloane.breath_milli();
        let lift = if offset.1 < -200 {
            breath * 3 / 1000
        } else {
            breath / 1000
        };
        (
            to_px(self.sloane.origin_x) + self.sloane.sway_px() + offset.0,
            GROUND_Y + offset.1 - lift,
        )
    }

    fn draw_menu(&self, list: &mut DrawList) {
        match self.screen {
            Screen::Title => {
                list.text(330, 120, 42, pal::PINK, "HANDFAST");
                list.text(
                    262,
                    176,
                    14,
                    pal::CREAM,
                    "PRIMA AND SLOANE. ONE TWO-BODY BEAT",
                );
            }
            Screen::HowTo => {
                list.text(330, 120, 32, pal::PINK, "HOW TO PLAY");
                let lines = [
                    "MOVE THE POINTER TO GUIDE PRIMAS HAND.",
                    "APPROACH: REST THE HAND ON THE GOLD MARK",
                    "AND PRESS TO PLANT IT.",
                    "CONTACT: HER BODY CARRIES THE HAND.",
                    "GUIDANCE: THE APPRAISAL BRUSH TRAVELS.",
                    "KEEP THE HAND WITH IT. A YANK TEARS IT.",
                    "HELD APPRAISAL: REST YOUR POINTER ON THE",
                    "RING BETWEEN THEIR FACES AND HOLD THE",
                    "GAZE UNTIL THE RING FILLS GOLD.",
                    "THREE BREAKS END THE RUN. ESC PAUSES.",
                ];
                for (i, line) in lines.iter().enumerate() {
                    list.text(150, 220 + (i as i32) * 34, 15, pal::CREAM, line);
                }
            }
            Screen::Settings => {
                list.text(352, 120, 32, pal::PINK, "SETTINGS");
            }
            Screen::Pause => {
                list.text(398, 268, 30, pal::PINK, "PAUSED");
            }
            Screen::Victory => {
                list.text(352, 120, 36, pal::PINK, "APPRAISED");
                let line = format!(
                    "SCORE {}  BEST {}",
                    self.score, self.save.best[self.level_index]
                );
                list.text(330, 190, 18, pal::GOLD, &line);
                list.text(300, 226, 14, pal::CREAM, "THE HAND HELD. THE GAZE HELD");
            }
            Screen::GameOver => {
                list.text(330, 120, 36, pal::MAGENTA, "UNFASTENED");
                list.text(316, 190, 15, pal::CREAM, "THE HAND TORE FREE THREE TIMES");
            }
            Screen::Game => {}
        }
        if self.screen == Screen::Settings {
            self.draw_settings_values(list);
        }
        let rows = self.menu_rows();
        for (i, (label, enabled)) in rows.iter().enumerate() {
            let (x, y, w, h) = Self::row_rect(i);
            let selected = i == self.sel;
            list.rect(x, y, w, h, if selected { pal::PLUM } else { pal::PANEL });
            if selected {
                list.rect(x, y, w, 3, pal::PINK);
            }
            let color = if *enabled {
                pal::CREAM
            } else {
                pal::VIOLET_DIM
            };
            let mut text = (*label).to_string();
            if self.screen == Screen::Title && i < LEVEL_COUNT {
                text = format!("LEVEL {} {}", i + 1, label);
                if !enabled {
                    text = format!("LEVEL {} {} LOCKED", i + 1, label);
                }
                if self.save.best[i] > 0 {
                    text = format!("{}  BEST {}", text, self.save.best[i]);
                }
            }
            list.text(x + 18, y + 17, 15, color, &text);
        }
    }

    fn draw_settings_values(&self, list: &mut DrawList) {
        let values = [
            if self.save.settings.calm_mode {
                "ON"
            } else {
                "OFF"
            },
            if self.save.settings.glow_default {
                "ON"
            } else {
                "OFF"
            },
            if self.save.settings.sound_on {
                "ON"
            } else {
                "OFF"
            },
        ];
        for (i, value) in values.iter().enumerate() {
            let (_, y, _, _) = Self::row_rect(i);
            list.text(610, y + 17, 15, pal::GOLD, value);
        }
    }

    /// One-line machine-readable state for the shell: screen, events
    /// bitmask, sound flag, save-dirt flag. The shell never parses the
    /// draw list for logic; this line is the contract.
    pub fn state_line(&self) -> String {
        format!(
            "# screen={} beat={} ev={} sound={} dirty={} score={}",
            screen_name(self.screen),
            beat_name(self.beat),
            self.events,
            self.save.settings.sound_on as u8,
            self.save_dirty as u8,
            self.score,
        )
    }

    /// Clear the save-dirt flag after the shell persisted the save.
    pub fn clear_save_dirty(&mut self) {
        self.save_dirty = false;
    }
}

enum Setting {
    Calm,
    Glow,
    Sound,
}

fn screen_name(screen: Screen) -> &'static str {
    match screen {
        Screen::Title => "title",
        Screen::HowTo => "howto",
        Screen::Settings => "settings",
        Screen::Game => "game",
        Screen::Pause => "pause",
        Screen::Victory => "victory",
        Screen::GameOver => "gameover",
    }
}

fn beat_name(beat: Beat) -> &'static str {
    match beat {
        Beat::Approach => "approach",
        Beat::Contact => "contact",
        Beat::Guidance => "guidance",
        Beat::Appraisal => "appraisal",
    }
}

fn edge(now: bool, before: bool) -> bool {
    now && !before
}

fn inside(x: i32, y: i32, rx: i32, ry: i32, rw: i32, rh: i32) -> bool {
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
}

fn sanitise(input: &Input) -> Input {
    let mut clean = *input;
    clean.pointer_x = clean.pointer_x.clamp(-64, 1024);
    clean.pointer_y = clean.pointer_y.clamp(-64, 1024);
    clean
}

fn dist_px(ax: Fx, ay: Fx, bx: Fx, by: Fx) -> i32 {
    to_px(fixed::length(ax - bx, ay - by))
}

fn set_expr(fig: &mut Figure, lid: i32, mouth: i32, blush: i32, brow: i32) {
    let amp = fig.activity;
    fig.expr.lid_t = lid;
    fig.expr.mouth_t = mouth * amp / 1000;
    fig.expr.blush_t = blush * amp / 1000;
    fig.expr.brow_t = brow;
}

/// Polyline length in fixed point, following Sloane's carried frame:
/// waypoints are offsets; consecutive differences are body-relative,
/// so the length is frame-independent.
fn segment_len(segment: &[(i32, i32)], _game: &Game) -> Fx {
    let mut total = 0;
    for pair in segment.windows(2) {
        total += fixed::length(px(pair[1].0 - pair[0].0), px(pair[1].1 - pair[0].1));
    }
    total
}

/// Point at arc distance `s` along a segment, in world fixed point.
/// The whole polyline rides Sloane's sway/breath via its first point's
/// world position — the path is on her body, so it is carried too.
fn segment_point(segment: &[(i32, i32)], s: Fx, game: &Game) -> (Fx, Fx) {
    let (ox, oy) = game.anchor_offset_world(segment[0]);
    let mut remaining = s;
    for pair in segment.windows(2) {
        let dx = px(pair[1].0 - pair[0].0);
        let dy = px(pair[1].1 - pair[0].1);
        let len = fixed::length(dx, dy);
        if remaining <= len || pair[1] == segment[segment.len() - 1] {
            let t = if len > 0 {
                fixed::div(remaining.clamp(0, len), len)
            } else {
                0
            };
            let base_x = px(ox + (pair[0].0 - segment[0].0));
            let base_y = px(oy + (pair[0].1 - segment[0].1));
            return (base_x + fixed::mul(dx, t), base_y + fixed::mul(dy, t));
        }
        remaining -= len;
    }
    let last = segment[segment.len() - 1];
    (
        px(ox + (last.0 - segment[0].0)),
        px(oy + (last.1 - segment[0].1)),
    )
}

/// Distance in px from a world fixed-point position to a segment
/// polyline (the body's surface path).
fn distance_to_segment(segment: &[(i32, i32)], x: Fx, y: Fx, game: &Game) -> i32 {
    let mut best = i32::MAX;
    for pair in segment.windows(2) {
        let (ax, ay) = game.anchor_offset_world(pair[0]);
        let (bx, by) = game.anchor_offset_world(pair[1]);
        let (axf, ayf) = (px(ax), px(ay));
        let dx = px(bx - ax);
        let dy = px(by - ay);
        let len2 = i64::from(fixed::mul(dx, dx)) + i64::from(fixed::mul(dy, dy));
        let t_fx = if len2 > 0 {
            let num = i64::from(fixed::mul(x - axf, dx)) + i64::from(fixed::mul(y - ayf, dy));
            ((num * 1000 / len2).clamp(0, 1000) as i32) * fixed::ONE / 1000
        } else {
            0
        };
        let cx = axf + fixed::mul(dx, t_fx);
        let cy = ayf + fixed::mul(dy, t_fx);
        best = best.min(dist_px(x, y, cx, cy));
    }
    best
}

/// Arc-distance projection of a world point onto a segment.
fn segment_project(segment: &[(i32, i32)], x: Fx, y: Fx, game: &Game) -> Fx {
    let mut best_s = 0;
    let mut best_d = i64::MAX;
    let mut acc = 0;
    for pair in segment.windows(2) {
        let (ax, ay) = game.anchor_offset_world(pair[0]);
        let (bx, by) = game.anchor_offset_world(pair[1]);
        let (axf, ayf) = (px(ax), px(ay));
        let dx = px(bx - ax);
        let dy = px(by - ay);
        let len2 = i64::from(fixed::mul(dx, dx)) + i64::from(fixed::mul(dy, dy));
        let t = if len2 > 0 {
            let num = i64::from(fixed::mul(x - axf, dx)) + i64::from(fixed::mul(y - ayf, dy));
            (num * 1000 / len2).clamp(0, 1000)
        } else {
            0
        };
        let t_fx = (t as i32) * fixed::ONE / 1000;
        let cx = axf + fixed::mul(dx, t_fx);
        let cy = ayf + fixed::mul(dy, t_fx);
        let d = i64::from(x - cx).pow(2) + i64::from(y - cy).pow(2);
        let len = fixed::length(dx, dy);
        if d < best_d {
            best_d = d;
            best_s = acc + fixed::mul(len, t_fx);
        }
        acc += len;
    }
    best_s
}
