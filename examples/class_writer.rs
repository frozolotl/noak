//! This example writes an example class.

use noak::writer::{cpool, ClassWriter};
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_writer Output.class`");
    let mut class_writer = ClassWriter::new();

    class_writer
        .write_version(noak::Version::V8)?
        .write_access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .write_this_class("Test")?
        .write_super_class("java/lang/Object")?
        .write_fields(|writer| {
            writer.write(|writer| {
                writer
                    .write_access_flags(
                        AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL,
                    )?
                    .write_name("SOME_FIELD")?
                    .write_descriptor("Ljava/lang/String;")?;
                Ok(())
            })?;
            Ok(())
        })?
        .write_methods(|writer| {
            writer.write(|writer| {
                writer
                    .write_access_flags(AccessFlags::PUBLIC | AccessFlags::STATIC)?
                    .write_name("main")?
                    .write_descriptor("([Ljava/lang/String;)V")?
                    .write_attributes(|writer| {
                        writer.write(|writer| {
                            writer.write_code(|writer| {
                                writer
                                    .write_max_stack(6)?
                                    .write_max_locals(2)?
                                    .write_instructions(|writer| {
                                        writer
                                            .write_getstatic(cpool::FieldRef::by(
                                                "java/lang/System",
                                                ("out", "Ljava/io/PrintStream;"),
                                            ))?
                                            .write_ldc(cpool::String::by("Hello, World!"))?
                                            .write_invokevirtual(cpool::MethodRef::by(
                                                "java/io/PrintStream",
                                                ("println", "(Ljava/lang/String;)V"),
                                            ))?
                                            .write_return()?;
                                        Ok(())
                                    })?;
                                Ok(())
                            })?;
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
