use crate::writer::encoding::*;
use crate::error::*;
use crate::header::Version;

#[derive(Clone)]
pub struct ClassWriter {
    encoder: VecEncoder,
    level: WriteLevel,
}

impl ClassWriter {
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::with_capacity(capacity),
            level: WriteLevel::Start,
        }
    }

    pub fn write_version(&mut self, version: Version) -> Result<&mut ClassWriter, EncodeError> {
        if self.level == WriteLevel::Start {
            self.encoder.write(0xCAFEBABEu32)?;
            self.encoder.write(version.minor)?;
            self.encoder.write(version.major)?;
            self.level = WriteLevel::ConstantPool;
        } else {
            // it starts at 4 as 0xCAFEBABE was already written
            let mut encoder = self.encoder.replacing(Position::new(4));
            encoder.write(version.minor)?;
            encoder.write(version.major)?;
        }
        Ok(self)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.encoder.as_bytes()
    }

    pub fn finish(self) -> Vec<u8> {
        self.encoder.into_inner()
    }
}

/// How much of the class is already written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteLevel {
    // Version numbers
    Start,
    ConstantPool,
    // Access Flags, Class Name, Super Class
    Info,
    // The field table
    Fields,
    // The method table
    Methods,
    // The attribute table
    Attributes,
}
