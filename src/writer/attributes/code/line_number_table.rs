use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

impl<'a, 'b, Ctx: EncoderContext> AttributeWriter<'a, CodeWriter<'b, Ctx>> {
    pub fn write_line_number_table<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(
            &mut CountedWriter<LineNumberWriter<'f, 'g, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("LineNumberTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct LineNumberWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    state: WriteState,
}

impl<'a, 'b, Ctx: EncoderContext> LineNumberWriter<'a, 'b, Ctx> {
    pub fn write_start(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Start, Context::AttributeContent)?;

        let offset = self.context.get_label_position(label)?;
        let offset = u16::try_from(offset).map_err(|_| {
            EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent)
        })?;

        self.context.class_writer_mut().encoder.write(offset)?;
        self.state = WriteState::LineNumber;
        Ok(self)
    }

    pub fn write_line_number(&mut self, line_number: u16) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(
            self.state,
            &WriteState::LineNumber,
            Context::AttributeContent,
        )?;

        self.context.class_writer_mut().encoder.write(line_number)?;
        self.state = WriteState::Finished;
        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for LineNumberWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(LineNumberWriter {
            context,
            state: WriteState::Start,
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
    Start,
    LineNumber,
    Finished,
}
