use std::fmt;
use std::marker::PhantomData;

use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn inner_classes<F>(mut self, f: F) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<InnerClassWriter<Ctx, InnerClassWriterState::InnerClass>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("InnerClasses")?;
        let mut builder = ManyWriter::new(self.context)?;
        f(&mut builder)?;
        self.context = builder.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct InnerClassWriter<Ctx, State: InnerClassWriterState::State> {
    context: Ctx,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> InnerClassWriter<Ctx, InnerClassWriterState::InnerClass> {
    pub fn inner_class<I>(
        mut self,
        class: I,
    ) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::OuterClass>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> InnerClassWriter<Ctx, InnerClassWriterState::OuterClass> {
    pub fn outer_class<I>(
        mut self,
        class: I,
    ) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::InnerName>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn no_outer_class<I>(mut self) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::InnerName>, EncodeError>
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

impl<Ctx: EncoderContext> InnerClassWriter<Ctx, InnerClassWriterState::InnerName> {
    pub fn inner_name<I>(
        mut self,
        name: I,
    ) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::InnerAccessFlags>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn no_inner_name<I>(
        mut self,
    ) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::InnerAccessFlags>, EncodeError>
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

impl<Ctx: EncoderContext> InnerClassWriter<Ctx, InnerClassWriterState::InnerAccessFlags> {
    pub fn inner_access_flags(
        mut self,
        flags: AccessFlags,
    ) -> Result<InnerClassWriter<Ctx, InnerClassWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(flags)?;

        Ok(InnerClassWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for InnerClassWriter<Ctx, InnerClassWriterState::InnerClass> {
    type Context = Ctx;
    type Disassembler = InnerClassWriter<Ctx, InnerClassWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(InnerClassWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for InnerClassWriter<Ctx, InnerClassWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<Ctx, State: InnerClassWriterState::State> fmt::Debug for InnerClassWriter<Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InnerClassWriter").finish()
    }
}

enc_state!(pub mod InnerClassWriterState: InnerClass, OuterClass, InnerName, InnerAccessFlags, End);
