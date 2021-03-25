use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct TableSwitchWriter<'a, 'b, 'c, Ctx, State: TableSwitchWriterState::State> {
    context: &'a mut InstructionWriter<'b, 'c, Ctx>,
    remaining: u32,
    _marker: PhantomData<State>,
}

impl<'a, 'b, 'c, Ctx: EncoderContext> TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Default> {
    pub fn write_default(
        self,
        label: LabelRef,
    ) -> Result<TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Low>, EncodeError> {
        self.context.class_writer_mut().encoder.write(label.0)?;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Low> {
    pub fn write_low(
        mut self,
        low: i32,
    ) -> Result<TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::High>, EncodeError> {
        self.context.class_writer_mut().encoder.write(low)?;
        self.remaining = low as u32;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::High> {
    pub fn write_high(
        mut self,
        high: i32,
    ) -> Result<TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Jumps>, EncodeError> {
        self.context.class_writer_mut().encoder.write(high)?;

        let low = self.remaining as i32;
        if low > high {
            return Err(EncodeError::with_context(
                EncodeErrorKind::IncorrectBounds,
                Context::Code,
            ));
        }

        self.remaining = (high - low + 1) as u32;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Jumps> {
    pub fn write_jump(mut self, label: LabelRef) -> Result<Self, EncodeError> {
        if self.remaining == 0 {
            return Err(EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code));
        }

        self.remaining -= 1;
        self.context.class_writer_mut().encoder.write(label.0)?;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteAssembler<'a>
    for TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Default>
{
    type Context = InstructionWriter<'b, 'c, Ctx>;
    type Disassembler = TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Jumps>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();

        context.class_writer_mut().encoder.write(0xaau8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.class_writer_mut().encoder.write(0u8)?;
        }

        Ok(TableSwitchWriter {
            context,
            remaining: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, 'c, Ctx: EncoderContext> WriteDisassembler<'a>
    for TableSwitchWriter<'a, 'b, 'c, Ctx, TableSwitchWriterState::Jumps>
{
    type Context = InstructionWriter<'b, 'c, Ctx>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        if self.remaining == 0 {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::ValuesMissing, Context::Code))
        }
    }
}

crate::__enc_state!(pub mod TableSwitchWriterState: Default, Low, High, Jumps);
