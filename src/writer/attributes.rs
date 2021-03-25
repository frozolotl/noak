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

pub struct AttributeWriter<'a, Ctx, State: AttributeWriterState::State> {
    context: &'a mut Ctx,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    fn attribute_writer<I>(&mut self, name: I) -> Result<LengthWriter<Ctx>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        LengthWriter::new(self.context)
    }

    pub fn write_attribute<I>(
        self,
        name: I,
        bytes: &[u8],
    ) -> Result<AttributeWriter<'a, Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.context)?;
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

impl<'a, Ctx: EncoderContext> WriteAssembler<'a> for AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    type Context = Ctx;
    type Disassembler = AttributeWriter<'a, Ctx, AttributeWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(AttributeWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler<'a> for AttributeWriter<'a, Ctx, AttributeWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod AttributeWriterState: Start, End);
