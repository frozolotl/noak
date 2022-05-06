use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start> {
    pub fn local_variable_table<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>, EncodeError>
    where
        F: for<'g> FnOnce(
            &mut CountedWriter<LocalVariableWriter<Ctx, LocalVariableWriterState::Start>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("LocalVariableTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        self.context = builder.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct LocalVariableWriter<Ctx, State: LocalVariableWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    start: u32,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> LocalVariableWriter<Ctx, LocalVariableWriterState::Start> {
    pub fn start(
        mut self,
        label: LabelRef,
    ) -> Result<LocalVariableWriter<Ctx, LocalVariableWriterState::Length>, EncodeError> {
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

impl<Ctx: EncoderContext> LocalVariableWriter<Ctx, LocalVariableWriterState::Length> {
    pub fn end(
        mut self,
        label: LabelRef,
    ) -> Result<LocalVariableWriter<Ctx, LocalVariableWriterState::Name>, EncodeError> {
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

impl<Ctx: EncoderContext> LocalVariableWriter<Ctx, LocalVariableWriterState::Name> {
    pub fn name<I>(
        mut self,
        name: I,
    ) -> Result<LocalVariableWriter<Ctx, LocalVariableWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableWriter<Ctx, LocalVariableWriterState::Descriptor> {
    pub fn descriptor<I>(
        mut self,
        descriptor: I,
    ) -> Result<LocalVariableWriter<Ctx, LocalVariableWriterState::Index>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(&mut self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LocalVariableWriter<Ctx, LocalVariableWriterState::Index> {
    pub fn index(mut self, index: u16) -> Result<LocalVariableWriter<Ctx, LocalVariableWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(index)?;

        Ok(LocalVariableWriter {
            context: self.context,
            start: self.start,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for LocalVariableWriter<Ctx, LocalVariableWriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = LocalVariableWriter<Ctx, LocalVariableWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(LocalVariableWriter {
            context,
            start: 0,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for LocalVariableWriter<Ctx, LocalVariableWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<Ctx, State: LocalVariableWriterState::State> fmt::Debug for LocalVariableWriter<Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalVariableWriter").finish()
    }
}

enc_state!(pub mod LocalVariableWriterState: Start, Length, Name, Descriptor, Index, End);
