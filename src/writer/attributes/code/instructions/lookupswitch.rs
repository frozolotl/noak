use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct LookupSwitchWriter<'a, 'b, Ctx> {
    code_writer: &'b mut CodeWriter<'a, Ctx>,
    state: WriteState,
    count_offset: Offset,
    count: u32,
    last_key: Option<i32>,
}

impl<'a, 'b, Ctx: EncoderContext> LookupSwitchWriter<'a, 'b, Ctx> {
    pub(super) fn new(
        code_writer: &'b mut CodeWriter<'a, Ctx>,
        offset: Offset,
    ) -> Result<Self, EncodeError> {
        code_writer.class_writer_mut().encoder.write(0xabu8)?;
        for _ in 0..3 - (offset.get() & 3) {
            code_writer.class_writer_mut().encoder.write(0u8)?;
        }

        let count_offset = code_writer
            .class_writer_mut()
            .encoder
            .position()
            .sub(code_writer.class_writer_mut().pool_end)
            .offset(4);

        Ok(LookupSwitchWriter {
            code_writer,
            state: WriteState::Default,
            count_offset,
            count: 0,
            last_key: None,
        })
    }

    pub(super) fn finish(self) -> Result<&'b mut CodeWriter<'a, Ctx>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Jumps, Context::Code)?;
        Ok(self.code_writer)
    }

    pub fn write_default(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Default, Context::Code)?;

        self.code_writer.class_writer_mut().encoder.write(label.0)?;
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
            self.code_writer
                .class_writer_mut()
                .encoder
                .write(self.count)?;
        } else {
            let count_offset = self
                .count_offset
                .add(self.code_writer.class_writer_mut().pool_end);
            self.code_writer
                .class_writer_mut()
                .encoder
                .replacing(count_offset)
                .write(self.count)?;
        }

        self.code_writer
            .class_writer_mut()
            .encoder
            .write(key)?
            .write(label.0)?;

        self.last_key = Some(key);

        Ok(self)
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Default,
    Jumps,
}
