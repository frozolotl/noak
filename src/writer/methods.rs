use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{attributes::AttributeWriter, cpool, encoding::*, ClassWriter};

pub struct MethodWriter<'a> {
    class_writer: &'a mut ClassWriter,
    state: WriteState,
}

impl<'a> MethodWriter<'a> {
    pub fn write_access_flags(&mut self, flags: AccessFlags) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::Methods)?;

        self.class_writer.encoder.write(flags)?;
        self.state = WriteState::Name;
        Ok(self)
    }

    pub fn write_name<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Name, Context::Methods)?;

        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        self.state = WriteState::Descriptor;
        Ok(self)
    }

    pub fn write_descriptor<I>(&mut self, descriptor: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Descriptor, Context::Methods)?;

        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        self.state = WriteState::Attributes;
        Ok(self)
    }

    pub fn write_attributes<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, AttributeWriter<'f, ClassWriter>, ClassWriter, u16>,
        ) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Attributes)?;
        let mut builder = CountedWriter::new(self.class_writer)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(())
    }
}

impl<'a> WriteBuilder<'a> for MethodWriter<'a> {
    type Context = ClassWriter;

    fn new(class_writer: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(MethodWriter {
            class_writer,
            state: WriteState::AccessFlags,
        })
    }

    fn finish(mut self) -> Result<&'a mut Self::Context, EncodeError> {
        // write attribute count 0 if no attribute was written
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Attributes)? {
            self.write_attributes(|_| Ok(()))?;
        }

        if self.state == WriteState::Finished {
            Ok(self.class_writer)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Attributes,
            ))
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
