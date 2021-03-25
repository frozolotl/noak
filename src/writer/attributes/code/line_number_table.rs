use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    encoding::*,
};

impl<'a, 'b, Ctx: EncoderContext>
    AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start>
{
    pub fn write_line_number_table<F>(
        mut self,
        f: F,
    ) -> Result<
        AttributeWriter<'a, CodeWriter<'b, Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>,
        EncodeError,
    >
    where
        F: for<'f, 'g> CountedWrite<'f, LineNumberWriter<'f, 'g, Ctx, LineNumberWriterState::Start>, u16>,
    {
        let length_writer = self.attribute_writer("LineNumberTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        length_writer.finish(self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct LineNumberWriter<'a, 'b, Ctx, State: LineNumberWriterState::State> {
    context: &'a mut CodeWriter<'b, Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<'a, 'b, Ctx: EncoderContext> LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::Start> {
    pub fn write_start(
        self,
        label: LabelRef,
    ) -> Result<LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::LineNumber>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::LineNumber> {
    pub fn write_line_number(
        self,
        line_number: u16,
    ) -> Result<LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(line_number)?;
        Ok(LineNumberWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteAssembler<'a> for LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::Start> {
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;
    type Disassembler = LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(LineNumberWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteDisassembler<'a> for LineNumberWriter<'a, 'b, Ctx, LineNumberWriterState::End> {
    type Context = CodeWriter<'b, Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod LineNumberWriterState: Start, LineNumber, End);
