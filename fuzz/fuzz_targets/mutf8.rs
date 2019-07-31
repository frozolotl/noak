#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate noak;

use noak::mutf8::MStr;
use std::char;

fuzz_target!(|data: &[u8]| {
    if let Ok(mstr) = MStr::from_bytes(data) {
        for c in mstr.chars() {
            char::from_u32(c as u32).expect("MStr::from_bytes(data) probably accepted an invalid input string");
        }
    }
});
