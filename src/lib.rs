#![forbid(non_ascii_idents)]
#![warn(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_debug_implementations,
    noop_method_call,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications
)]
#![warn(
    clippy::allow_attributes_without_reason,
    clippy::cargo,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::empty_structs_with_brackets,
    clippy::float_cmp,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::mod_module_files,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::separated_literal_suffix,
    clippy::string_to_string,
    clippy::todo,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::use_debug
)]

pub mod descriptor;
pub mod error;
mod header;
pub mod mutf8;
pub mod reader;
pub mod writer;

pub use header::{AccessFlags, Version};
pub use mutf8::{MStr, MString};
