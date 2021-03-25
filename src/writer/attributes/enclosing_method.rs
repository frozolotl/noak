use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    pub fn write_enclosing_method<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<'a, Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: for<'f> CountedWrite<'f, EnclosingMethodWriter<'f, Ctx, EnclosingMethodWriterState::Class>, u16>,
    {
        let length_writer = self.attribute_writer("EnclosingMethod")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct EnclosingMethodWriter<'a, Ctx, State: EnclosingMethodWriterState::State> {
    context: &'a mut Ctx,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::Class> {
    pub fn write_class<I>(
        self,
        class: I,
    ) -> Result<EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::Method>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(EnclosingMethodWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::Method> {
    pub fn write_method<I>(
        self,
        class: Option<I>,
    ) -> Result<EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::NameAndType>,
    {
        let index = class
            .map(|class| Ok(Some(class.insert(self.context)?)))
            .unwrap_or(Ok(None))?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(EnclosingMethodWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteAssembler<'a> for EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::Class> {
    type Context = Ctx;
    type Disassembler = EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(EnclosingMethodWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler<'a>
    for EnclosingMethodWriter<'a, Ctx, EnclosingMethodWriterState::End>
{
    type Context = Ctx;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod EnclosingMethodWriterState: Class, Method, End);
