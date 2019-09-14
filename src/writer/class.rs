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

const CAFEBABE_END: Offset = Offset::new(4);
const POOL_START: Offset = CAFEBABE_END.offset(2 + 2);
const EMPTY_POOL_END: Offset = POOL_START.offset(2);

/// This class offset starting from the pool end
const THIS_CLASS_OFFSET: Offset = Offset::new(2);
/// Super class offset starting from the pool end
const SUPER_CLASS_OFFSET: Offset = THIS_CLASS_OFFSET.offset(2);

#[derive(Clone)]
pub struct ClassWriter {
    pub(in crate::writer) encoder: VecEncoder,
    state: WriteState,

    pool: ConstantPool,
    pool_end: Offset,
    interface_encoder: Option<CountedEncoder>,
    field_encoder: Option<CountedEncoder>,
    method_encoder: Option<CountedEncoder>,
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
            interface_encoder: None,
            field_encoder: None,
            method_encoder: None,
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

        let start_end = self.pool_end;
        let index = self.pool.insert(item, &mut encoder)?;
        self.pool_end = encoder.position();

        self.encoder.replacing(POOL_START).write(self.pool.len())?;

        if let Some(interface_encoder) = &mut self.interface_encoder {
            let offset = self.pool_end.sub(start_end);
            interface_encoder.move_start_offset_by(offset);
            if let Some(field_encoder) = &mut self.field_encoder {
                field_encoder.move_start_offset_by(offset);
                if let Some(method_encoder) = &mut self.method_encoder {
                    method_encoder.move_start_offset_by(offset);
                }
            }
        }

        Ok(index)
    }

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut ClassWriter, EncodeError> {
        self.write_empty_pool()?;
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::ClassInfo)?;
        self.encoder.write(flags)?;
        self.state = WriteState::ThisClass;
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
        EncodeError::result_from_state(self.state, &WriteState::ThisClass, Context::ClassInfo)?;
        self.encoder.write(index)?;
        self.state = WriteState::SuperClass;
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
        EncodeError::result_from_state(self.state, &WriteState::SuperClass, Context::ClassInfo)?;
        self.encoder.write(index)?;
        self.state = WriteState::Interfaces;
        Ok(self)
    }

    pub fn write_interface_name<I: Into<MString>>(
        &mut self,
        name: I,
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
        EncodeError::result_from_state(self.state, &WriteState::Interfaces, Context::Interfaces)?;
        let encoder = if let Some(encoder) = self.interface_encoder.as_mut() {
            encoder
        } else {
            self.interface_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
            self.interface_encoder.as_mut().unwrap()
        };

        self.encoder.write(index)?;
        encoder.increment_count(&mut self.encoder)?;

        Ok(self)
    }

    pub fn write_field<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: FnOnce(&mut FieldWriter) -> Result<(), EncodeError>,
    {
        self.write_empty_interfaces()?;
        EncodeError::result_from_state(self.state, &WriteState::Fields, Context::Fields)?;

        if self.field_encoder.is_none() {
            self.field_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
        }

        let mut writer = FieldWriter::new(self);
        f(&mut writer)?;
        writer.finish()?;

        self.field_encoder.as_mut().unwrap().increment_count(&mut self.encoder)?;

        Ok(self)
    }

    pub fn write_method<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: FnOnce(&mut MethodWriter) -> Result<(), EncodeError>,
    {
        self.write_empty_fields()?;
        EncodeError::result_from_state(self.state, &WriteState::Methods, Context::Methods)?;

        if self.method_encoder.is_none() {
            self.method_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
        }

        let mut writer = MethodWriter::new(self);
        f(&mut writer)?;
        writer.finish()?;

        self.method_encoder.as_mut().unwrap().increment_count(&mut self.encoder)?;

        Ok(self)
    }

    pub fn write_empty_interfaces(&mut self) -> Result<(), EncodeError> {
        if self.state < WriteState::Interfaces {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Interfaces,
            ))
        } else if self.interface_encoder.is_none() {
            self.interface_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
            self.state = WriteState::Fields;
            Ok(())
        } else if self.state == WriteState::Interfaces {
            self.state = WriteState::Fields;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn write_empty_fields(&mut self) -> Result<(), EncodeError> {
        self.write_empty_interfaces()?;
        if self.field_encoder.is_none() {
            self.field_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
            self.state = WriteState::Methods;
        } else if self.state == WriteState::Fields {
            self.state = WriteState::Methods;
        }
        Ok(())
    }

    pub fn write_empty_methods(&mut self) -> Result<(), EncodeError> {
        self.write_empty_fields()?;
        if self.method_encoder.is_none() {
            self.method_encoder = Some(CountedEncoder::new(&mut self.encoder)?);
            self.state = WriteState::Attributes;
        } else if self.state == WriteState::Methods {
            self.state = WriteState::Attributes;
        }
        Ok(())
    }

    pub fn write_empty_attributes(&mut self) -> Result<(), EncodeError> {
        self.write_empty_methods()?;
        if self.state == WriteState::Attributes {
            self.encoder.write(0u16)?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        self.write_empty_attributes()?;
        Ok(self.encoder.into_inner())
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
