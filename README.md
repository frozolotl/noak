# Noak
A library for reading and writing java class files.

## Example
```rust
use noak::reader::Class;

let mut class = Class::new(&bytes)?;

println!("Major Version: {}", class.version().major);
println!(" Access Flags: {:?}", class.access_flags()?);
println!("   Class Name: {}", class.this_class_name()?.display()?);
```

## Why should you use noak?
- Reading:
  - You want to parse class files without losing information.
  - You don't want to parse the entire class file.
  - Any valid class file is accepted by noak. Many invalid class files are accepted as well (this can be useful for reading heavily obfuscated code).
- Writing:
  - You want to write your own class files (limitations apply; see below).

## Why wouldn't you use noak?
Many of these issues are in the process of being resolved, but that may take some time.
- You want more than just a low-level class file reader and writer.
- Noak makes heavy use of the type system and macros. This can cause problems with your IDE or make some aspects hard to understand.
- Documentation is virtually non-existent.
- The API is very unstable.
- The code is not heavily tested.
- Reading:
  - Some code may be repetitive (e.g. retrieving values from the constant pool).
- Writing:
  - Modifying existing class files can be very tedious.
  - Not every attribute can be written at the moment ([related issue](https://gitlab.com/frozo/noak/-/issues/4))
  - Jumps above 65535 bytes may fail to be written.
  - The builder API isn't flexible enough for your use case.
  - Custom errors are quite restricted.
  - Stack Map Frames are not automatically generated.

## Alternatives
This is not an exhaustive list. The statements below may not accurately reflect reality.

Libraries written in Rust:
- [cafebabe](https://github.com/staktrace/cafebabe)
  - Parses the entire file at once.
  - Accesses the constant pool during parsing. No explicit retrieval from the user side required.
- [classreader-rs](https://github.com/Alfriadox/classreader-rs)
  - Parses the entire file at once.
  - Only supports parsing.
  - Does not support any attributes from version 53.0 and above.
  - Does not parse strings containing unpaired surrogates.
- [jbcrs](https://github.com/orasunis/jbcrs)
  - Parses the entire file at once.
  - Only supports parsing.
  - Does not support any attributes from version 53.0 and above.
  - Does not parse files containing UTF-8 constants with unpaired surrogates.
- [frappe/classfile](https://github.com/tjdetwiler/frappe)
  - Parses the entire file at once.
  - Only supports parsing.
  - Does not support any attributes from version 53.0 and above.
  - Does not parse files containing UTF-8 constants with surrogates.
- [classfile-rs](https://github.com/x4e/classfile-rs)
  - Seems to be very incomplete.
  - Supports writing as well as parsing.
  - Parses the entire file at once.
  - Reads UTF-8 constants lossily.
- [classfile-parser](https://github.com/palmr/classfile-parser)
  - Seems to be very incomplete.
  - Parses the entire file at once.
  - Only supports parsing.
- [classfmt](https://github.com/chickenbreeder/classfmt)
  - Seems to be very incomplete.
  - Parses the entire file at once.
  - Only supports parsing.
- [javabc](https://github.com/dylanmckay/javabc)
  - Seems to be very incomplete.
  - Parses the entire file at once.
  - Only supports parsing.

A small fraction of libraries written in languages that are not Rust:
- [ASM](https://asm.ow2.io/) (Java)
  - The most comprehensive and best supported class file manipulation and analysis framework.
  - Use this if noak does not suit your requirements and probably even if it does suit them.
- [javassist](https://github.com/jboss-javassist/javassist) (Java)
  - A high-level bytecode library.
- [BCEL](https://commons.apache.org/proper/commons-bcel/) (Java)

## License
This project is licensed under the [MIT](https://gitlab.com/frozo/noak/-/blob/master/LICENSE) license.
