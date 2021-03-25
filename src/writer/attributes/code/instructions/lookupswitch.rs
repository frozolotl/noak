use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct LookupSwitchWriter<'a, 'b, 'c, Ctx, State: LookupSwitchWriterState::State> {
    context: &'a mut InstructionWriter<'b, 'c, Ctx>,
    count_offset: Offset,
    count: u32,
    last_key: Option<i32>,
    _marker: PhantomData<State>,
}

impl<'a, 'b, 'c, Ctx: EncoderContext> LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Default> {
    pub fn write_default(
        self,
        label: LabelRef,
    ) -> Result<LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Jumps>, EncodeError> {
        self.context.class_writer_mut().encoder.write(label.0)?;

        Ok(LookupSwitchWriter {
            context: self.context,
            count_offset: self.count_offset,
            count: self.count,
            last_key: self.last_key,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Jumps> {
    /// Write a key-label pair, where the keys must be written in an increasing numerical order.
    pub fn write_pair(mut self, key: i32, label: LabelRef) -> Result<Self, EncodeError> {
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
            self.context.class_writer_mut().encoder.write(self.count)?;
        } else {
            let count_offset = self.count_offset.add(self.context.class_writer_mut().pool_end);
            self.context
                .class_writer_mut()
                .encoder
                .replacing(count_offset)
                .write(self.count)?;
        }

        self.context.class_writer_mut().encoder.write(key)?.write(label.0)?;

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

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteAssembler<'a>
    for LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Default>
{
    type Context = InstructionWriter<'b, 'c, Ctx>;
    type Disassembler = LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Jumps>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();

        context.class_writer_mut().encoder.write(0xabu8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.class_writer_mut().encoder.write(0u8)?;
        }

        let count_offset = context
            .class_writer()
            .encoder
            .position()
            .sub(context.class_writer_mut().pool_end)
            .offset(4);

        Ok(LookupSwitchWriter {
            context,
            count_offset,
            count: 0,
            last_key: None,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteDisassembler<'a>
    for LookupSwitchWriter<'a, 'b, 'c, Ctx, LookupSwitchWriterState::Jumps>
{
    type Context = InstructionWriter<'b, 'c, Ctx>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod LookupSwitchWriterState: Default, Jumps);
