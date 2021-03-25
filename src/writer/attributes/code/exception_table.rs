use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

pub struct ExceptionWriter<Ctx, State: ExceptionWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::ExceptionTable>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Start> {
    pub fn start(
        mut self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::Length>, EncodeError> {
        let position = self.context.get_label_position(label)?;
        // end has to fit into an u16 and thus the last valid index for end is 65535
        // but start has to be less than end and thus the last valid index for start is 65534
        if position >= u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }
        self.context.class_writer_mut().encoder.write(position as u16)?;

        Ok(ExceptionWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> ExceptionWriter<Ctx, ExceptionWriterState::Length> {
    pub fn end(
        mut self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<Ctx, ExceptionWriterState::Handler>, EncodeError> {
        let position = self.context.get_label_position(label)?;
        if position > u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }
        self.context.class_writer_mut().encoder.write(position as u16)?;

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
        let position = self.context.get_label_position(label)?;
        if position > u16::max_value() as u32 {
            return Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code));
        }
        self.context.class_writer_mut().encoder.write(position as u16)?;

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
        self.context.class_writer_mut().encoder.write(index)?;

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

crate::__enc_state!(pub mod ExceptionWriterState: Start, Length, Handler, CatchType, End);
