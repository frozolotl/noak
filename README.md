# Noak
A library for reading and writing java bytecode fast.

# Example
```rust
use noak::error::DecodeError;
use noak::reader::Class;

let mut class = Class::new(&bytes)?;

let version = class.version();
println!("Major Version: {}", version.major);
println!("Minor Version: {}", version.minor);
println!(" Access Flags: {:?}", class.access_flags()?);
println!("   Class Name: {}", class.this_class_name()?.display()?);
```
