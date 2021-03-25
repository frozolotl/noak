use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<'a, 'b, Ctx: EncoderContext>
    AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start>
{
    pub fn write_local_variable_table<F>(
        mut self,
        f: F,
    ) -> Result<
        AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>,
        EncodeError,
    >
    where
        F: for<'f, 'g> CountedWrite<'f, LocalVariableWriter<'f, 'g, Ctx, LocalVariableWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("LocalVariableTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct LocalVariableWriter<'a, 'b, Ctx, State: LocalVariableWriterState::State> {
    context: &'a mut CodeWriter<'b, Ctx, CodeWriterState::Attributes>,
    start: u32,
    _marker: PhantomData<State>,
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Start> {
    pub fn write_start(
        self,
        label: LabelRef,
    ) -> Result<LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Length>, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        let offset_u16 = u16::try_from(offset)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent))?;

        self.context.class_writer_mut().encoder.write(offset_u16)?;
        Ok(LocalVariableWriter {
            context: self.context,
            start: offset,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Length> {
    pub fn write_end(
        self,
        label: LabelRef,
    ) -> Result<LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Name>, EncodeError> {
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
        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Name> {
    pub fn write_name<I>(
        self,
        name: I,
    ) -> Result<LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Descriptor> {
    pub fn write_descriptor<I>(
        self,
        descriptor: I,
    ) -> Result<LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Index>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Index> {
    pub fn write_index(
        self,
        index: u16,
    ) -> Result<LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(index)?;
        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteAssembler<'a>
    for LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::Start>
{
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;
    type Disassembler = LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(LocalVariableWriter {
            context,
            start: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteDisassembler<'a>
    for LocalVariableWriter<'a, 'b, Ctx, LocalVariableWriterState::End>
{
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod LocalVariableWriterState: Start, Length, Name, Descriptor, Index, End);
