mod debug;

use crate::error::*;
use crate::writer::{cpool, encoding::*, ClassWriter};

pub struct AttributeWriter<'a> {
    class_writer: &'a mut ClassWriter,
    finished: bool,
}

impl<'a> AttributeWriter<'a> {
    fn attribute_writer<I>(&mut self, name: I) -> Result<LengthPrefixedEncoder, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.class_writer)?;
        self.class_writer.encoder.write(index)?;

        LengthPrefixedEncoder::new(self.class_writer)
    }

    pub fn write_attribute<I>(&mut self, name: I, bytes: &[u8]) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.class_writer)?;
        self.class_writer
            .encoder
            .write(index)?
            .write(bytes.len() as u32)?
            .write(bytes)?;

        Ok(self)
    }
}

impl<'a> WriteBuilder<'a> for AttributeWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(AttributeWriter {
            class_writer,
            finished: false,
        })
    }

    fn finish(self) -> Result<&'a mut ClassWriter, EncodeError> {
        if self.finished {
            Ok(self.class_writer)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Attributes,
            ))
        }
    }
}
