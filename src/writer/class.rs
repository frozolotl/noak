use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::mutf8::MString;
use crate::writer::{
    cpool::{self, ConstantPool},
    encoding::*,
    fields::FieldWriter,
    methods::MethodWriter,
};
use std::cmp::Ordering;
use std::fmt;

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

/// Method table length offset starting from the field table end
const METHODS_START_OFFSET: Offset = Offset::new(0);
/// Method table end offset starting from the field table end
const METHODS_EMPTY_END_OFFSET: Offset = METHODS_START_OFFSET.offset(2);

#[derive(Clone)]
pub struct ClassWriter {
    pub(in crate::writer) encoder: VecEncoder,
    state: WriteState,

    pool: ConstantPool,
    pool_end: Offset,
    interface_count: u16,
    pub(in crate::writer) fields_end_offset: Offset,
    field_count: u16,
    pub(in crate::writer) methods_end_offset: Offset,
    method_count: u16,
}

impl ClassWriter {
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::with_capacity(capacity),
            state: WriteState::Start,
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
            interface_count: 0,
            fields_end_offset: FIELDS_EMPTY_END_OFFSET,
            field_count: 0,
            methods_end_offset: METHODS_EMPTY_END_OFFSET,
            method_count: 0,
        }
    }

    pub fn write_version(&mut self, version: Version) -> Result<&mut ClassWriter, EncodeError> {
        if self.state == WriteState::Start {
            self.encoder.write(0xCAFEBABEu32)?;
            self.encoder.write(version.minor)?;
            self.encoder.write(version.major)?;
            self.state = WriteState::ConstantPool;
        } else {
            let mut encoder = self.encoder.replacing(CAFEBABE_END);
            encoder.write(version.minor)?;
            encoder.write(version.major)?;
        }
        Ok(self)
    }

    fn write_empty_pool(&mut self) -> Result<&mut ClassWriter, EncodeError> {
        if self.state == WriteState::Start {
            self.write_version(Version::latest())?;
        }

        if self.state == WriteState::ConstantPool {
            self.encoder.write(1u16)?;
            self.state = WriteState::AccessFlags;
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
        match self.state.cmp(&WriteState::AccessFlags) {
            Ordering::Less => {
                self.write_empty_pool()?;
                self.encoder.write(flags)?;
                self.state = WriteState::ThisClass;
            }
            Ordering::Equal => {
                self.encoder.write(flags)?;
                self.state = WriteState::ThisClass;
            }
            Ordering::Greater => self.encoder.replacing(self.pool_end).write(flags)?,
        }
        Ok(self)
    }

    pub fn write_this_class_name(
        &mut self,
        name: impl Into<MString>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        let utf8_index = self.insert_constant(cpool::Utf8 {
            content: name.into(),
        })?;
        let class_index = self.insert_constant(cpool::Class { name: utf8_index })?;
        self.write_this_class_index(class_index)
    }

    pub fn write_this_class_index(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.state.cmp(&WriteState::ThisClass) {
            Ordering::Less => {
                self.write_access_flags(AccessFlags::empty())?;
                self.encoder.write(index)?;
                self.state = WriteState::SuperClass;
            }
            Ordering::Equal => {
                self.encoder.write(index)?;
                self.state = WriteState::SuperClass;
            }
            Ordering::Greater => self
                .encoder
                .replacing(self.pool_end.add(THIS_CLASS_OFFSET))
                .write(index)?,
        }
        Ok(self)
    }

    pub fn write_super_class_name(
        &mut self,
        name: impl Into<MString>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        let utf8_index = self.insert_constant(cpool::Utf8 {
            content: name.into(),
        })?;
        let class_index = self.insert_constant(cpool::Class { name: utf8_index })?;
        self.write_super_class_index(class_index)
    }

    pub fn write_super_class_index(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.state.cmp(&WriteState::SuperClass) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::ClassInfo,
                ));
            }
            Ordering::Equal => {
                self.encoder.write(index)?;
                self.state = WriteState::Interfaces;
            }
            Ordering::Greater => self
                .encoder
                .replacing(self.pool_end.add(SUPER_CLASS_OFFSET))
                .write(index)?,
        }
        Ok(self)
    }

    pub fn write_interface_name(
        &mut self,
        name: impl Into<MString>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        let utf8_index = self.insert_constant(cpool::Utf8 {
            content: name.into(),
        })?;
        let class_index = self.insert_constant(cpool::Class { name: utf8_index })?;
        self.write_interface_index(class_index)
    }

    pub fn write_interface_index(
        &mut self,
        index: cpool::Index<cpool::Class>,
    ) -> Result<&mut ClassWriter, EncodeError> {
        match self.state.cmp(&WriteState::Interfaces) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::Interfaces,
                ));
            }
            Ordering::Equal => {
                // the amount of implemented interfaces
                self.interface_count += 1;
                self.encoder.write(self.interface_count)?;
                self.encoder.write(index)?;
                self.state = WriteState::Fields;
            }
            Ordering::Greater => {
                self.interface_count = self.interface_count.checked_add(1).ok_or_else(|| {
                    EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Interfaces)
                })?;
                self.encoder
                    .replacing(self.interface_end_position())
                    .write(index)?;
                self.encoder
                    .replacing(self.pool_end.add(INTERFACES_START_OFFSET))
                    .write(self.interface_count)?;
            }
        }
        Ok(self)
    }

    pub fn write_field<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: FnOnce(&mut FieldWriter) -> Result<(), EncodeError>,
    {
        if self.field_count == 0 {
            self.encoder.write(1u16)?;
            self.field_count = 1;
        } else {
            self.field_count = self.field_count.checked_add(1).ok_or_else(|| {
                EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Fields)
            })?;
            self.encoder
                .replacing(self.interface_end_position().add(FIELDS_START_OFFSET))
                .write(self.field_count)?;
        }
        let mut writer = FieldWriter::new(self);
        f(&mut writer)?;
        writer.finish()?;
        Ok(self)
    }

    pub fn write_method<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: FnOnce(&mut MethodWriter) -> Result<(), EncodeError>,
    {
        if self.method_count == 0 {
            self.encoder.write(1u16)?;
            self.method_count = 1;
        } else {
            self.method_count = self.method_count.checked_add(1).ok_or_else(|| {
                EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Methods)
            })?;
            self.encoder
                .replacing(self.interface_end_position().add(METHODS_START_OFFSET))
                .write(self.method_count)?;
        }
        let mut writer = MethodWriter::new(self);
        f(&mut writer)?;
        writer.finish()?;
        Ok(self)
    }

    fn interface_end_position(&self) -> Offset {
        self.pool_end
            .add(INTERFACES_EMPTY_END_OFFSET)
            .offset(self.interface_count as usize * 2)
    }

    fn fields_end_position(&self) -> Offset {
        self.interface_end_position().add(self.fields_end_offset)
    }

    fn methods_end_position(&self) -> Offset {
        self.fields_end_position().add(self.methods_end_offset)
    }

    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        if self.state >= WriteState::Interfaces {
            Ok(self.encoder.into_inner())
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Interfaces,
            ))
        }
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
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
