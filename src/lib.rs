//! handfast — Handfast core (Rust, native + wasm32).
//!
//! One two-body beat: a handoff, a held gaze, a hand guided into
//! position. The core is a deterministic fixed-step simulation that
//! emits a per-frame draw list (see `draw`); the JS shell renders the
//! list, owns the DOM 18+ gate, input devices, and WebAudio. The same
//! core is driven headlessly by `tools/capture.mjs` for preview videos.
//!
//! Unsafe code lives only in this module's FFI boundary: each export
//! documents its contract, all pointers refer to core-owned statics
//! that outlive the call, and lengths are always paired with pointers.
//! The simulation modules are safe code throughout.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod draw;
pub mod figure;
pub mod fixed;
pub mod game;
pub mod levels;
pub mod palette;
pub mod rng;
pub mod save;

use std::sync::{Mutex, MutexGuard, OnceLock};

use draw::DrawList;
use game::{Game, Input};

/// The single game instance behind the FFI boundary.
static GAME: Mutex<Option<Game>> = Mutex::new(None);
/// The frame buffer the current draw list is rendered into.
static FRAME: Mutex<Option<DrawList>> = Mutex::new(None);
/// Scratch space the shell writes save bytes into before `hf_load`.
static SCRATCH: Mutex<Vec<u8>> = Mutex::new(Vec::new());
/// The input snapshot set by `hf_input`.
static INPUT: Mutex<Input> = Mutex::new(Input {
    pointer_x: 0,
    pointer_y: 0,
    pointer_present: false,
    pointer_down: false,
    confirm: false,
    back: false,
    up: false,
    down: false,
});

/// Scratch capacity; saves are capped well below this (save.rs: 256).
const SCRATCH_CAPACITY: usize = 512;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn with_game<R>(body: impl FnOnce(&mut Game) -> R) -> Option<R> {
    let mut slot = lock(&GAME);
    slot.as_mut().map(body)
}

fn palette_json() -> &'static str {
    static JSON: OnceLock<String> = OnceLock::new();
    JSON.get_or_init(palette::json)
}

/// Pointer to the palette JSON (UTF-8). Valid for the module's life.
/// Pair with `hf_palette_len`.
#[unsafe(no_mangle)]
pub extern "C" fn hf_palette_ptr() -> *const u8 {
    palette_json().as_ptr()
}

/// Length of the palette JSON in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn hf_palette_len() -> usize {
    palette_json().len()
}

/// Create a fresh game at the Title screen.
#[unsafe(no_mangle)]
pub extern "C" fn hf_new(seed: u32) {
    *lock(&GAME) = Some(Game::new(seed));
}

/// Create a game already inside `level`, driven by the demo policy
/// (the capture harness and the page's demo mode use this).
#[unsafe(no_mangle)]
pub extern "C" fn hf_start_demo(seed: u32, level: u32) {
    *lock(&GAME) = Some(Game::new_demo(seed, level as usize));
}

/// Set the input snapshot for subsequent steps. Flag bits: 1 present,
/// 2 pointer down, 4 confirm, 8 back, 16 up, 32 down.
#[unsafe(no_mangle)]
pub extern "C" fn hf_input(x: i32, y: i32, flags: u32) {
    *lock(&INPUT) = Input {
        pointer_x: x,
        pointer_y: y,
        pointer_present: flags & 1 != 0,
        pointer_down: flags & 2 != 0,
        confirm: flags & 4 != 0,
        back: flags & 8 != 0,
        up: flags & 16 != 0,
        down: flags & 32 != 0,
    };
}

/// Advance the game one fixed step under the current input snapshot.
#[unsafe(no_mangle)]
pub extern "C" fn hf_step() {
    let input = *lock(&INPUT);
    with_game(|game| game.step(&input));
}

/// Render the current frame into the frame buffer and return its
/// pointer. The buffer is core-owned and overwritten by the next
/// `hf_frame_ptr` call; the caller copies what it needs first.
#[unsafe(no_mangle)]
pub extern "C" fn hf_frame_ptr() -> *const u8 {
    let mut frame_slot = lock(&FRAME);
    let list = frame_slot.get_or_insert_with(DrawList::new);
    let drawn = with_game(|game| {
        game.draw(list);
        true
    });
    if drawn != Some(true) {
        list.clear();
    }
    list.as_str().as_ptr()
}

