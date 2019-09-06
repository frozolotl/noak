//! This example writes an example class.

use noak::writer::{cpool, ClassWriter};
use noak::mutf8::MString;
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_writer Output.class`");
    let mut class_writer = ClassWriter::new();

    let this_name_index = class_writer.insert_constant(cpool::Utf8 { content: MString::from("Test") })?;
    let this_index = class_writer.insert_constant(cpool::Class { name: this_name_index })?;

    let super_name_index = class_writer.insert_constant(cpool::Utf8 { content: MString::from("java/lang/Object") })?;
    let super_index = class_writer.insert_constant(cpool::Class { name: super_name_index })?;

    class_writer
        .write_access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .write_this_class(this_index)?
        .write_super_class(super_index)?;

    let bytes = class_writer.finish()?;
    std::fs::write(path, &bytes)?;

    Ok(())
}
