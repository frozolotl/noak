//! This example writes an example class.

use noak::writer::ClassWriter;
use noak::AccessFlags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_writer Output.class`");
    let mut class_writer = ClassWriter::new();

    class_writer
        .write_version(noak::Version {
            minor: 0,
            major: 52,
        })?
        .write_access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
        .write_this_class("Test")?
        .write_super_class("java/lang/Object")?
        .write_interfaces(|writer| {
            writer.write_simple("java/io/Serializable")?;
            Ok(())
        })?
        .write_fields(|writer| {
            writer.write(|writer| {
                writer
                    .write_access_flags(
                        AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL,
                    )?
                    .write_name("ZERO")?
                    .write_descriptor("Ljava/lang/String;")?
                    .write_attributes(|writer| {
                        writer.write(|writer| {
                            writer.write_constant_value("Hello World")?;
                            Ok(())
                        })?;
                        Ok(())
                    })?;
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
                        writer
                            .write(|writer| {
                                writer.write_deprecated()?;
                                Ok(())
                            })?
                            .write(|writer| {
                                writer.write_exceptions(|writer| {
                                    writer
                                        .write_simple("java/io/IOException")?
                                        .write_simple("java/lang/RuntimeException")?;
                                    Ok(())
                                })?;
                                Ok(())
                            })?
                            .write(|writer| {
                                writer.write_code(|writer| {
                                    writer
                                        .write_max_stack(0)?
                                        .write_max_locals(0)?
                                        .write_instructions(|writer| {
                                            writer.write_return()?;

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
            writer.write(|writer| {
                writer
                    .write_access_flags(AccessFlags::PUBLIC)?
                    .write_name("addOne")?
                    .write_descriptor("(I)I")?
                    .write_attributes(|writer| {
                        writer.write(|writer| {
                            writer.write_code(|writer| {
                                writer
                                    .write_max_stack(2)?
                                    .write_max_locals(1)?
                                    .write_instructions(|writer| {
                                        let (def, def_ref) = writer.new_label()?;
                                        let (zero, zero_ref) = writer.new_label()?;
                                        let (one, one_ref) = writer.new_label()?;
                                        let (two, two_ref) = writer.new_label()?;

                                        writer.write_iload1()?.write_tableswitch(|writer| {
                                            writer
                                                .write_default(def_ref)?
                                                .write_low(0)?
                                                .write_high(2)?
                                                .write_jump(zero_ref)?
                                                .write_jump(one_ref)?
                                                .write_jump(two_ref)?;
                                            Ok(())
                                        })?;

                                        writer
                                            .write_label(zero)?
                                            .write_iconst1()?
                                            .write_ireturn()?;

                                        writer
                                            .write_label(one)?
                                            .write_iconst2()?
                                            .write_ireturn()?;

                                        writer
                                            .write_label(two)?
                                            .write_iconst3()?
                                            .write_ireturn()?;

                                        writer
                                            .write_label(def)?
                                            .write_bipush(1)?
                                            .write_iload1()?
                                            .write_iadd()?
                                            .write_ireturn()?;

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
