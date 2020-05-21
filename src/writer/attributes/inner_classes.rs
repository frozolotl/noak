use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_inner_classes<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, InnerClassWriter<'f, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("InnerClasses")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct InnerClassWriter<'a, Ctx> {
    context: &'a mut Ctx,
    state: WriteState,
}

impl<'a, Ctx: EncoderContext> InnerClassWriter<'a, Ctx> {
    pub fn write_inner_class<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::InnerClass,
            Context::AttributeContent,
        )?;

        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        self.state = WriteState::OuterClass;
        Ok(self)
    }

    pub fn write_outer_class<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::OuterClass,
            Context::AttributeContent,
        )?;

        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        self.state = WriteState::InnerName;
        Ok(self)
    }

    pub fn write_no_outer_class<I>(&mut self) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::OuterClass,
            Context::AttributeContent,
        )?;

        self.context.class_writer_mut().encoder.write(0)?;
        self.state = WriteState::InnerName;
        Ok(self)
    }

    pub fn write_inner_name<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::InnerName,
            Context::AttributeContent,
        )?;

        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        self.state = WriteState::InnerAccessFlags;
        Ok(self)
    }

    pub fn write_no_inner_name<I>(&mut self) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::InnerName,
            Context::AttributeContent,
        )?;

        self.context.class_writer_mut().encoder.write(0)?;
        self.state = WriteState::InnerAccessFlags;
        Ok(self)
    }

    pub fn write_inner_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(
            self.state,
            &WriteState::InnerAccessFlags,
            Context::AttributeContent,
        )?;

        self.context.class_writer_mut().encoder.write(flags)?;
        self.state = WriteState::Finished;
        Ok(self)
    }
}

impl<'a, Ctx: EncoderContext> WriteBuilder<'a> for InnerClassWriter<'a, Ctx> {
    type Context = Ctx;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(InnerClassWriter {
            context,
            state: WriteState::InnerClass,
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
    InnerClass,
    OuterClass,
    InnerName,
    InnerAccessFlags,
    Finished,
}
