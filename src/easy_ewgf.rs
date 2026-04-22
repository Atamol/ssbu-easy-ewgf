use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};

use crate::offsets::OFFSET_FIM;

const ZERO_U8: AtomicU8 = AtomicU8::new(0);
const FALSE_BOOL: AtomicBool = AtomicBool::new(false);
const ONE_I32: AtomicI32 = AtomicI32::new(1);

static INJECT_PHASE: [AtomicU8; 8] = [ZERO_U8; 8];
static CACHED_SIGN: [AtomicI32; 8] = [ONE_I32; 8];
static PREV_CSTICK_DOWN: [AtomicBool; 8] = [FALSE_BOOL; 8];

const BTN_ATTACK: u32 = 1 << 0;
const BTN_CSTICK_ON: u32 = 1 << 7;

const STICK_MAX: i8 = 127;
// Above every C-stick deadzone I've seen (~0.63 of full range)
const CSTICK_TRIGGER: i8 = -80;

// Written out by final_input_mapping, command detection reads this
#[repr(C)]
#[derive(Copy, Clone)]
struct MappedInput {
    buttons: u32,
    stick_x: i8,
    stick_y: i8,
    cstick_x: i8,
    cstick_y: i8,
}

const NEUTRAL_INPUT: MappedInput = MappedInput {
    buttons: 0,
    stick_x: 0,
    stick_y: 0,
    cstick_x: 0,
    cstick_y: 0,
};

fn cstick_down(m: &MappedInput) -> bool {
    // Tilt-stick mode keeps C-stick in cstick_y
    // Smash-stick mode routes it through stick_y with BTN_CSTICK_ON set
    if m.cstick_y < CSTICK_TRIGGER {
        return true;
    }
    if (m.buttons & BTN_CSTICK_ON) != 0 && m.stick_y < CSTICK_TRIGGER {
        return true;
    }
    false
}

fn phase_input(phase: u8, sign: i32) -> MappedInput {
    let sx = |mag: i8| (mag as i32 * sign) as i8;
    match phase {
        1 => MappedInput { stick_x: sx(STICK_MAX), ..NEUTRAL_INPUT },
        2 => MappedInput { stick_y: -STICK_MAX, ..NEUTRAL_INPUT },
        3 => MappedInput {
            buttons: BTN_ATTACK,
            stick_x: sx(STICK_MAX),
            stick_y: -STICK_MAX,
            ..NEUTRAL_INPUT
        },
        4 => MappedInput {
            stick_x: sx(STICK_MAX),
            stick_y: -STICK_MAX,
            ..NEUTRAL_INPUT
        },
        _ => NEUTRAL_INPUT,
    }
}

fn resolve_facing_sign(entry: usize) -> i32 {
    crate::state::entry_facing_sign(entry as i32).unwrap_or(1)
}

unsafe fn process_entry(out: *mut MappedInput, entry: usize) {
    let mapped = &mut *out;
    let now_down = cstick_down(mapped);
    let prev_down = PREV_CSTICK_DOWN[entry].swap(now_down, Ordering::Relaxed);
    let pressed = now_down && !prev_down;

    let phase = INJECT_PHASE[entry].load(Ordering::Relaxed);

    if phase == 0 {
        if pressed && crate::state::is_entry_kazuya(entry as i32) {
            let sign = resolve_facing_sign(entry);
            CACHED_SIGN[entry].store(sign, Ordering::Relaxed);
            *mapped = phase_input(1, sign);
            INJECT_PHASE[entry].store(2, Ordering::Relaxed);
        }
    } else {
        let sign = CACHED_SIGN[entry].load(Ordering::Relaxed);
        *mapped = phase_input(phase, sign);
        let next = if phase >= 4 { 0 } else { phase + 1 };
        INJECT_PHASE[entry].store(next, Ordering::Relaxed);
    }
}

// Runs once per player per frame. Let the original map normally, then overwrite Kazuya's slot
#[skyline::hook(offset = *OFFSET_FIM)]
unsafe fn final_input_mapping_hook(
    mappings: *mut u8,
    player_idx: i32,
    out: *mut MappedInput,
    controller_struct: *mut u8,
    arg: bool,
) {
    call_original!(mappings, player_idx, out, controller_struct, arg);
    if out.is_null() || !(0..8).contains(&player_idx) {
        return;
    }
    process_entry(out, player_idx as usize);
}

pub fn install() {
    crate::state::init();
    skyline::install_hooks!(final_input_mapping_hook);
}
