#![cfg_attr(feature = "libfuzzer", no_main)]
use noak_fuzz::fuzz;

use noak::reader::Class;

fuzz!(|data: &[u8]| {
    if let Ok(mut class) = Class::new(&data) {
        let _ = class.pool();
    }
});
