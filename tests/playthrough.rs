//! Integration gates for Handfast: half validation (the happy path,
//! end to end, through the real input surface), half adversarial
//! (separation attacks, hostile coordinates, degenerate timing,
//! malformed saves, invariant fuzz). Estate 50/50 split.

use handfast::draw::{DrawList, MAX_OPS};
use handfast::game::{Beat, Game, Input, Screen};
use handfast::levels::LEVEL_COUNT;
use handfast::rng::Rng;
use handfast::save::SaveData;

fn input_at(x: i32, y: i32, down: bool) -> Input {
    Input {
        pointer_x: x,
        pointer_y: y,
        pointer_present: true,
        pointer_down: down,
        ..Input::default()
    }
}

/// A competent external player: reads only public state, drives the
/// real input surface. Used by the playthrough validation tests.
fn play_one_step(game: &mut Game) {
    let input = match game.screen {
        Screen::Game => match game.beat {
            Beat::Approach => {
                let kind = game.target_anchor_kind();
                let (ax, ay) = game.anchor_world(kind);
                input_at(ax, ay, true)
            }
            Beat::Contact => {
                let kind = game.target_anchor_kind();
                let (ax, ay) = game.anchor_world(kind);
                input_at(ax, ay, false)
            }
            Beat::Guidance => {
                let (bx, by) = game.brush_world();
                input_at(bx, by, false)
            }
            Beat::Appraisal => {
                let (gx, gy) = game.regard_point();
                input_at(gx, gy, false)
            }
        },
        _ => Input::default(),
    };
    game.step(&input);
}

fn play_to_victory(level: usize, seed: u32) -> Game {
    let mut game = Game::new(seed);
    game.start_level(level, seed);
    for _ in 0..60 * 240 {
        if game.screen == Screen::Victory {
            return game;
        }
        assert_ne!(
            game.screen,
            Screen::GameOver,
            "level {level} should be winnable"
        );
        play_one_step(&mut game);
    }
    panic!("level {level} did not reach victory in the step budget");
}

// ---------- validation ----------

#[test]
fn every_level_is_winnable_end_to_end() {
    for level in 0..LEVEL_COUNT {
        let game = play_to_victory(level, 100 + level as u32);
        assert!(game.score() > 400, "level {level} score {}", game.score());
        assert_eq!(game.save.best[level], game.score() as u16);
    }
}

#[test]
fn plant_requires_proximity_and_press() {
    let mut game = Game::new(5);
    game.start_level(0, 5);
    let kind = game.target_anchor_kind();
    let (ax, ay) = game.anchor_world(kind);
    // Hover without pressing: no plant.
    for _ in 0..30 {
        game.step(&input_at(ax, ay, false));
    }
    assert!(!game.hand_planted());
    // Press far away: no plant.
    for _ in 0..10 {
        game.step(&input_at(60, 60, true));
    }
    assert!(!game.hand_planted());
    // Press on the mark: plant.
    game.step(&input_at(ax, ay, true));
    assert!(game.hand_planted());
    assert_eq!(game.beat, Beat::Contact);
}

#[test]
fn planted_hand_is_carried_by_the_partner() {
    let mut game = Game::new(9);
    game.start_level(0, 9);
    let kind = game.target_anchor_kind();
    let (ax, ay) = game.anchor_world(kind);
    game.step(&input_at(ax, ay, true));
    assert!(game.hand_planted());
    // Through Contact and beyond, the render hand tracks the anchor
    // within the render-spring lag budget while Sloane breathes/sways.
    for _ in 0..600 {
        play_one_step(&mut game);
        if game.hand_planted() {
            let err = game.planted_hand_error_px();
            assert!(err <= 20, "planted hand drifted {err}px from its anchor");
        }
    }
}

#[test]
fn unlocks_progress_and_persist() {
    let game = play_to_victory(0, 77);
    assert_eq!(game.save.unlocked, 1);
    let text = game.save.serialise();
    let mut restored = Game::new(1);
    restored.load_save(&text).expect("own save must parse");
    assert_eq!(restored.save.unlocked, 1);
    assert_eq!(restored.save.best[0], game.save.best[0]);
}

#[test]
fn demo_mode_wins_every_level_unattended() {
    for level in 0..LEVEL_COUNT {
        let mut game = Game::new_demo(400 + level as u32, level);
        let mut won = false;
        for _ in 0..60 * 240 {
            game.step(&Input::default());
            if game.screen == Screen::Victory {
                won = true;
                break;
            }
        }
        assert!(won, "demo policy should win level {level}");
    }
}

#[test]
fn determinism_same_seed_same_draw_lists() {
    let mut a = Game::new_demo(1234, 1);
    let mut b = Game::new_demo(1234, 1);
    let mut list_a = DrawList::new();
    let mut list_b = DrawList::new();
    for step in 0..1200 {
        a.step(&Input::default());
        b.step(&Input::default());
        if step % 60 == 0 {
            a.draw(&mut list_a);
            b.draw(&mut list_b);
            assert_eq!(
                list_a.hash(),
                list_b.hash(),
                "draw lists diverged at step {step}"
            );
            assert!(!list_a.overflowed);
            assert!(list_a.op_count() <= MAX_OPS);
        }
    }
}

