//! This example writes an example class.

use noak::writer::{ClassWriter, cpool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut class_writer = ClassWriter::new();
    let index = class_writer.insert_constant(cpool::Integer { value: 0 })?;
    println!("Index: {:?}", index);

    let bytes = class_writer.finish()?;
    std::fs::write("Output.class", &bytes)?;

    Ok(())
}
