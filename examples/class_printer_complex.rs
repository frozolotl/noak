use noak::error::DecodeError;
use noak::reader::{cpool, Attributes, Class};

fn main() -> Result<(), DecodeError> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_printer MyClass.class`");
    let bytes = std::fs::read(path).expect("could not read file");

    let mut class = Class::new(&bytes)?;

    let version = class.version();
    println!("- Major Version: {}", version.major);
    println!("- Minor Version: {}", version.minor);
    println!("- Access Flags: {:?}", class.access_flags()?);
    println!("- Class Name: {}", class.this_class_name()?);
    println!("- Super Class Name: {}", class.super_class_name()?);

    println!("- Interfaces:");
    for name in class.interface_names()? {
        println!("  - {}", name);
    }

    println!("- Fields:");
    for field in class.field_indices()? {
        let name = class.pool()?.get(field.name())?.content;
        let descriptor = class.pool()?.get(field.descriptor())?.content;
        println!("  - {}:", name);
        println!("    - Access Flags: {:?}", field.access_flags());
        println!("    - Descriptor: {}", descriptor);

        print_attributes(2, class.pool()?, field.attribute_indices())?;
    }

    println!("- Methods:");
    for method in class.method_indices()? {
        let name = class.pool()?.get(method.name())?.content;
        let descriptor = class.pool()?.get(method.descriptor())?.content;
        println!("  - {}:", name);
        println!("    - Access Flags: {:?}", method.access_flags());
        println!("    - Descriptor: {}", descriptor);

        print_attributes(2, class.pool()?, method.attribute_indices())?;
    }

    let attrs = class.attribute_indices()?;
    print_attributes(0, class.pool()?, attrs)?;

    Ok(())
}

fn print_attributes(
    indentation: usize,
    pool: &cpool::ConstantPool,
    attributes: Attributes,
) -> Result<(), DecodeError> {
    let indent = "  ".repeat(indentation);
    println!("{}- Attributes:", indent);
    for attr in attributes {
        let name = pool.get(attr.name)?.content;
        println!("{}  - {}", indent, name);

        if let Ok(content) = attr.read_content(pool) {
            use noak::reader::attributes::AttributeContent::*;
            match content {
                Code(code) => {
                    println!("{}    - Max Stack: {}", indent, code.max_stack());
                    println!("{}    - Max Locals: {}", indent, code.max_locals());
                    println!("{}    - Instructions", indent);
                    for (idx, instruction) in code.raw_instructions() {
                        println!("{}      {}. {:?}", indent, idx, instruction);
                    }
                }
                ConstantValue(source_file) => {
                    let value = pool.get(source_file.value())?;
                    println!("{}    - {:?}", indent, value);
                }
                SourceFile(source_file) => {
                    let source = pool.get(source_file.source_file())?.content;
                    println!("{}    - {}", indent, source);
                }
                SourceDebugExtension(content) => {
                    println!("{}    - {}", indent, content);
                }
                Deprecated => {}
                Synthetic => {}
            }
        }
    }

    Ok(())
}
