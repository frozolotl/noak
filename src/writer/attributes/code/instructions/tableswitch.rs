use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct TableSwitchWriter<'a, 'b, 'c, Ctx> {
    context: &'a mut InstructionWriter<'b, 'c, Ctx>,
    state: WriteState,
    remaining: u32,
}

impl<'a, 'b, 'c, Ctx: EncoderContext> TableSwitchWriter<'a, 'b, 'c, Ctx> {
    pub fn write_default(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Default, Context::Code)?;

        self.context.class_writer_mut().encoder.write(label.0)?;
        self.state = WriteState::Low;
        Ok(self)
    }

    pub fn write_low(&mut self, low: i32) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Low, Context::Code)?;

        self.context.class_writer_mut().encoder.write(low)?;
        self.remaining = low as u32;

        self.state = WriteState::High;
        Ok(self)
    }

    pub fn write_high(&mut self, high: i32) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::High, Context::Code)?;

        self.context.class_writer_mut().encoder.write(high)?;

        let low = self.remaining as i32;
        if low > high {
            return Err(EncodeError::with_context(
                EncodeErrorKind::IncorrectBounds,
                Context::Code,
            ));
        }

        self.remaining = (high - low + 1) as u32;

        self.state = WriteState::Jumps;
        Ok(self)
    }

    pub fn write_jump(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Jumps, Context::Code)?;
        self.context.class_writer_mut().encoder.write(label.0)?;
        if self.remaining == 1 {
            self.state = WriteState::Finished;
        } else if self.remaining == 0 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                Context::Code,
            ));
        }

        self.remaining -= 1;

        Ok(self)
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteBuilder<'a> for TableSwitchWriter<'a, 'b, 'c, Ctx> {
    type Context = InstructionWriter<'b, 'c, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();

        context.class_writer_mut().encoder.write(0xaau8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.class_writer_mut().encoder.write(0u8)?;
        }

        Ok(TableSwitchWriter {
            context,
            state: WriteState::Default,
            remaining: 0,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Finished, Context::Code)?;
        Ok(self.context)
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Default,
    Low,
    High,
    Jumps,
    Finished,
}
