use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{attributes::code::*, encoding::*};

pub struct TableSwitchWriter<'a, Ctx, State: TableSwitchWriterState::State> {
    context: &'a mut InstructionWriter<Ctx>,
    /// This field is multi-purpose.
    /// If the state is `High`, then it describes the `low` value of the table switch when casted to [`i32`].
    /// Else it actually describes the number of remaining jumps.
    remaining: u32,
    _marker: PhantomData<State>,
}

impl<'a, Ctx: EncoderContext> TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Default> {
    pub fn default(
        self,
        label: LabelRef,
    ) -> Result<TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Low>, EncodeError> {
        self.context.encoder().write(label.0)?;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Low> {
    pub fn low(mut self, low: i32) -> Result<TableSwitchWriter<'a, Ctx, TableSwitchWriterState::High>, EncodeError> {
        self.context.encoder().write(low)?;
        self.remaining = low as u32;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> TableSwitchWriter<'a, Ctx, TableSwitchWriterState::High> {
    pub fn high(mut self, high: i32) -> Result<TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Jumps>, EncodeError> {
        self.context.encoder().write(high)?;

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

impl<'a, Ctx: EncoderContext> TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Jumps> {
    pub fn jump(mut self, label: LabelRef) -> Result<Self, EncodeError> {
        if self.remaining == 0 {
            return Err(EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code));
        }

        self.remaining -= 1;
        self.context.encoder().write(label.0)?;

        Ok(TableSwitchWriter {
            context: self.context,
            remaining: self.remaining,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteAssembler for TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Default> {
    type Context = &'a mut InstructionWriter<Ctx>;
    type Disassembler = TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Jumps>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        let offset = context.current_offset();

        context.encoder().write(0xaau8)?;
        for _ in 0..3 - (offset.get() & 3) {
            context.encoder().write(0u8)?;
        }

        Ok(TableSwitchWriter {
            context,
            remaining: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a, Ctx: EncoderContext> WriteDisassembler for TableSwitchWriter<'a, Ctx, TableSwitchWriterState::Jumps> {
    type Context = &'a mut InstructionWriter<Ctx>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        if self.remaining == 0 {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::ValuesMissing, Context::Code))
        }
    }
}

impl<'a, Ctx, State: TableSwitchWriterState::State> fmt::Debug for TableSwitchWriter<'a, Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableSwitchWriter").finish()
    }
}

enc_state!(pub mod TableSwitchWriterState: Default, Low, High, Jumps);
