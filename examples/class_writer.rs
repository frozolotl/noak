//! This example writes an example class.

use noak::writer::ClassWriter;
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
        .write_interface_name("java/io/Serializable")?
        .write_interface_name("TestInterface")?;

    class_writer.write_field(|writer| {
        writer
            .write_access_flags(AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL)?
            .write_name("ZERO")?
            .write_descriptor("I")?;
        Ok(())
    })?;

    class_writer.write_method(|writer| {
        writer
            .write_access_flags(AccessFlags::PUBLIC)?
            .write_name("<init>")?
            .write_descriptor("()V")?;
        Ok(())
    })?;

    let bytes = class_writer.finish()?;
    std::fs::write(path, &bytes)?;

    Ok(())
}
