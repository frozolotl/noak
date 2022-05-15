#![no_main]
use libfuzzer_sys::fuzz_target;

use noak::mutf8::MStr;

fuzz_target!(|data: &[u8]| {
    if let Ok(mstr) = MStr::from_bytes(data) {
        for c in mstr.chars().filter_map(|o| o) {
            char::from_u32(c as u32).expect("MStr::from_bytes(data) probably accepted an invalid input string");
        }

        for c in mstr.chars().rev().filter_map(|o| o) {
            char::from_u32(c as u32).expect("MStr::from_bytes(data) probably accepted an invalid input string");
        }
    }
});
