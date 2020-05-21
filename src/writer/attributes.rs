mod inner_classes;
pub mod code;
mod debug;
mod field;
mod method;

pub use inner_classes::*;
pub use debug::*;
pub use field::*;
pub use method::*;

use crate::error::*;
use crate::writer::{cpool, encoding::*};

pub struct AttributeWriter<'a, Ctx> {
    context: &'a mut Ctx,
    finished: bool,
}

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    fn attribute_writer<I>(&mut self, name: I) -> Result<LengthWriter<Ctx>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        if self.finished {
            return Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                Context::AttributeContent,
            ));
        }

        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        LengthWriter::new(self.context)
    }

    pub fn write_attribute<I>(&mut self, name: I, bytes: &[u8]) -> Result<&mut Self, EncodeError>
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

        Ok(self)
    }
}

impl<'a, Ctx: EncoderContext> WriteBuilder<'a> for AttributeWriter<'a, Ctx> {
    type Context = Ctx;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(AttributeWriter {
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
                Context::Attributes,
            ))
        }
    }
}
