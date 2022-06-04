use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

pub struct ExceptionWriter<Ctx, State: ExceptionWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::ExceptionTable>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Start> {
    pub fn start(mut self, label: LabelRef) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::Length>, EncodeError> {
        let position = self.context.get_label_position_u16(label)?;
        self.context.encoder().write(position)?;

        Ok(ExceptionWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Length> {
    pub fn end(mut self, label: LabelRef) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::Handler>, EncodeError> {
        let position = self.context.get_label_position_u16(label)?;
        self.context.encoder().write(position)?;

        Ok(ExceptionWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Handler> {
    pub fn handler(
        mut self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::CatchType>, EncodeError> {
        let position = self.context.get_label_position_u16(label)?;
        self.context.encoder().write(position)?;

        Ok(ExceptionWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::CatchType> {
    pub fn catch_type<I>(
        mut self,
        catch_type: I,
    ) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = catch_type.insert(&mut self.context)?;
        self.context.encoder().write(index)?;

        Ok(ExceptionWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for ExceptionWriter<Ctx, ExceptionWriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::ExceptionTable>;
    type Disassembler = ExceptionWriter<Ctx, ExceptionWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for ExceptionWriter<Ctx, ExceptionWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::ExceptionTable>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<Ctx, State: ExceptionWriterState::State> fmt::Debug for ExceptionWriter<Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExceptionWriter").finish()
    }
}

enc_state!(pub mod ExceptionWriterState: Start, Length, Handler, CatchType, End);
