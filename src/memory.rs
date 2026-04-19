use core::arch::asm;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct MemoryInfo {
    base_address: u64,
    size: u64,
    memory_type: u32,
    memory_attribute: u32,
    permission: u32,
    ipc_ref_count: u32,
    device_ref_count: u32,
    _padding: u32,
}

const PERM_READ: u32 = 1;

unsafe fn query_memory(addr: u64) -> MemoryInfo {
    let mut info: MemoryInfo = core::mem::zeroed();
    let _result: u64;
    let _page_info: u64;
    asm!(
        "svc 0x6",
        inout("x0") &mut info as *mut MemoryInfo => _result,
        inout("x1") addr => _page_info,
        options(nostack),
    );
    info
}

// Returns true when the address is in a mapped, readable page.
pub fn is_readable<T>(ptr: *const T) -> bool {
    if ptr.is_null() {
        return false;
    }
    let info = unsafe { query_memory(ptr as u64) };
    info.memory_type != 0 && (info.permission & PERM_READ) != 0
}
