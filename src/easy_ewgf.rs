use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU32, AtomicU8, Ordering};

use crate::hid::{
    get_npad_full_key_state, get_npad_gc_state, get_npad_handheld_state,
    get_npad_joy_dual_state, get_npad_joy_left_state, get_npad_joy_right_state,
    NpadState, BUTTON_A,
};

// phases 1..=3 emit the 623+A command one step at a time
static INJECT_PHASE: AtomicU8 = AtomicU8::new(0);
static LAST_UPDATE_COUNT: AtomicI64 = AtomicI64::new(-1);

// most recent direction the user nudged the L-stick in
static LAST_DIRECTION: AtomicI32 = AtomicI32::new(1);
static DIRECTION_FRESHNESS: AtomicU32 = AtomicU32::new(0);
static CACHED_SIGN: AtomicI32 = AtomicI32::new(1);

const STICK_FULL: i32 = 32767;
const TRIGGER_THRESHOLD: i32 = -20000;
const DIRECTION_THRESHOLD: i32 = 10000;
// about 300ms at a 200Hz controller poll
const FRESHNESS_WINDOW: u32 = 60;

fn direction_from_lstick(x: i32) -> Option<i32> {
    if x > DIRECTION_THRESHOLD {
        Some(1)
    } else if x < -DIRECTION_THRESHOLD {
        Some(-1)
    } else {
        None
    }
}

fn resolve_facing_sign(s: &NpadState) -> i32 {
    if let Some(d) = direction_from_lstick(s.lstick_x) {
        return d;
    }
    // stick is neutral but was pushed recently
    if DIRECTION_FRESHNESS.load(Ordering::Relaxed) > 0 {
        return LAST_DIRECTION.load(Ordering::Relaxed);
    }
    1
}

fn process_state(s: &mut NpadState) {
    let last_uc = LAST_UPDATE_COUNT.load(Ordering::Relaxed);
    let is_new_update = s.update_count != last_uc;

    if is_new_update {
        LAST_UPDATE_COUNT.store(s.update_count, Ordering::Relaxed);

        match direction_from_lstick(s.lstick_x) {
            Some(d) => {
                LAST_DIRECTION.store(d, Ordering::Relaxed);
                DIRECTION_FRESHNESS.store(FRESHNESS_WINDOW, Ordering::Relaxed);
            }
            None => {
                let f = DIRECTION_FRESHNESS.load(Ordering::Relaxed);
                if f > 0 {
                    DIRECTION_FRESHNESS.store(f - 1, Ordering::Relaxed);
                }
            }
        }

        let phase = INJECT_PHASE.load(Ordering::Relaxed);
        if phase == 0 {
            if s.rstick_y < TRIGGER_THRESHOLD {
                CACHED_SIGN.store(resolve_facing_sign(s), Ordering::Relaxed);
                INJECT_PHASE.store(1, Ordering::Relaxed);
            }
        } else {
            let next = match phase {
                1 => 2,
                2 => 3,
                3 => 0,
                p => p,
            };
            INJECT_PHASE.store(next, Ordering::Relaxed);
        }
    }

    let phase = INJECT_PHASE.load(Ordering::Relaxed);
    if phase == 0 {
        return;
    }
    let sign = CACHED_SIGN.load(Ordering::Relaxed);
    match phase {
        1 => {
            s.lstick_x = STICK_FULL * sign;
            s.lstick_y = 0;
            s.rstick_x = 0;
            s.rstick_y = 0;
        }
        2 => {
            s.lstick_x = 0;
            s.lstick_y = -STICK_FULL;
            s.rstick_x = 0;
            s.rstick_y = 0;
        }
        3 => {
            s.lstick_x = STICK_FULL * sign;
            s.lstick_y = -STICK_FULL;
            s.rstick_x = 0;
            s.rstick_y = 0;
            s.buttons |= BUTTON_A;
        }
        _ => {}
    }
}

#[skyline::hook(replace = get_npad_full_key_state)]
unsafe fn full_key_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

#[skyline::hook(replace = get_npad_handheld_state)]
unsafe fn handheld_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

#[skyline::hook(replace = get_npad_joy_dual_state)]
unsafe fn joy_dual_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

#[skyline::hook(replace = get_npad_joy_left_state)]
unsafe fn joy_left_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

#[skyline::hook(replace = get_npad_joy_right_state)]
unsafe fn joy_right_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

#[skyline::hook(replace = get_npad_gc_state)]
unsafe fn gc_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    if !state.is_null() {
        process_state(&mut *state);
    }
}

pub fn install() {
    skyline::install_hooks!(
        full_key_hook,
        handheld_hook,
        joy_dual_hook,
        joy_left_hook,
        joy_right_hook,
        gc_hook,
    );
}
