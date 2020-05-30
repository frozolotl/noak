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
    for name in class.interfaces()? {
        println!("  - {}", class.pool()?.retrieve(name?)?.name);
    }

    println!("- Fields:");
    for field in class.fields()? {
        let field = field?;
        let pool = class.pool()?;
        println!("  - {}:", pool.retrieve(field.name())?);
        println!("    - Access Flags: {:?}", field.access_flags());
        println!("    - Descriptor: {}", pool.retrieve(field.descriptor())?);
    }

    println!("- Methods:");
    for method in class.methods()? {
        let method = method?;
        let pool = class.pool()?;
        println!("  - {}:", pool.retrieve(method.name())?);
        println!("    - Access Flags: {:?}", method.access_flags());
        println!("    - Descriptor: {}", pool.retrieve(method.descriptor())?);
    }

    Ok(())
}
