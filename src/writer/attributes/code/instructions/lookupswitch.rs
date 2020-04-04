use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct LookupSwitchWriter<'a, 'b, 'c, Ctx> {
    context: &'a mut InstructionWriter<'b, 'c, Ctx>,
    state: WriteState,
    count_offset: Offset,
    count: u32,
    last_key: Option<i32>,
}

impl<'a, 'b, 'c, Ctx: EncoderContext> LookupSwitchWriter<'a, 'b, 'c, Ctx> {
    pub fn write_default(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Default, Context::Code)?;

        self.context.class_writer_mut().encoder.write(label.0)?;
        self.state = WriteState::Jumps;
        Ok(self)
    }

    /// Write a key-label pair, where the keys must be written in an increasing numerical order.
    pub fn write_pair(&mut self, key: i32, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Jumps, Context::Code)?;
        if self.last_key.map_or(false, |last_key| last_key >= key) {
            return Err(EncodeError::with_context(
                EncodeErrorKind::InvalidKeyOrder,
                Context::Code,
            ));
        }

        self.count = self.count.checked_add(1).ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code)
        })?;
        if self.count == 1 {
            self.context
                .class_writer_mut()
                .encoder
                .write(self.count)?;
        } else {
            let count_offset = self
                .count_offset
                .add(self.context.class_writer_mut().pool_end);
            self.context
                .class_writer_mut()
                .encoder
                .replacing(count_offset)
                .write(self.count)?;
        }

        self.context
            .class_writer_mut()
            .encoder
            .write(key)?
            .write(label.0)?;

        self.last_key = Some(key);

        Ok(self)
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteBuilder<'a> for LookupSwitchWriter<'a, 'b, 'c, Ctx> {
    type Context = InstructionWriter<'b, 'c, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();
        context.class_writer_mut().encoder.write(0xabu8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.class_writer_mut().encoder.write(0u8)?;
        }

        let count_offset = context
            .class_writer_mut()
            .encoder
            .position()
            .sub(context.class_writer_mut().pool_end)
            .offset(4);

        Ok(LookupSwitchWriter {
            context,
            state: WriteState::Default,
            count_offset,
            count: 0,
            last_key: None,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Jumps, Context::Code)?;
        Ok(self.context)
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Default,
    Jumps,
}
