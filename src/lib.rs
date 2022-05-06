#![warn(
    clippy::cargo,
    missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    unused_qualifications
)]

pub mod descriptor;
pub mod error;
mod header;
pub mod mutf8;
pub mod reader;
pub mod writer;

pub use header::*;
