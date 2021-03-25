use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start> {
    pub fn write_local_variable_type_table<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>, EncodeError>
    where
        F: for<'g> CountedWrite<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("LocalVariableTypeTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        self.context = builder.finish()?;
        length_writer.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct LocalVariableTypeWriter<Ctx, State: LocalVariableTypeWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    start: u32,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Start> {
    pub fn write_start(
        mut self,
        label: LabelRef,
    ) -> Result<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Length>, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        let offset_u16 = u16::try_from(offset)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent))?;
        self.context.class_writer_mut().encoder.write(offset_u16)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: offset,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Length> {
    pub fn write_end(
        mut self,
        label: LabelRef,
    ) -> Result<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Name>, EncodeError> {
        let offset = self.context.get_label_position(label)?;

        if offset < self.start {
            return Err(EncodeError::with_context(
                EncodeErrorKind::NegativeOffset,
                Context::AttributeContent,
            ));
        }

        let length = u16::try_from(offset - self.start)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent))?;
        self.context.class_writer_mut().encoder.write(length)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Name> {
    pub fn write_name<I>(
        mut self,
        name: I,
    ) -> Result<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Signature>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Signature> {
    pub fn write_signature<I>(
        mut self,
        signature: I,
    ) -> Result<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Index>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = signature.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Index> {
    pub fn write_index(
        mut self,
        index: u16,
    ) -> Result<LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(LocalVariableTypeWriter {
            context,
            start: 0,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for LocalVariableTypeWriter<Ctx, LocalVariableTypeWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod LocalVariableTypeWriterState: Start, Length, Name, Signature, Index, End);
