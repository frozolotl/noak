use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<'a, 'b, Ctx: EncoderContext>
    AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start>
{
    pub fn write_local_variable_type_table<F>(
        mut self,
        f: F,
    ) -> Result<
        AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>,
        EncodeError,
    >
    where
        F: for<'f, 'g> CountedWrite<'f, LocalVariableTypeWriter<'f, 'g, Ctx, LocalVariableTypeWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("LocalVariableTypeTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct LocalVariableTypeWriter<'a, 'b, Ctx, State: LocalVariableTypeWriterState::State> {
    context: &'a mut CodeWriter<'b, Ctx, CodeWriterState::Attributes>,
    start: u32,
    _marker: PhantomData<State>,
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Start> {
    pub fn write_start(
        self,
        label: LabelRef,
    ) -> Result<LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Length>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Length> {
    pub fn write_end(
        self,
        label: LabelRef,
    ) -> Result<LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Name>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Name> {
    pub fn write_name<I>(
        self,
        name: I,
    ) -> Result<LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Signature>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Signature> {
    pub fn write_signature<I>(
        self,
        signature: I,
    ) -> Result<LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Index>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = signature.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Index> {
    pub fn write_index(
        self,
        index: u16,
    ) -> Result<LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(LocalVariableTypeWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteAssembler<'a>
    for LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::Start>
{
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;
    type Disassembler = LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(LocalVariableTypeWriter {
            context,
            start: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteDisassembler<'a>
    for LocalVariableTypeWriter<'a, 'b, Ctx, LocalVariableTypeWriterState::End>
{
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod LocalVariableTypeWriterState: Start, Length, Name, Signature, Index, End);
