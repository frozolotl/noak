use crate::writer::{
    encoding::*,
    cpool::{self, ConstantPool},
};
use crate::error::*;
use crate::header::Version;

const CAFEBABE_END: Position = Position::new(4);
const POOL_START: Position = CAFEBABE_END.offset(2 + 2);
const EMPTY_POOL_END: Position = POOL_START.offset(2);

#[derive(Clone)]
pub struct ClassWriter {
    encoder: VecEncoder,
    level: WriteLevel,

    pool: ConstantPool,
    pool_end: Position,
}

impl ClassWriter {
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::with_capacity(capacity),
            level: WriteLevel::Start,
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
        }
    }

    pub fn write_version(&mut self, version: Version) -> Result<&mut ClassWriter, EncodeError> {
        if self.level == WriteLevel::Start {
            self.encoder.write(0xCAFEBABEu32)?;
            self.encoder.write(version.minor)?;
            self.encoder.write(version.major)?;
            self.level = WriteLevel::ConstantPool;
        } else {
            let mut encoder = self.encoder.replacing(CAFEBABE_END);
            encoder.write(version.minor)?;
            encoder.write(version.major)?;
        }
        Ok(self)
    }

    pub fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        let mut encoder = self.encoder.inserting(self.pool_end);
        let index = self.pool.insert(item, &mut encoder)?;
        self.pool_end = encoder.position();

        self.encoder.replacing(POOL_START).write(self.pool.len())?;

        Ok(index)
    }

    fn write_missing(&mut self) -> Result<&mut ClassWriter, EncodeError> {
        self.write_version(Version::latest())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        self.write_missing()?;
        Ok(self.encoder.into_inner())
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
