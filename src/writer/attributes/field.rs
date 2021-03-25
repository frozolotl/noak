use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool::{self, Insertable},
    encoding::*,
};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    pub fn write_constant_value<I>(
        mut self,
        value: I,
    ) -> Result<AttributeWriter<'a, Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: Insertable<cpool::Item>,
    {
        let length_writer = self.attribute_writer("ConstantValue")?;
        let value_index = value.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(value_index)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}
