use crate::error::*;
use crate::writer::{cpool, encoding::*, ClassWriter};

pub struct InterfaceWriter<'a> {
    class_writer: &'a mut ClassWriter,
    finished: bool,
}

impl<'a> InterfaceWriter<'a> {
    /// Writes the index to an interface implemented by this class.
    pub fn write_interface<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        if self.finished {
            Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                Context::Interfaces,
            ))
        } else {
            let index = name.insert(&mut self.class_writer)?;
            self.class_writer.encoder.write(index)?;
            self.finished = true;
            Ok(self)
        }
    }
}

impl<'a> WriteBuilder<'a> for InterfaceWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(InterfaceWriter {
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
                Context::Interfaces,
            ))
        }
    }
}
