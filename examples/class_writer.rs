fn main() {}
// //! This example writes an example class.

// use std::fs::File;

// use noak::writer::{cpool, ClassWriter};
// use noak::AccessFlags;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let path = std::env::args().nth(1).expect("usage: `class_writer Output.class`");

//     let mut file = File::create(path)?;
//     ClassWriter::new()
//         .version(noak::Version::V8)?
//         .access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
//         .this_class("Test")?
//         .super_class("java/lang/Object")?
//         .interfaces(|_interfaces| Ok(()))?
//         .fields(|fields| {
//             fields.begin(|field| {
//                 field
//                     .access_flags(AccessFlags::PRIVATE | AccessFlags::STATIC | AccessFlags::FINAL)?
//                     .name("SOME_FIELD")?
//                     .descriptor("Ljava/lang/String;")?
//                     .attributes(|_attributes| Ok(()))
//             })?;
//             Ok(())
//         })?
//         .methods(|methods| {
//             methods.begin(|method| {
//                 method
//                     .access_flags(AccessFlags::PUBLIC | AccessFlags::STATIC)?
//                     .name("main")?
//                     .descriptor("([Ljava/lang/String;)V")?
//                     .attributes(|attributes| {
//                         attributes.begin(|attribute| {
//                             attribute.code(|mut code| {
//                                 let (block_end, block_end_ref) = code.new_label()?;

//                                 code.max_stack(2)?
//                                     .max_locals(3)?
//                                     .instructions(|instructions| {
//                                         instructions
//                                             .aload0()?
//                                             .iconst0()?
//                                             .aaload()?
//                                             .astore1()?
//                                             .aload1()?
//                                             .ldc(cpool::String::by("no"))?
//                                             .invokevirtual(cpool::MethodRef::by(
//                                                 cpool::Class::by("java/lang/String"),
//                                                 ("equals", "(Ljava/lang/Object;)Z"),
//                                             ))?
//                                             .ifeq(block_end_ref)?
//                                             .getstatic(cpool::FieldRef::by(
//                                                 cpool::Class::by("java/lang/System"),
//                                                 ("out", "Ljava/io/PrintStream;"),
//                                             ))?
//                                             .ldc(cpool::String::by("yes"))?
//                                             .invokevirtual(cpool::MethodRef::by(
//                                                 cpool::Class::by("java/io/PrintStream"),
//                                                 ("println", "(Ljava/lang/String;)V"),
//                                             ))?
//                                             .label(block_end)?
//                                             .aload0()?
//                                             .iconst1()?
//                                             .aaload()?
//                                             .invokevirtual(cpool::MethodRef::by(
//                                                 cpool::Class::by("java/lang/String"),
//                                                 ("toUpperCase", "()Ljava/lang/String;"),
//                                             ))?
//                                             .astore2()?
//                                             .getstatic(cpool::FieldRef::by(
//                                                 cpool::Class::by("java/lang/System"),
//                                                 ("out", "Ljava/io/PrintStream;"),
//                                             ))?
//                                             .aload2()?
//                                             .invokevirtual(cpool::MethodRef::by(
//                                                 cpool::Class::by("java/io/PrintStream"),
//                                                 ("println", "(Ljava/lang/String;)V"),
//                                             ))?
//                                             .return_()?;
//                                         Ok(())
//                                     })?
//                                     .exceptions(|_exceptions| Ok(()))?
//                                     .attributes(|attributes| {
//                                         attributes.begin(|attribute| {
//                                             attribute.stack_map_table(|stack_map_table| {
//                                                 stack_map_table.append(block_end_ref, |frame| {
//                                                     frame.local(|typ| typ.object("java/lang/String"))
//                                                 })?;
//                                                 Ok(())
//                                             })
//                                         })?;
//                                         Ok(())
//                                     })
//                             })
//                         })?;
//                         Ok(())
//                     })
//             })?;
//             Ok(())
//         })?
//         .attributes(|attributes| {
//             attributes.begin(|attribute| attribute.source_file("Test.java"))?;
//             Ok(())
//         })?
//         .write_bytes_to(&mut file)?;

//     Ok(())
// }
