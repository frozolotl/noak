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
        .write_this_class("Test")?
        .write_super_class("java/lang/Object")?
        .write_interfaces(|writer| {
            writer.write(|writer| {
                writer.write_interface("java/io/Serializable")?;
                Ok(())
            })?;
            Ok(())
        })?
        .write_fields(|writer| {
            writer.write(|writer| {
                writer
                    .write_access_flags(
                        AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL,
                    )?
                    .write_name("ZERO")?
                    .write_descriptor("I")?;
                Ok(())
            })?;
            Ok(())
        })?
        .write_methods(|writer| {
            writer.write(|writer| {
                writer
                    .write_access_flags(AccessFlags::PUBLIC)?
                    .write_name("<init>")?
                    .write_descriptor("()V")?
                    .write_attributes(|writer| {
                        writer.write(|writer| {
                            writer.write_deprecated()?;
                            Ok(())
                        })?;
                        Ok(())
                    })?;
                Ok(())
            })?;
            Ok(())
        })?
        .write_attributes(|writer| {
            writer.write(|writer| {
                writer.write_source_file("Test.java")?;
                Ok(())
            })?;
            Ok(())
        })?;

    let bytes = class_writer.finish()?;
    std::fs::write(path, &bytes)?;

    Ok(())
}
