use noak::error::DecodeError;
use noak::reader::Class;

fn main() -> Result<(), DecodeError> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_printer MyClass.class`");
    let bytes = std::fs::read(path).expect("could not read file");

    let mut class = Class::new(&bytes)?;

    let version = class.version();
    println!("   Major Version: {}", version.major);
    println!("   Minor Version: {}", version.minor);
    println!("    Access Flags: {:?}", class.access_flags()?);
    println!("      Class Name: {}", class.this_class_name()?);
    println!("Super Class Name: {}", class.super_class_name()?);

    Ok(())
}
