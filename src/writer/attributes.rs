pub mod code;
mod debug;
mod enclosing_method;
mod field;
mod inner_classes;
mod method;

use std::marker::PhantomData;

pub use debug::*;
pub use field::*;
pub use inner_classes::*;
pub use method::*;

use crate::error::*;
use crate::writer::{cpool, encoding::*};

pub struct AttributeWriter<Ctx, State: AttributeWriterState::State> {
    context: Ctx,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    fn attribute_writer<I>(&mut self, name: I) -> Result<LengthWriter<Ctx>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        LengthWriter::new(&mut self.context)
    }

    pub fn raw_attribute<I>(
        mut self,
        name: I,
        bytes: &[u8],
    ) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.context)?;
        self.context
            .class_writer_mut()
            .encoder
            .write(index)?
            .write(bytes.len() as u32)?
            .write(bytes)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for AttributeWriter<Ctx, AttributeWriterState::Start> {
    type Context = Ctx;
    type Disassembler = AttributeWriter<Ctx, AttributeWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(AttributeWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for AttributeWriter<Ctx, AttributeWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

enc_state!(pub mod AttributeWriterState: Start, End);
