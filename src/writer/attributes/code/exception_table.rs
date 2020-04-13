use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

pub struct ExceptionWriter<'a, 'b, Ctx> {
    code_writer: &'a mut CodeWriter<'b, Ctx>,
    state: WriteState,
}

impl<'a, 'b, Ctx: EncoderContext> ExceptionWriter<'a, 'b, Ctx> {
    pub fn write_start(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Start, Context::Code)?;

        let position = self.code_writer.get_label_position(label)?;
        // end has to fit into an u16 and thus the last valid index for end is 65535
        // but start has to be less than end and thus the last valid index for start is 65534
        if position >= u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }

        self.code_writer.class_writer_mut().encoder.write(position as u16)?;
        self.state = WriteState::End;
        Ok(self)
    }

    pub fn write_end(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::End, Context::Code)?;

        let position = self.code_writer.get_label_position(label)?;
        if position > u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }

        self.code_writer.class_writer_mut().encoder.write(position as u16)?;
        self.state = WriteState::Handler;
        Ok(self)
    }

    pub fn write_handler(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Handler, Context::Code)?;

        let position = self.code_writer.get_label_position(label)?;
        if position > u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }

        self.code_writer.class_writer_mut().encoder.write(position as u16)?;
        self.state = WriteState::CatchType;
        Ok(self)
    }

    pub fn write_catch_type<I>(&mut self, catch_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::CatchType, Context::Code)?;
        let index = catch_type.insert(&mut self.code_writer)?;
        self.code_writer.class_writer_mut().encoder.write(index)?;
        self.state = WriteState::Finished;
        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for ExceptionWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(code_writer: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            code_writer,
            state: WriteState::Start,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Finished, Context::Code)?;

        Ok(self.code_writer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Start,
    End,
    Handler,
    CatchType,
    Finished,
}
