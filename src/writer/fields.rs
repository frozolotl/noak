use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{cpool, encoding::*, ClassWriter};
use std::cmp::Ordering;

const ACCESS_FLAGS_OFFSET: Offset = Offset::new(0);
const NAME_OFFSET: Offset = ACCESS_FLAGS_OFFSET.offset(2);
const DESCRIPTOR_OFFSET: Offset = NAME_OFFSET.offset(2);
const ATTRIBUTE_LENGTH_OFFSET: Offset = DESCRIPTOR_OFFSET.offset(2);

pub struct FieldWriter<'a> {
    class_writer: &'a mut ClassWriter,
    state: WriteState,
}

impl<'a> FieldWriter<'a> {
    pub fn new(class_writer: &'a mut ClassWriter) -> FieldWriter<'a> {
        FieldWriter {
            class_writer,
            state: WriteState::AccessFlags,
        }
    }

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        let offset = self.class_writer.fields_end_offset.add(ACCESS_FLAGS_OFFSET);
        if self.state == WriteState::AccessFlags {
            self.class_writer.encoder.inserting(offset).write(flags)?;
            self.state = WriteState::Name;
        } else {
            self.class_writer.encoder.replacing(offset).write(flags)?;
        }
        Ok(self)
    }

    pub fn write_name(
        &mut self,
        name: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        let offset = self.class_writer.fields_end_offset.add(NAME_OFFSET);
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

    pub fn write_descriptor(
        &mut self,
        name: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        let offset = self.class_writer.fields_end_offset.add(DESCRIPTOR_OFFSET);
        match self.state.cmp(&WriteState::Name) {
            Ordering::Less => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::Fields,
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
        Ok(self)
    }

    pub fn finish(mut self) -> Result<&'a mut ClassWriter, EncodeError> {
        if self.state == WriteState::Attributes {
            Ok(self.class_writer)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Fields,
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
