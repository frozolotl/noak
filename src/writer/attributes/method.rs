use crate::error::*;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_exceptions<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, ExceptionWriter<'f, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("Exceptions")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct ExceptionWriter<'a, Ctx> {
    context: &'a mut Ctx,
    finished: bool,
}

impl<'a, Ctx: EncoderContext> ExceptionWriter<'a, Ctx> {
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
            let index = name.insert(self.context)?;
            self.context.class_writer_mut().encoder.write(index)?;
            self.finished = true;
            Ok(self)
        }
    }
}

impl<'a, Ctx: EncoderContext> WriteBuilder<'a> for ExceptionWriter<'a, Ctx> {
    type Context = Ctx;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            context,
            finished: false,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        if self.finished {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::AttributeContent,
            ))
        }
    }
}

impl<'a, I, Ctx> WriteSimple<'a, I> for ExceptionWriter<'a, Ctx>
where
    I: cpool::Insertable<cpool::Class>,
    Ctx: EncoderContext,
{
    fn write_simple(
        class_writer: &'a mut Self::Context,
        exception: I,
    ) -> Result<&'a mut Self::Context, EncodeError> {
        let mut writer = ExceptionWriter::new(class_writer)?;
        writer.write_exception(exception)?;
        writer.finish()
    }
}
