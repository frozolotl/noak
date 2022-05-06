use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start> {
    pub fn line_number_table<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>, EncodeError>
    where
        F: for<'g> FnOnce(
            &mut CountedWriter<LineNumberWriter<Ctx, LineNumberWriterState::Start>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("LineNumberTable")?;
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

pub struct LineNumberWriter<Ctx, State: LineNumberWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> LineNumberWriter<Ctx, LineNumberWriterState::Start> {
    pub fn start(
        mut self,
        label: LabelRef,
    ) -> Result<LineNumberWriter<Ctx, LineNumberWriterState::LineNumber>, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        let offset = u16::try_from(offset)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent))?;
        self.context.class_writer_mut().encoder.write(offset)?;

        Ok(LineNumberWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> LineNumberWriter<Ctx, LineNumberWriterState::LineNumber> {
    pub fn line_number(
        mut self,
        line_number: u16,
    ) -> Result<LineNumberWriter<Ctx, LineNumberWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(line_number)?;

        Ok(LineNumberWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for LineNumberWriter<Ctx, LineNumberWriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = LineNumberWriter<Ctx, LineNumberWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(LineNumberWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for LineNumberWriter<Ctx, LineNumberWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<Ctx, State: LineNumberWriterState::State> fmt::Debug for LineNumberWriter<Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineNumberWriter").finish()
    }
}

enc_state!(pub mod LineNumberWriterState: Start, LineNumber, End);
