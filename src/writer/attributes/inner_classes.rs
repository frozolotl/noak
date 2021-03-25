use std::marker::PhantomData;

use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx, AttributeWriterState::Start> {
    pub fn write_inner_classes<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<'a, Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: for<'f> CountedWrite<'f, InnerClassWriter<'f, Ctx, InnerClassWriterState::InnerClass>, u16>,
    {
        let length_writer = self.attribute_writer("InnerClasses")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct InnerClassWriter<'a, Ctx, State: InnerClassWriterState::State> {
    context: &'a mut Ctx,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerClass> {
    pub fn write_inner_class<I>(
        self,
        class: I,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::OuterClass>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> InnerClassWriter<'a, Ctx, InnerClassWriterState::OuterClass> {
    pub fn write_outer_class<I>(
        self,
        class: I,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerName>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn write_no_outer_class<I>(
        self,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerName>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.context.class_writer_mut().encoder.write(0u16)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerName> {
    pub fn write_inner_name<I>(
        self,
        name: I,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerAccessFlags>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn write_no_inner_name<I>(
        self,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerAccessFlags>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        self.context.class_writer_mut().encoder.write(0u16)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerAccessFlags> {
    pub fn write_inner_access_flags(
        self,
        flags: AccessFlags,
    ) -> Result<InnerClassWriter<'a, Ctx, InnerClassWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(flags)?;
        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteAssembler<'a> for InnerClassWriter<'a, Ctx, InnerClassWriterState::InnerClass> {
    type Context = Ctx;
    type Disassembler = InnerClassWriter<'a, Ctx, InnerClassWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(InnerClassWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler<'a> for InnerClassWriter<'a, Ctx, InnerClassWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod InnerClassWriterState: InnerClass, OuterClass, InnerName, InnerAccessFlags, End);
