//! This example writes an example class.

use std::fs::File;

use noak::writer::{cpool, ClassWriter};
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("usage: `class_writer Output.class`");

    let mut file = File::create(path)?;
    ClassWriter::new()
        .version(noak::Version::V8)?
        .access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .this_class("Test")?
        .super_class("java/lang/Object")?
        .interfaces(|interfaces| {
            interfaces.begin(|i| i.interface("Whatever"))?;
            Ok(())
        })?
        .fields(|fields| {
            fields.begin(|field| {
                field
                    .access_flags(AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL)?
                    .name("SOME_FIELD")?
                    .descriptor("Ljava/lang/String;")?
                    .attributes(|_attributes| Ok(()))
            })?;
            Ok(())
        })?
        .methods(|methods| {
            methods.begin(|method| {
                method
                    .access_flags(AccessFlags::PUBLIC | AccessFlags::STATIC)?
                    .name("main")?
                    .descriptor("([Ljava/lang/String;)V")?
                    .attributes(|attributes| {
                        attributes.begin(|attribute| {
                            attribute.code(|code| {
                                code.max_stack(6)?
                                    .max_locals(2)?
                                    .instructions(|instructions| {
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
                                    })?
                                    .exceptions(|_exceptions| Ok(()))?
                                    .attributes(|_attributes| Ok(()))
                            })
                        })?;
                        Ok(())
                    })
            })?;
            Ok(())
        })?
        .attributes(|attributes| {
            attributes.begin(|attribute| attribute.source_file("Test.java"))?;
            Ok(())
        })?
        .write_bytes_to(&mut file)?;

    Ok(())
}
