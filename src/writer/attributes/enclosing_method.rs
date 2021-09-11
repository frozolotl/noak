use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn enclosing_method<F>(mut self, f: F) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut CountedWriter<EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::Class>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("EnclosingMethod")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        self.context = builder.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct EnclosingMethodWriter<Ctx, State: EnclosingMethodWriterState::State> {
    context: Ctx,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::Class> {
    pub fn class<I>(
        mut self,
        class: I,
    ) -> Result<EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::Method>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(EnclosingMethodWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::Method> {
    pub fn method<I>(
        mut self,
        class: Option<I>,
    ) -> Result<EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::NameAndType>,
    {
        let index = class
            .map(|class| Ok(Some(class.insert(&mut self.context)?)))
            .unwrap_or(Ok(None))?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(EnclosingMethodWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::Class> {
    type Context = Ctx;
    type Disassembler = EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(EnclosingMethodWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for EnclosingMethodWriter<Ctx, EnclosingMethodWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

enc_state!(pub mod EnclosingMethodWriterState: Class, Method, End);
