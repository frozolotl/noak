use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct LookupSwitchWriter<'a, Ctx, State: LookupSwitchWriterState::State> {
    context: &'a mut InstructionWriter<Ctx>,
    count_offset: Offset,
    count: u32,
    last_key: Option<i32>,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Default> {
    pub fn default(
        self,
        label: LabelRef,
    ) -> Result<LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Jumps>, EncodeError> {
        self.context.encoder().write(label.0)?;

        Ok(LookupSwitchWriter {
            context: self.context,
            count_offset: self.count_offset,
            count: self.count,
            last_key: self.last_key,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Jumps> {
    /// Write a key-label pair, where the keys must be written in an increasing numerical order.
    pub fn pair(mut self, key: i32, label: LabelRef) -> Result<Self, EncodeError> {
        if self.last_key.map_or(false, |last_key| last_key >= key) {
            return Err(EncodeError::with_context(
                EncodeErrorKind::InvalidKeyOrder,
                Context::Code,
            ));
        }

        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code))?;
        if self.count == 1 {
            self.context.encoder().write(self.count)?;
        } else {
            self.context.encoder().replacing(self.count_offset).write(self.count)?;
        }

        self.context.encoder().write(key)?.write(label.0)?;

        self.last_key = Some(key);

        Ok(LookupSwitchWriter {
            context: self.context,
            count_offset: self.count_offset,
            count: self.count,
            last_key: self.last_key,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteAssembler for LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Default> {
    type Context = &'a mut InstructionWriter<Ctx>;
    type Disassembler = LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Jumps>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();

        context.encoder().write(0xabu8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.encoder().write(0u8)?;
        }

        let count_offset = context.encoder().position().offset(4);

        Ok(LookupSwitchWriter {
            context,
            count_offset,
            count: 0,
            last_key: None,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler for LookupSwitchWriter<'a, Ctx, LookupSwitchWriterState::Jumps> {
    type Context = &'a mut InstructionWriter<Ctx>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<'a, Ctx, State: LookupSwitchWriterState::State> fmt::Debug for LookupSwitchWriter<'a, Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LookupSwitchWriter").finish()
    }
}

enc_state!(pub mod LookupSwitchWriterState: Default, Jumps);
