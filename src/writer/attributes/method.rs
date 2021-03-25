use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    pub fn write_exceptions<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<'a, Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: for<'f> CountedWrite<'f, ExceptionWriter<'f, Ctx, ExceptionWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("Exceptions")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct ExceptionWriter<'a, Ctx, State: ExceptionWriterState::State> {
    context: &'a mut Ctx,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> ExceptionWriter<'a, Ctx, ExceptionWriterState::Start> {
    /// Writes the index to an exception able to be thrown by this method.
    pub fn write_exception<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(self)
    }
}

impl<'a, Ctx: EncoderContext> WriteAssembler<'a> for ExceptionWriter<'a, Ctx, ExceptionWriterState::Start> {
    type Context = Ctx;
    type Disassembler = ExceptionWriter<'a, Ctx, ExceptionWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler<'a> for ExceptionWriter<'a, Ctx, ExceptionWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod ExceptionWriterState: Start, End);
