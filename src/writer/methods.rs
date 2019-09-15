use crate::error::*;
use crate::header::AccessFlags;
use crate::mutf8::MString;
use crate::writer::{cpool, encoding::*, ClassWriter};

pub struct MethodWriter<'a> {
    class_writer: &'a mut ClassWriter,
    state: WriteState,
}

impl<'a> MethodWriter<'a> {
    pub(crate) fn new(class_writer: &'a mut ClassWriter) -> MethodWriter<'a> {
        MethodWriter {
            class_writer,
            state: WriteState::AccessFlags,
        }
    }

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::Methods)?;

        self.class_writer.encoder.write(flags)?;
        self.state = WriteState::Name;
        Ok(self)
    }

    pub fn write_name<I: Into<MString>>(
        &mut self,
        name: I,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let utf8_index = self.class_writer.insert_constant(cpool::Utf8 {
            content: name.into(),
        })?;
        self.write_name_index(utf8_index)
    }

    pub fn write_name_index(
        &mut self,
        name: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Name, Context::Methods)?;

        self.class_writer.encoder.write(name)?;
        self.state = WriteState::Descriptor;
        Ok(self)
    }

    pub fn write_descriptor<I: Into<MString>>(
        &mut self,
        descriptor: I,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        let utf8_index = self.class_writer.insert_constant(cpool::Utf8 {
            content: descriptor.into(),
        })?;
        self.write_descriptor_index(utf8_index)
    }

    pub fn write_descriptor_index(
        &mut self,
        descriptor: cpool::Index<cpool::Utf8>,
    ) -> Result<&mut MethodWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Descriptor, Context::Methods)?;

        self.class_writer.encoder.write(descriptor)?;
        self.state = WriteState::Attributes;
        self.write_empty_attributes()
    }

    fn write_empty_attributes(&mut self) -> Result<&mut MethodWriter<'a>, EncodeError> {
        if self.state == WriteState::Attributes {
            self.class_writer.encoder.write(0u16)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Methods,
            ))
        }
    }

    pub fn finish(self) -> Result<(), EncodeError> {
        if self.state == WriteState::Attributes {
            Ok(())
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
