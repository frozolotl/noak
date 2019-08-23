use noak::error::DecodeError;
use noak::reader::{cpool, Attributes, Class};
use std::ops::Range;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_printer MyClass.class`");
    let bytes = std::fs::read(&path).expect("could not read file");
    if let Err(err) = print(&bytes) {
        eprintln!("Error in file `{}`: {}", path, err);
        std::process::exit(1);
    }
}

fn print(bytes: &[u8]) -> Result<(), DecodeError> {
    let mut class = Class::new(bytes)?;

    let version = class.version();
    println!("- Major Version: {}", version.major);
    println!("- Minor Version: {}", version.minor);
    println!("- Access Flags: {:?}", class.access_flags()?);
    println!("- Class Name: {}", class.this_class_name()?);
    if let Some(name) = class.super_class_name()? {
        println!("- Super Class Name: {}", name);
    }

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
                    for res in code.raw_instructions() {
                        let (idx, instruction) = res?;
                        println!("{}      {}. {:?}", indent, idx, instruction);
                    }

                    println!("{}    - Exception Handlers:", indent);
                    for handler in code.exception_handlers() {
                        let range = handler.range();
                        println!("{}      - Exception Handler:", indent);
                        println!("{}        - Start: {}", indent, range.start);
                        println!("{}        - End: {}", indent, range.end);
                        println!("{}        - Handler: {}", indent, handler.handler());
                        println!("{}        - Catch Type: {}", indent, handler.catch_type());
                    }

                    print_attributes(indentation + 3, pool, code.attribute_indices())?;
                }
                ConstantValue(source_file) => {
                    let value = pool.get(source_file.value())?;
                    println!("{}    - Constant Value: {:?}", indent, value);
                }
                Deprecated => {}
                EnclosingMethod(enclosing) => {
                    let class = pool.get(enclosing.class())?.name;
                    let class = pool.get(class)?.content;
                    let method = pool.get(enclosing.method())?;
                    let method_name = pool.get(method.name)?.content;
                    let method_desc = pool.get(method.descriptor)?.content;
                    println!("{}      - Enclosing Method:", indent);
                    println!("{}        - Class: {}", indent, class);
                    println!("{}        - Method Name: {}", indent, method_name);
                    println!("{}        - Method Descriptor: {}", indent, method_desc);
                }
                Exceptions(exceptions) => {
                    println!("{}      - Exceptions:", indent);
                    for exception in exceptions.iter() {
                        let class = pool.get(exception)?.name;
                        let class = pool.get(class)?.content;
                        println!("{}        - {}", indent, class);
                    }
                }
                LineNumberTable(line_number_table) => {
                    println!("{}      - Line Number Table:", indent);
                    for line in line_number_table.iter() {
                        println!("{}        {}: {}", indent, line.start(), line.line_number());
                    }
                }
                LocalVariableTable(local_variable_table) => {
                    println!("{}      - Local Variable Table:", indent);
                    for local in local_variable_table.iter() {
                        let Range { start, end } = local.range();
                        let name = pool.get(local.name())?.content;
                        let descriptor = pool.get(local.descriptor())?.content;
                        println!("{}        {}…{}: {} ({})", indent, start, end, name, descriptor);
                    }
                }
                NestHost(nest_host) => {
                    let class = pool.get(nest_host.host_class())?.name;
                    let content = pool.get(class)?.content;
                    println!("{}    - Nest Host: {}", indent, content);
                }
                Signature(signature) => {
                    let signature = pool.get(signature.signature())?.content;
                    println!("{}    - Signature: {}", indent, signature);
                }
                SourceDebugExtension(content) => {
                    println!("{}    - Source Debug Extension: {}", indent, content);
                }
                SourceFile(source_file) => {
                    let source = pool.get(source_file.source_file())?.content;
                    println!("{}    - Source File: {}", indent, source);
                }
                Synthetic => {}
            }
        }
    }

    Ok(())
}
