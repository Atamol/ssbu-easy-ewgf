use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU8, Ordering};

use crate::hid::{
    get_npad_full_key_state, get_npad_gc_state, get_npad_handheld_state,
    get_npad_joy_dual_state, get_npad_joy_left_state, get_npad_joy_right_state,
    NpadState, BUTTON_A,
};

const ZERO_U8: AtomicU8 = AtomicU8::new(0);
const FALSE_BOOL: AtomicBool = AtomicBool::new(false);
const NEG_ONE_I64: AtomicI64 = AtomicI64::new(-1);
const ONE_I32: AtomicI32 = AtomicI32::new(1);
const ZERO_U32: AtomicU32 = AtomicU32::new(0);

static INJECT_PHASE: [AtomicU8; 8] = [ZERO_U8; 8];
static PHASE_UPDATES: [AtomicU8; 8] = [ZERO_U8; 8];
static LAST_UPDATE_COUNT: [AtomicI64; 8] = [NEG_ONE_I64; 8];
static LAST_DIRECTION: [AtomicI32; 8] = [ONE_I32; 8];
static DIRECTION_FRESHNESS: [AtomicU32; 8] = [ZERO_U32; 8];
static CACHED_SIGN: [AtomicI32; 8] = [ONE_I32; 8];
static PREV_CSTICK_LOW: [AtomicBool; 8] = [FALSE_BOOL; 8];

const STICK_FULL: i32 = 32767;
const STICK_WEAK: i32 = 8500;
const TRIGGER_THRESHOLD: i32 = -20000;
const DIRECTION_THRESHOLD: i32 = 10000;
const PHASE_UPDATES_LEN: u8 = 4;
const FRESHNESS_WINDOW: u32 = 60;

fn npad_to_entry(npad_id: u32) -> Option<usize> {
    match npad_id {
        0..=7 => Some(npad_id as usize),
        0x20 => Some(0), // handheld drives player 1
        _ => None,
    }
}

fn direction_from_lstick(x: i32) -> Option<i32> {
    if x > DIRECTION_THRESHOLD {
        Some(1)
    } else if x < -DIRECTION_THRESHOLD {
        Some(-1)
    } else {
        None
    }
}

fn resolve_facing_sign(s: &NpadState, entry: usize) -> i32 {
    if let Some(d) = direction_from_lstick(s.lstick_x) {
        return d;
    }
    if let Some(sign) = crate::state::entry_facing_sign(entry as i32) {
        return sign;
    }
    if DIRECTION_FRESHNESS[entry].load(Ordering::Relaxed) > 0 {
        return LAST_DIRECTION[entry].load(Ordering::Relaxed);
    }
    1
}

fn process_state(s: &mut NpadState, entry: usize) {
    let last_uc = LAST_UPDATE_COUNT[entry].load(Ordering::Relaxed);
    let is_new_update = s.update_count != last_uc;

    if is_new_update {
        LAST_UPDATE_COUNT[entry].store(s.update_count, Ordering::Relaxed);

        match direction_from_lstick(s.lstick_x) {
            Some(d) => {
                LAST_DIRECTION[entry].store(d, Ordering::Relaxed);
                DIRECTION_FRESHNESS[entry].store(FRESHNESS_WINDOW, Ordering::Relaxed);
            }
            None => {
                let f = DIRECTION_FRESHNESS[entry].load(Ordering::Relaxed);
                if f > 0 {
                    DIRECTION_FRESHNESS[entry].store(f - 1, Ordering::Relaxed);
                }
            }
        }

        // Fire once per press, not every poll while it's held
        let now_low = s.rstick_y < TRIGGER_THRESHOLD;
        let prev_low = PREV_CSTICK_LOW[entry].swap(now_low, Ordering::Relaxed);
        let pressed = now_low && !prev_low;

        let phase = INJECT_PHASE[entry].load(Ordering::Relaxed);
        if phase == 0 {
            if pressed && crate::state::is_entry_kazuya(entry as i32) {
                CACHED_SIGN[entry].store(resolve_facing_sign(s, entry), Ordering::Relaxed);
                INJECT_PHASE[entry].store(1, Ordering::Relaxed);
                PHASE_UPDATES[entry].store(0, Ordering::Relaxed);
            }
        } else {
            let held = PHASE_UPDATES[entry].fetch_add(1, Ordering::Relaxed) + 1;
            if held >= PHASE_UPDATES_LEN {
                let next = match phase {
                    1 => 2,
                    2 => 3,
                    3 => 4,
                    4 => 0,
                    p => p,
                };
                INJECT_PHASE[entry].store(next, Ordering::Relaxed);
                PHASE_UPDATES[entry].store(0, Ordering::Relaxed);
            }
        }
    }

    let phase = INJECT_PHASE[entry].load(Ordering::Relaxed);
    if phase == 0 {
        return;
    }
    let sign = CACHED_SIGN[entry].load(Ordering::Relaxed);
    match phase {
        1 => {
            s.lstick_x = STICK_WEAK * sign;
            s.lstick_y = 0;
            s.rstick_x = 0;
            s.rstick_y = 0;
        }
        2 => {
            s.lstick_x = 0;
            s.lstick_y = -STICK_WEAK;
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
        4 => {
            s.lstick_x = STICK_FULL * sign;
            s.lstick_y = -STICK_FULL;
            s.rstick_x = 0;
            s.rstick_y = 0;
        }
        _ => {}
    }
}

unsafe fn dispatch(state: *mut NpadState, id: *const u32) {
    if state.is_null() || id.is_null() {
        return;
    }
    let Some(entry) = npad_to_entry(*id) else {
        return;
    };
    process_state(&mut *state, entry);
}

#[skyline::hook(replace = get_npad_full_key_state)]
unsafe fn full_key_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

#[skyline::hook(replace = get_npad_handheld_state)]
unsafe fn handheld_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

#[skyline::hook(replace = get_npad_joy_dual_state)]
unsafe fn joy_dual_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

#[skyline::hook(replace = get_npad_joy_left_state)]
unsafe fn joy_left_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

#[skyline::hook(replace = get_npad_joy_right_state)]
unsafe fn joy_right_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

#[skyline::hook(replace = get_npad_gc_state)]
unsafe fn gc_hook(state: *mut NpadState, id: *const u32) {
    call_original!(state, id);
    dispatch(state, id);
}

pub fn install() {
    crate::state::init();
    skyline::install_hooks!(
        full_key_hook,
        handheld_hook,
        joy_dual_hook,
        joy_left_hook,
        joy_right_hook,
        gc_hook,
    );
}
