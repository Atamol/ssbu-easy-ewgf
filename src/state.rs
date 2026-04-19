use std::sync::atomic::{AtomicUsize, Ordering};

use skyline::nn::ro::LookupSymbol;
use smash::app::lua_bind::{
    FighterEntry as FighterEntryFns, FighterManager, PostureModule,
};
use smash::app::{
    self, sv_battle_object, BattleObjectModuleAccessor, FighterEntry, FighterEntryID,
};

const FIGHTER_KIND_DEMON: i32 = 0x5C;

// Looked up at init; the game writes the singleton pointer later
static FIGHTER_MANAGER_ADDR: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    unsafe {
        let mut addr: usize = 0;
        let sym = b"_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\0";
        LookupSymbol(&mut addr, sym.as_ptr());
        FIGHTER_MANAGER_ADDR.store(addr, Ordering::Relaxed);
    }
}

fn read_manager() -> Option<*mut app::FighterManager> {
    let addr = FIGHTER_MANAGER_ADDR.load(Ordering::Relaxed);
    if addr == 0 {
        return None;
    }
    let manager = unsafe { *(addr as *const *mut app::FighterManager) };
    if manager.is_null() {
        None
    } else {
        Some(manager)
    }
}

fn entry_kazuya_boma(entry_id: i32) -> Option<*mut BattleObjectModuleAccessor> {
    if !(0..=7).contains(&entry_id) {
        return None;
    }
    let manager = read_manager()?;
    let entry = unsafe {
        FighterManager::get_fighter_entry(manager, FighterEntryID(entry_id))
    } as *mut FighterEntry;
    if entry.is_null() {
        return None;
    }
    let battle_object_id =
        unsafe { FighterEntryFns::current_fighter_id(entry) } as u32;
    // Top 4 bits = object category; fighters are 0
    if battle_object_id >> 28 != 0 {
        return None;
    }
    if unsafe { sv_battle_object::kind(battle_object_id) } != FIGHTER_KIND_DEMON {
        return None;
    }
    let boma = unsafe { sv_battle_object::module_accessor(battle_object_id) };
    if boma.is_null() {
        None
    } else {
        Some(boma)
    }
}

pub fn is_entry_kazuya(entry_id: i32) -> bool {
    entry_kazuya_boma(entry_id).is_some()
}

pub fn entry_facing_sign(entry_id: i32) -> Option<i32> {
    let boma = entry_kazuya_boma(entry_id)?;
    let lr = unsafe { PostureModule::lr(boma) };
    Some(if lr < 0.0 { -1 } else { 1 })
}
