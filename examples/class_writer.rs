//! This example writes an example class.

use noak::writer::{cpool, ClassWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: `class_writer Output.class`");
    let mut class_writer = ClassWriter::new();
    let index = class_writer.insert_constant(cpool::Integer { value: 0 })?;
    println!("Index: {:?}", index);

    let bytes = class_writer.finish()?;
    std::fs::write(path, &bytes)?;

    Ok(())
}
