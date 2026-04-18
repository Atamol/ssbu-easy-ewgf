#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct NpadState {
    pub update_count: i64,
    pub buttons: u64,
    pub lstick_x: i32,
    pub lstick_y: i32,
    pub rstick_x: i32,
    pub rstick_y: i32,
    pub flags: u32,
}

pub const BUTTON_A: u64 = 1 << 0;

extern "C" {
    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_16NpadFullKeyStateERKj"]
    pub fn get_npad_full_key_state(state: *mut NpadState, id: *const u32);

    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_17NpadHandheldStateERKj"]
    pub fn get_npad_handheld_state(state: *mut NpadState, id: *const u32);

    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyDualStateERKj"]
    pub fn get_npad_joy_dual_state(state: *mut NpadState, id: *const u32);

    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyLeftStateERKj"]
    pub fn get_npad_joy_left_state(state: *mut NpadState, id: *const u32);

    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_17NpadJoyRightStateERKj"]
    pub fn get_npad_joy_right_state(state: *mut NpadState, id: *const u32);

    #[link_name = "_ZN2nn3hid12GetNpadStateEPNS0_11NpadGcStateERKj"]
    pub fn get_npad_gc_state(state: *mut NpadState, id: *const u32);
}
