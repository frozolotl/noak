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
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    encoding::*,
    ClassWriter,
};
use std::{convert::TryFrom, num::NonZeroU32};
use std::{fmt, marker::PhantomData};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn write_code<F>(mut self, f: F) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        F: WriteOnce<CodeWriter<Ctx, CodeWriterState::MaxStack>>,
    {
        let length_writer = self.attribute_writer("Code")?;
        self.context = f.write_once(CodeWriter::new(self.context)?)?.finish()?;
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
    pub fn write_max_stack(
        mut self,
        max_stack: u16,
    ) -> Result<CodeWriter<Ctx, CodeWriterState::MaxLocals>, EncodeError> {
        self.context.class_writer_mut().encoder.write(max_stack)?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::MaxLocals> {
    pub fn write_max_locals(
        mut self,
        max_locals: u16,
    ) -> Result<CodeWriter<Ctx, CodeWriterState::Instructions>, EncodeError> {
        self.context.class_writer_mut().encoder.write(max_locals)?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::Instructions> {
    pub fn write_instructions<F>(
        mut self,
        f: F,
    ) -> Result<CodeWriter<Ctx, CodeWriterState::ExceptionTable>, EncodeError>
    where
        F: for<'g> WriteOnce<InstructionWriter<Ctx>>,
    {
        let length_writer = LengthWriter::new(&mut self.context)?;
        self = f.write_once(InstructionWriter::new(self)?)?.finish()?;
        length_writer.finish(&mut self.context)?;

        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::ExceptionTable> {
    pub fn write_exceptions<F>(mut self, f: F) -> Result<CodeWriter<Ctx, CodeWriterState::Attributes>, EncodeError>
    where
        F: for<'g> CountedWrite<ExceptionWriter<Ctx, ExceptionWriterState::Start>, u16>,
    {
        let mut builder = CountedWriter::new(self)?;
        f.write_to(&mut builder)?;
        self = builder.finish()?;
        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> CodeWriter<Ctx, CodeWriterState::Attributes> {
    pub fn write_attributes<F>(mut self, f: F) -> Result<CodeWriter<Ctx, CodeWriterState::End>, EncodeError>
    where
        F: CountedWrite<AttributeWriter<Ctx, AttributeWriterState::Start>, u16>,
    {
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        self.context = builder.finish()?;

        Ok(CodeWriter {
            context: self.context,
            label_positions: self.label_positions,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext, State: CodeWriterState::State> CodeWriter<Ctx, State> {
    fn new_label(&mut self) -> Result<(Label, LabelRef), EncodeError> {
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
}

impl<Ctx: EncoderContext, State: CodeWriterState::State> EncoderContext for CodeWriter<Ctx, State> {
    type State = Ctx::State;

    fn class_writer(&self) -> &ClassWriter<Self::State> {
        self.context.class_writer()
    }

    fn class_writer_mut(&mut self) -> &mut ClassWriter<Self::State> {
        self.context.class_writer_mut()
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

// TODO: find a way to allow this kind of thing
// impl<Ctx: EncoderContext> WriteDisassembler for CodeWriter<Ctx, CodeWriterState::ExceptionTable> {
//     type Context = Ctx;

//     fn finish(mut self) -> Result<Self::Context, EncodeError> {
//         let w = self.write_exceptions(|_| Ok(()))?;
//         let w = w.write_attributes(|_| Ok(()))?;

//         Ok(w.context)
//     }
// }

// impl<Ctx: EncoderContext> WriteDisassembler for CodeWriter<Ctx, CodeWriterState::Attributes> {
//     type Context = Ctx;

//     fn finish(mut self) -> Result<Self::Context, EncodeError> {
//         let w = self.write_attributes(|w| Ok(()))?;
//         Ok(w.context)
//     }
// }

impl<Ctx: EncoderContext> WriteDisassembler for CodeWriter<Ctx, CodeWriterState::End> {
    type Context = Ctx;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod CodeWriterState: MaxStack, MaxLocals, Instructions, ExceptionTable, Attributes, End);

pub struct Label(u32);

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Label").finish()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LabelRef(u32);

impl fmt::Debug for LabelRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LabelRef").finish()
    }
}
