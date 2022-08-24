mod exception_table;
pub mod instructions;
mod line_number_table;
mod local_variable_table;
mod local_variable_type_table;
pub mod stack_map;

pub use exception_table::{ExceptionWriter, ExceptionWriterState};
pub use instructions::InstructionWriter;
pub use line_number_table::{LineNumberWriter, LineNumberWriterState};
pub use local_variable_table::{LocalVariableWriter, LocalVariableWriterState};
pub use local_variable_type_table::{LocalVariableTypeWriter, LocalVariableTypeWriterState};
pub use stack_map::StackMapTableWriter;

use crate::error::*;
use crate::writer::cpool;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    encoding::*,
};
use std::num::NonZeroU32;
use std::{fmt, marker::PhantomData};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn code<F>(mut self, f: F) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: FnOnce(
            CodeWriter<Ctx, CodeWriterState::MaxStack>,
        ) -> Result<CodeWriter<Ctx, CodeWriterState::End>, EncodeError>,
    {
        let length_writer = self.attribute_writer("Code")?;
        self.context = f(CodeWriter::new(self.context)?)?.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct CodeWriter<Ctx, State: CodeWriterState::State> {
    context: Ctx,
    label_positions: Vec<Option<NonZeroU32>>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::MaxStack> {
    pub fn max_stack(mut self, max_stack: u16) -> Result<CodeWriter<Ctx, CodeWriterState::MaxLocals>, EncodeError> {
        self.context.encoder().write(max_stack)?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::MaxLocals> {
    pub fn max_locals(
        mut self,
        max_locals: u16,
    ) -> Result<CodeWriter<Ctx, CodeWriterState::Instructions>, EncodeError> {
        self.context.encoder().write(max_locals)?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::Instructions> {
    pub fn instructions<F>(mut self, f: F) -> Result<CodeWriter<Ctx, CodeWriterState::ExceptionTable>, EncodeError>
    where
        F: for<'f> FnOnce(&'f mut InstructionWriter<Ctx>) -> Result<(), EncodeError>,
    {
        let length_writer = LengthWriter::new(&mut self.context)?;
        let mut writer = <InstructionWriter<Ctx> as WriteAssembler>::new(self)?;
        f(&mut writer)?;
        self = writer.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::ExceptionTable> {
    pub fn exceptions<F>(mut self, f: F) -> Result<CodeWriter<Ctx, CodeWriterState::Attributes>, EncodeError>
    where
        F: FnOnce(&mut ManyWriter<ExceptionWriter<Ctx, ExceptionWriterState::Start>, u16>) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::Attributes> {
    pub fn attributes<F>(mut self, f: F) -> Result<CodeWriter<Ctx, CodeWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<
                AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start>,
                u16,
            >,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;

        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext, State: CodeWriterState::State> CodeWriter<Ctx, State> {
    pub fn new_label(&mut self) -> Result<(Label, LabelRef), EncodeError> {
        let index = u32::try_from(self.label_positions.len())
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code))?;
        self.label_positions.push(None);
        Ok((Label(index), LabelRef(index)))
    }

    fn get_label_position(&self, label: LabelRef) -> Result<u32, EncodeError> {
        if let Some(pos) = self.label_positions[label.0 as usize] {
            Ok(pos.get() - 1)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelNotFound, Context::Code))
        }
    }

    fn get_label_position_u16(&self, label: LabelRef) -> Result<u16, EncodeError> {
        let position = self.get_label_position(label)?;
        position
            .try_into()
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
    }
}

impl<Ctx: EncoderContext, State: CodeWriterState::State> InternalEncoderContext for CodeWriter<Ctx, State> {
    fn encoder(&mut self) -> &mut VecEncoder {
        self.context.encoder()
    }

    fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        self.context.insert_constant(item)
    }
}

impl<Ctx: EncoderContext> WriteAssembler for CodeWriter<Ctx, CodeWriterState::MaxStack> {
    type Context = Ctx;
    type Disassembler = CodeWriter<Ctx, CodeWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(CodeWriter {
            context,
            label_positions: Vec::new(),
            _marker: PhantomData,
        })
    }
}

impl<Ctx, State: CodeWriterState::State> fmt::Debug for CodeWriter<Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeWriter").finish()
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for CodeWriter<Ctx, CodeWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

enc_state!(pub mod CodeWriterState: MaxStack, MaxLocals, Instructions, ExceptionTable, Attributes, End);

#[derive(PartialEq, Eq)]
pub struct Label(u32);

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Label").finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LabelRef(u32);

impl fmt::Debug for LabelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LabelRef").finish()
    }
}
