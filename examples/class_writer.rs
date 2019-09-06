//! This example writes an example class.

use noak::mutf8::MString;
use noak::writer::{cpool, ClassWriter};
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_writer Output.class`");
    let mut class_writer = ClassWriter::new();

    class_writer
        .write_access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .write_this_class_name("Test")?
        .write_super_class_name("java/lang/Object")?
        .write_interface_name("java/io/Serializable")?;

    let bytes = class_writer.finish()?;
    std::fs::write(path, &bytes)?;

    Ok(())
}
