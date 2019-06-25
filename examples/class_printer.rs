use noak::error::DecodeError;
use noak::reader::Class;

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
    let mut class = Class::new(&bytes)?;

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
    }

    println!("- Methods:");
    for method in class.method_indices()? {
        let name = class.pool()?.get(method.name())?.content;
        let descriptor = class.pool()?.get(method.descriptor())?.content;
        println!("  - {}:", name);
        println!("    - Access Flags: {:?}", method.access_flags());
        println!("    - Descriptor: {}", descriptor);
    }

    Ok(())
}