#[test]
fn different_seeds_diverge() {
    let mut a = Game::new_demo(1, 0);
    let mut b = Game::new_demo(2, 0);
    let mut list_a = DrawList::new();
    let mut list_b = DrawList::new();
    for _ in 0..600 {
        a.step(&Input::default());
        b.step(&Input::default());
    }
    a.draw(&mut list_a);
    b.draw(&mut list_b);
    assert_ne!(
        list_a.hash(),
        list_b.hash(),
        "different seeds should diverge"
    );
}

// ---------- adversarial ----------

#[test]
fn separation_attack_tears_the_plant_and_counts_a_strike() {
    let mut game = Game::new(11);
    game.start_level(0, 11);
    // Plant, ride into Guidance.
    for _ in 0..200 {
        play_one_step(&mut game);
        if game.beat == Beat::Guidance {
            break;
        }
    }
    assert_eq!(game.beat, Beat::Guidance);
    // The attack: pointer teleports to the far corner and stays.
    for _ in 0..(31 + 5) {
        game.step(&input_at(0, 0, false));
    }
    assert!(!game.hand_planted(), "sustained yank must tear the plant");
    assert_eq!(game.strikes(), 1);
    assert_eq!(game.beat, Beat::Approach);
    // And the run is still winnable afterwards (failure is a beat
    // reset, not a soft-lock).
    for _ in 0..60 * 240 {
        if game.screen == Screen::Victory {
            return;
        }
        play_one_step(&mut game);
    }
    panic!("run should recover and win after one strike");
}

#[test]
fn three_separation_attacks_end_the_run() {
    let mut game = Game::new(13);
    game.start_level(0, 13);
    for strike in 1..=3 {
        // Plant again.
        for _ in 0..400 {
            play_one_step(&mut game);
            if game.beat == Beat::Guidance || game.screen == Screen::GameOver {
                break;
            }
        }
        if game.screen == Screen::GameOver {
            break;
        }
        for _ in 0..40 {
            game.step(&input_at(959, 0, false));
        }
        if strike < 3 {
            assert_eq!(game.strikes(), strike);
        }
    }
    assert_eq!(game.screen, Screen::GameOver);
}

#[test]
fn hostile_coordinates_never_panic_or_corrupt() {
    let mut game = Game::new(17);
    game.start_level(2, 17);
    let extremes = [
        (i32::MAX, i32::MIN),
        (i32::MIN, i32::MAX),
        (-1, -1),
        (1_000_000, 1_000_000),
        (0, 0),
    ];
    for (x, y) in extremes {
        for _ in 0..120 {
            game.step(&input_at(x, y, true));
        }
    }
    // The state machine is still coherent and playable.
    assert!(game.strikes() <= 3);
    for _ in 0..60 * 240 {
        if game.screen == Screen::Victory || game.screen == Screen::GameOver {
            return;
        }
        play_one_step(&mut game);
    }
    panic!("game should resolve after hostile input");
}

#[test]
fn zero_and_max_dt_are_clamped() {
    let mut game = Game::new_demo(21, 0);
    let mut list = DrawList::new();
    game.draw(&mut list);
    let before = list.hash();
    game.advance(0, &Input::default());
    game.draw(&mut list);
    assert_eq!(list.hash(), before, "advance(0) must be a no-op");
    // A max-dt burst advances at most the clamp, then play continues.
    game.advance(u32::MAX, &Input::default());
    assert!(game.screen == Screen::Game || game.screen == Screen::Victory);
}

#[test]
fn malformed_saves_are_rejected_without_mutation() {
    let mut game = Game::new(23);
    let before = game.save;
    for bad in [
        "",
        "HF1;",
        "HF1;calm=9;glow=1;sound=1;unlocked=0;best=0,0,0",
        "HF1;calm=0;glow=1;sound=1;unlocked=0;best=0,0,9999",
        "garbage",
    ] {
        assert!(game.load_save(bad).is_err(), "{bad:?} must be rejected");
        assert_eq!(game.save, before, "rejected save must not mutate state");
    }
    let good = SaveData::default().serialise();
    assert!(game.load_save(&good).is_ok());
}

#[test]
fn invariant_fuzz_under_random_inputs() {
    let mut rng = Rng::new(99);
    let mut game = Game::new(99);
    game.start_level(1, 99);
    let mut list = DrawList::new();
    for step in 0..20_000 {
        let input = Input {
            pointer_x: rng.below(1400) - 200,
            pointer_y: rng.below(1400) - 200,
            pointer_present: rng.below(10) > 0,
            pointer_down: rng.below(2) == 0,
            confirm: rng.below(20) == 0,
            back: rng.below(500) == 0,
            up: rng.below(50) == 0,
            down: rng.below(50) == 0,
        };
        game.step(&input);
        assert!(game.regard_milli() >= 0 && game.regard_milli() <= 1000);
        if game.hand_planted() {
            assert!(
                game.planted_hand_error_px() <= 24,
                "step {step}: planted hand error {}",
                game.planted_hand_error_px()
            );
        }
        if step % 500 == 0 {
            game.draw(&mut list);
            assert!(!list.overflowed, "draw list overflowed under fuzz");
            // Recover to a sane screen if the fuzz wandered off.
            if game.screen == Screen::GameOver || game.screen == Screen::Victory {
                game.start_level(1, 99);
            }
        }
    }
}
