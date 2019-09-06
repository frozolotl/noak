use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::writer::{
    cpool::{self, ConstantPool},
    encoding::*,
    fields::FieldWriter,
};
use std::cmp::Ordering;

const CAFEBABE_END: Offset = Offset::new(4);
const POOL_START: Offset = CAFEBABE_END.offset(2 + 2);
const EMPTY_POOL_END: Offset = POOL_START.offset(2);

/// This class offset starting from the pool end
const THIS_CLASS_OFFSET: Offset = Offset::new(2);
/// Super class offset starting from the pool end
const SUPER_CLASS_OFFSET: Offset = THIS_CLASS_OFFSET.offset(2);

/// Interface table length offset starting from the pool end
const INTERFACES_START_OFFSET: Offset = SUPER_CLASS_OFFSET.offset(2);
/// Interface table end offset starting from the pool end
const INTERFACES_EMPTY_END_OFFSET: Offset = INTERFACES_START_OFFSET.offset(2);

/// Fields table length offset starting from the interfaces end
const FIELDS_START_OFFSET: Offset = Offset::new(0);
/// Fields table end offset starting from the interfaces end
const FIELDS_EMPTY_END_OFFSET: Offset = FIELDS_START_OFFSET.offset(2);

#[derive(Clone)]
pub struct ClassWriter {
    pub(in crate::writer) encoder: VecEncoder,
    level: WriteLevel,

    pool: ConstantPool,
    pool_end: Offset,
    interfaces_end_offset: Offset,
    pub(in crate::writer) fields_end_offset: Offset,
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
            interfaces_end_offset: INTERFACES_EMPTY_END_OFFSET,
            fields_end_offset: FIELDS_EMPTY_END_OFFSET,
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

    fn write_empty_pool(&mut self) -> Result<&mut ClassWriter, EncodeError> {
        if self.level == WriteLevel::Start {
            self.write_version(Version::latest())?;
        }

        if self.level == WriteLevel::ConstantPool {
            self.encoder.write(1u16)?;
            self.level = WriteLevel::AccessFlags;
        }
        Ok(self)
    }

    pub fn insert_constant<I: Into<cpool::Item>>(
        &mut self,
        item: I,
    ) -> Result<cpool::Index<I>, EncodeError> {
        self.write_empty_pool()?;

        let mut encoder = self.encoder.inserting(self.pool_end);
        let index = self.pool.insert(item, &mut encoder)?;
        self.pool_end = encoder.position();

        self.encoder.replacing(POOL_START).write(self.pool.len())?;

        Ok(index)
    }

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.level.cmp(&WriteLevel::AccessFlags) {
            Ordering::Less => {
                self.write_empty_pool()?;
                self.encoder.write(flags)?;
                self.level = WriteLevel::ThisClass;
            }
            Ordering::Equal => {
                self.encoder.write(flags)?;
                self.level = WriteLevel::ThisClass;
            }
            Ordering::Greater => self.encoder.replacing(self.pool_end).write(flags)?,
        }
        Ok(self)
    }

    pub fn write_this_class(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.level.cmp(&WriteLevel::ThisClass) {
            Ordering::Less => {
                self.write_access_flags(AccessFlags::empty())?;
                self.encoder.write(index)?;
                self.level = WriteLevel::SuperClass;
            }
            Ordering::Equal => {
                self.encoder.write(index)?;
                self.level = WriteLevel::SuperClass;
            }
            Ordering::Greater => self
                .encoder
                .replacing(self.pool_end.add(THIS_CLASS_OFFSET))
                .write(index)?,
        }
        Ok(self)
    }

    pub fn write_super_class(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.level.cmp(&WriteLevel::SuperClass) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::ClassInfo,
                ));
            }
            Ordering::Equal => {
                self.encoder.write(index)?;
                self.level = WriteLevel::Interfaces;
            }
            Ordering::Greater => self
                .encoder
                .replacing(self.pool_end.add(SUPER_CLASS_OFFSET))
                .write(index)?,
        }
        Ok(self)
    }

    pub fn write_interface(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.level.cmp(&WriteLevel::Interfaces) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::Interfaces,
                ));
            }
            Ordering::Equal => {
                // the amount of implemented interfaces
                self.encoder.write(1u16)?;
                self.encoder.write(index)?;
                self.level = WriteLevel::Fields;
            }
            Ordering::Greater => self
                .encoder
                .replacing(self.pool_end.add(SUPER_CLASS_OFFSET))
                .write(index)?,
        }
        self.interfaces_end_offset = self.interfaces_end_offset.offset(2);
        Ok(self)
    }

    fn write_field(&mut self) -> Result<FieldWriter, EncodeError> {
        Ok(FieldWriter::new(self))
    }

    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        Ok(self.encoder.into_inner())
    }

    pub fn fields_end_position(&self) -> Offset {
        self.pool_end.add(self.interfaces_end_offset).add(self.fields_end_offset)
    }
}

/// How much of the class is already written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteLevel {
    // Version numbers
    Start,
    ConstantPool,
    AccessFlags,
    ThisClass,
    SuperClass,
    Interfaces,
    Fields,
    Methods,
    Attributes,
}
