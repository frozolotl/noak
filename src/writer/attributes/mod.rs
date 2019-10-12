use crate::error::*;
use crate::writer::{cpool, encoding::*, ClassWriter};

pub struct AttributeWriter<'a> {
    class_writer: &'a mut ClassWriter,
    finished: bool,
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
