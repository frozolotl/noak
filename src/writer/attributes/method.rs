use crate::error::*;
use crate::writer::{cpool, encoding::*, AttributeWriter, ClassWriter};

impl<'a> AttributeWriter<'a> {
    pub fn write_exceptions<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, ExceptionWriter<'f>>) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("Exceptions")?;
        let mut builder = CountedWriter::new(self.class_writer)?;
        f(&mut builder)?;
        length_writer.finish(self.class_writer)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct ExceptionWriter<'a> {
    class_writer: &'a mut ClassWriter,
    finished: bool,
}

impl<'a> ExceptionWriter<'a> {
    /// Writes the index to an exception able to be thrown by this method.
    pub fn write_exception<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        if self.finished {
            Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                Context::AttributeContent,
            ))
        } else {
            let index = name.insert(&mut self.class_writer)?;
            self.class_writer.encoder.write(index)?;
            self.finished = true;
            Ok(self)
        }
    }
}

impl<'a> WriteBuilder<'a> for ExceptionWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
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
                Context::AttributeContent,
            ))
        }
    }
}
