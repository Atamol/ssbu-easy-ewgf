use once_cell::sync::Lazy;

// final_input_mapping prologue on patch 13.x, borrowed from UTM
static NEEDLE_FIM: &[u8] = &[
    0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06, 0xa9,
    0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77, 0x00, 0x54,
];

fn text_slice() -> &'static [u8] {
    unsafe {
        let start = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u8;
        let end = skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as *const u8;
        let length = end.offset_from(start) as usize;
        std::slice::from_raw_parts(start, length)
    }
}

fn find_unique(needle: &[u8]) -> Option<usize> {
    let haystack = text_slice();
    let first = memchr::memmem::find(haystack, needle)?;
    if memchr::memmem::rfind(haystack, needle)? != first {
        return None;
    }
    Some(first)
}

pub static OFFSET_FIM: Lazy<usize> =
    Lazy::new(|| find_unique(NEEDLE_FIM).expect("ssbu-easy-ewgf: final_input_mapping not found"));
