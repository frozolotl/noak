use crate::error::*;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_enclosing_method<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, EnclosingMethodWriter<'f, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("EnclosingMethod")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct EnclosingMethodWriter<'a, Ctx> {
    context: &'a mut Ctx,
    state: WriteState,
}

impl<'a, Ctx: EncoderContext> EnclosingMethodWriter<'a, Ctx> {
    pub fn write_class<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Class, Context::AttributeContent)?;

        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        self.state = WriteState::Method;
        Ok(self)
    }

    pub fn write_method<I>(&mut self, class: Option<I>) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::NameAndType>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Method, Context::AttributeContent)?;

        let index = class
            .map(|class| Ok(Some(class.insert(self.context)?)))
            .unwrap_or(Ok(None))?;
        self.context.class_writer_mut().encoder.write(index)?;

        self.state = WriteState::Finished;
        Ok(self)
    }
}

impl<'a, Ctx: EncoderContext> WriteBuilder<'a> for EnclosingMethodWriter<'a, Ctx> {
    type Context = Ctx;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(EnclosingMethodWriter {
            context,
            state: WriteState::Class,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        EncodeError::result_from_state(
            self.state,
            &WriteState::Finished,
            Context::AttributeContent,
        )?;

        Ok(self.context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Class,
    Method,
    Finished,
}
