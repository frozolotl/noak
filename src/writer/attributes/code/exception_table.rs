use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

pub struct ExceptionWriter<'a, 'b, Ctx, State: ExceptionWriterState::State> {
    context: &'a mut CodeWriter<'b, Ctx, CodeWriterState::ExceptionTable>,
    _marker: PhantomData<State>,
}

impl<'a, 'b, Ctx: EncoderContext> ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Start> {
    pub fn write_start(
        self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Length>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Length> {
    pub fn write_end(
        self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Handler>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Handler> {
    pub fn write_handler(
        self,
        label: LabelRef,
    ) -> Result<ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::CatchType>, EncodeError> {
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

impl<'a, 'b, Ctx: EncoderContext> ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::CatchType> {
    pub fn write_catch_type<I>(
        mut self,
        catch_type: I,
    ) -> Result<ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::End>, EncodeError>
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

impl<'a, 'b, Ctx: EncoderContext> WriteAssembler<'a> for ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::Start> {
    type Context = CodeWriter<'b, Ctx, CodeWriterState::ExceptionTable>;
    type Disassembler = ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::End>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(ExceptionWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteDisassembler<'a> for ExceptionWriter<'a, 'b, Ctx, ExceptionWriterState::End> {
    type Context = CodeWriter<'b, Ctx, CodeWriterState::ExceptionTable>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod ExceptionWriterState: Start, Length, Handler, CatchType, End);
