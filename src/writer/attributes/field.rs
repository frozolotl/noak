use crate::error::*;
use crate::writer::{
    cpool::{self, Insertable},
    encoding::*,
    AttributeWriter,
};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_constant_value<I>(&mut self, value: I) -> Result<&mut Self, EncodeError>
    where
        I: Insertable<cpool::Item>,
    {
        let length_writer = self.attribute_writer("ConstantValue")?;
        let value_index = value.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(value_index)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}
