use crate::error::*;
use crate::writer::{cpool, encoding::*, attributes::AttributeWriter, ClassWriter};
use crate::header::AccessFlags;

pub struct FieldWriter<'a> {
    class_writer: &'a mut ClassWriter,
    state: WriteState,
}

impl<'a> FieldWriter<'a> {
    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::Fields)?;

        self.class_writer.encoder.write(flags)?;
        self.state = WriteState::Name;
        Ok(self)
    }

    pub fn write_name<I: cpool::Insertable<cpool::Utf8>>(
        &mut self,
        name: I,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Name, Context::Fields)?;

        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        self.state = WriteState::Descriptor;
        Ok(self)
    }

    pub fn write_descriptor<I: cpool::Insertable<cpool::Utf8>>(
        &mut self,
        descriptor: I,
    ) -> Result<&mut FieldWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Descriptor, Context::Fields)?;

        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        self.state = WriteState::Attributes;
        Ok(self)
    }

    pub fn write_attributes<F, T>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut CountedWriter<AttributeWriter>) -> Result<T, EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Attributes)?;
        let mut builder = CountedWriter::new(self.class_writer)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(())
    }
}

impl<'a> WriteBuilder<'a> for FieldWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(FieldWriter {
            class_writer,
            state: WriteState::AccessFlags,
        })
    }

    fn finish(mut self) -> Result<&'a mut ClassWriter, EncodeError> {
        // write attribute count 0 if no attribute was written
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Attributes)? {
            self.write_attributes(|_| Ok(()))?;
        }

        if self.state == WriteState::Finished {
            Ok(self.class_writer)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::ValuesMissing, Context::Attributes))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    AccessFlags,
    Name,
    Descriptor,
    Attributes,
    Finished,
}
