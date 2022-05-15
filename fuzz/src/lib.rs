#[cfg(feature = "libfuzzer")]
pub use libfuzzer_sys::fuzz_target as fuzz;

#[cfg(feature = "afl")]
#[macro_export]
macro_rules! fuzz {
    ($($tt:tt)*) => {
        fn main() {
            afl::fuzz!($($tt)*);
        }
    };
}