/// Length of the current frame buffer in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn hf_frame_len() -> usize {
    let frame_slot = lock(&FRAME);
    frame_slot.as_ref().map_or(0, |list| list.as_str().len())
}

/// Pointer to the one-line machine state (see `Game::state_line`).
/// The string is rebuilt per call into a leaked-free static buffer:
/// callers must copy before the next call.
#[unsafe(no_mangle)]
pub extern "C" fn hf_state_ptr() -> *const u8 {
    state_buffer().as_ptr()
}

/// Length of the state line in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn hf_state_len() -> usize {
    state_buffer().len()
}

fn state_buffer() -> &'static str {
    static STATE: Mutex<String> = Mutex::new(String::new());
    let mut slot = lock(&STATE);
    slot.clear();
    if let Some(text) = with_game(|game| game.state_line()) {
        slot.push_str(&text);
    }
    // SAFETY: the returned reference borrows the static's buffer. The
    // FFI contract is copy-before-next-call; the shell honours it by
    // reading through the exported memory immediately after each call.
    // We hand back a raw view by leaking the guard's target address as
    // a 'static str — sound under that contract because the buffer is
    // never freed, only overwritten under the same mutex.
    let ptr = slot.as_ptr();
    let len = slot.len();
    drop(slot);
    // SAFETY: see above; pointer remains valid (static storage).
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
}

/// Whether persisted state changed since the last `hf_clear_dirty`.
#[unsafe(no_mangle)]
pub extern "C" fn hf_dirty() -> u32 {
    with_game(|game| u32::from(game.save_dirty)).unwrap_or(0)
}

/// Mark persisted state as saved.
#[unsafe(no_mangle)]
pub extern "C" fn hf_clear_dirty() {
    with_game(|game| game.clear_save_dirty());
}

/// Pointer to the serialised save. Rebuilt per call; copy before the
/// next FFI call. Pair with `hf_save_len`.
#[unsafe(no_mangle)]
pub extern "C" fn hf_save_ptr() -> *const u8 {
    save_buffer().as_ptr()
}

/// Length of the serialised save in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn hf_save_len() -> usize {
    save_buffer().len()
}

fn save_buffer() -> &'static str {
    static SAVE: Mutex<String> = Mutex::new(String::new());
    let mut slot = lock(&SAVE);
    slot.clear();
    if let Some(text) = with_game(|game| game.save.serialise()) {
        slot.push_str(&text);
    }
    let ptr = slot.as_ptr();
    let len = slot.len();
    drop(slot);
    // SAFETY: same copy-before-next-call contract as `state_buffer`.
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
}

/// Writable scratch of `len` bytes (capped) for the shell to stage a
/// save string before `hf_load`. Returns null past the cap.
#[unsafe(no_mangle)]
pub extern "C" fn hf_scratch(len: usize) -> *mut u8 {
    if len > SCRATCH_CAPACITY {
        return std::ptr::null_mut();
    }
    let mut scratch = lock(&SCRATCH);
    scratch.clear();
    scratch.resize(len, 0);
    scratch.as_mut_ptr()
}

/// Load the save staged in scratch (length `len`). Returns 0 on
/// success, -1 on any parse/validation failure (state untouched).
///
/// # Safety
/// `len` must not exceed the length passed to the preceding
/// `hf_scratch` call; the bytes read are exactly the scratch buffer's.
#[unsafe(no_mangle)]
pub extern "C" fn hf_load(len: usize) -> i32 {
    if len > SCRATCH_CAPACITY {
        return -1;
    }
    let text = {
        let scratch = lock(&SCRATCH);
        if len > scratch.len() {
            return -1;
        }
        match std::str::from_utf8(&scratch[..len]) {
            Ok(text) => text.to_owned(),
            Err(_) => return -1,
        }
    };
    match with_game(|game| game.load_save(&text)) {
        Some(Ok(())) => 0,
        _ => -1,
    }
}
