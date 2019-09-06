use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{cpool, encoding::*, ClassWriter};
use crate::mutf8::MString;
use std::cmp::Ordering;

const ACCESS_FLAGS_OFFSET: Offset = Offset::new(0);
const NAME_OFFSET: Offset = ACCESS_FLAGS_OFFSET.offset(2);
const DESCRIPTOR_OFFSET: Offset = NAME_OFFSET.offset(2);
const ATTRIBUTE_LENGTH_OFFSET: Offset = DESCRIPTOR_OFFSET.offset(2);

pub struct MethodWriter<'a> {
    class_writer: &'a mut ClassWriter,
    state: WriteState,
}

impl<'a> MethodWriter<'a> {
    pub fn new(class_writer: &'a mut ClassWriter) -> MethodWriter<'a> {
        MethodWriter {
            class_writer,
            state: WriteState::AccessFlags,
        }
    }

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let offset = self.class_writer.methods_end_offset.add(ACCESS_FLAGS_OFFSET);
        if self.state == WriteState::AccessFlags {
            self.class_writer.encoder.inserting(offset).write(flags)?;
            self.state = WriteState::Name;
        } else {
            self.class_writer.encoder.replacing(offset).write(flags)?;
        }
        Ok(self)
    }

    pub fn write_name(&mut self, name: impl Into<MString>) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let utf8_index = self.class_writer.insert_constant(cpool::Utf8 { content: name.into() })?;
        self.write_name_index(utf8_index)
    }

    pub fn write_name_index(
        &mut self,
        name: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let offset = self.class_writer.methods_end_offset.add(NAME_OFFSET);
        match self.state.cmp(&WriteState::Name) {
            Ordering::Less => {
                self.write_access_flags(AccessFlags::empty())?;
                self.class_writer.encoder.inserting(offset).write(name)?;
                self.state = WriteState::Descriptor;
            }
            Ordering::Equal => {
                self.class_writer.encoder.inserting(offset).write(name)?;
                self.state = WriteState::Descriptor;
            }
            Ordering::Greater => {
                self.class_writer.encoder.replacing(offset).write(name)?;
            }
        }
        Ok(self)
    }

    pub fn write_descriptor(&mut self, descriptor: impl Into<MString>) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let utf8_index = self.class_writer.insert_constant(cpool::Utf8 { content: descriptor.into() })?;
        self.write_descriptor_index(utf8_index)
    }

    pub fn write_descriptor_index(
        &mut self,
        name: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let offset = self.class_writer.methods_end_offset.add(DESCRIPTOR_OFFSET);
        match self.state.cmp(&WriteState::Name) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::Methods,
                ));
            }
            Ordering::Equal => {
                self.class_writer.encoder.inserting(offset).write(name)?;
                self.state = WriteState::Attributes;
            }
            Ordering::Greater => {
                self.class_writer.encoder.replacing(offset).write(name)?;
            }
        }
        self.write_empty_attributes()
    }

    pub fn write_empty_attributes(&mut self) -> Result<&mut MethodWriter<'a>, EncodeError> {
        if self.state == WriteState::Attributes {
            self.class_writer.encoder.write(0u16);
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Methods,
            ))
        }
    }

    pub fn finish(mut self) -> Result<&'a mut ClassWriter, EncodeError> {
        if self.state == WriteState::Attributes {
            Ok(self.class_writer)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Methods,
            ))
        }
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    AccessFlags,
    Name,
    Descriptor,
    Attributes,
}