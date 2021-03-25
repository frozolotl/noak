use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn exceptions<F>(mut self, f: F) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: CountedWrite<ExceptionWriter<Ctx, ExceptionWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("Exceptions")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        self.context = builder.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct ExceptionWriter<Ctx, State: ExceptionWriterState::State> {
    context: Ctx,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Start> {
    /// Writes the index to an exception able to be thrown by this method.
    pub fn exception<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(self)
    }
}

impl<Ctx: EncoderContext> WriteAssembler for ExceptionWriter<Ctx, ExceptionWriterState::Start> {
    type Context = Ctx;
    type Disassembler = ExceptionWriter<Ctx, ExceptionWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for ExceptionWriter<Ctx, ExceptionWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod ExceptionWriterState: Start, End);
