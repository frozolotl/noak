//! This example writes an example class.

use noak::writer::{cpool, ClassWriter, FieldWriter, FieldWriterState, CountedWriter};
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("usage: `class_writer Output.class`");

    let bytes = ClassWriter::new()
        .version(noak::Version::V8)?
        .access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .this_class("Test")?
        .super_class("java/lang/Object")?
        .interfaces(|interfaces| Ok(()))?
        .fields(|fields| {
            fields.begin(|field| {
                Ok(field
                    .access_flags(AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL)?
                    .name("SOME_FIELD")?
                    .descriptor("Ljava/lang/String;")?)
            })?;
            Ok(())
        })?
        .methods(|methods| {
            methods.begin(|method| {
                Ok(method
                    .access_flags(AccessFlags::PUBLIC | AccessFlags::STATIC)?
                    .name("main")?
                    .descriptor("([Ljava/lang/String;)V")?
                    .attributes(|attributes| {
                        attributes.code().begin(|code| {
                            Ok(code.max_stack(6)?.max_locals(2)?.instructions(|instructions| {
                                instructions
                                    .getstatic(cpool::FieldRef::by(
                                        "java/lang/System",
                                        ("out", "Ljava/io/PrintStream;"),
                                    ))?
                                    .ldc(cpool::String::by("Hello, World!"))?
                                    .invokevirtual(cpool::MethodRef::by(
                                        "java/io/PrintStream",
                                        ("println", "(Ljava/lang/String;)V"),
                                    ))?
                                    .return_()?;
                                Ok(())
                            })?)
                        })?;
                        Ok(())
                    })?)
            })?;
            Ok(())
        })?
        .attributes(|attributes| {
            attributes.source_file("Test.java")?;
            Ok(())
        })?
        .finish()?;

    std::fs::write(path, &bytes)?;

    Ok(())
}
