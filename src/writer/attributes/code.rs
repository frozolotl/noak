mod exception_table;
pub mod instructions;
mod line_number_table;
mod local_variable_table;
mod stack_map;

pub use exception_table::ExceptionWriter;
pub use instructions::InstructionWriter;
pub use line_number_table::LineNumberWriter;
pub use local_variable_table::LocalVariableWriter;
pub use stack_map::StackMapTableWriter;

use crate::error::*;
use crate::writer::{encoding::*, AttributeWriter, ClassWriter};
use std::fmt;
use std::{convert::TryFrom, num::NonZeroU32};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_code<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut CodeWriter<'f, Ctx>) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("Code")?;
        let mut writer = CodeWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct CodeWriter<'a, Ctx> {
    context: &'a mut Ctx,
    label_positions: Vec<Option<NonZeroU32>>,
    state: WriteState,
}

impl<'a, Ctx: EncoderContext> CodeWriter<'a, Ctx> {
    pub fn write_max_stack(
        &mut self,
        max_stack: u16,
    ) -> Result<&mut CodeWriter<'a, Ctx>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::MaxStack, Context::Code)?;

        self.context.class_writer_mut().encoder.write(max_stack)?;
        self.state = WriteState::MaxLocals;
        Ok(self)
    }

    pub fn write_max_locals(
        &mut self,
        max_locals: u16,
    ) -> Result<&mut CodeWriter<'a, Ctx>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::MaxLocals, Context::Code)?;

        self.context.class_writer_mut().encoder.write(max_locals)?;
        self.state = WriteState::Instructions;
        Ok(self)
    }

    pub fn write_instructions<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut InstructionWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Instructions, Context::Code)?;

        let length_writer = LengthWriter::new(self.context)?;
        let mut writer = InstructionWriter::new(self)?;
        f(&mut writer)?;
        writer.finish()?;
        length_writer.finish(self.context)?;
        self.state = WriteState::ExceptionTable;

        Ok(self)
    }

    pub fn write_exceptions<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(
            &mut CountedWriter<'f, ExceptionWriter<'f, 'g, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::ExceptionTable, Context::Fields)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Attributes;
        Ok(self)
    }

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
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelNotFound,
                Context::Code,
            ))
        }
    }

    pub fn write_attributes<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, AttributeWriter<'f, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        // write exception count 0 if no exception was written
        if EncodeError::can_write(self.state, &WriteState::ExceptionTable, Context::Code)? {
            self.write_exceptions(|_| Ok(()))?;
        }

        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Code)?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(())
    }
}

impl<'a, Ctx: EncoderContext> EncoderContext for CodeWriter<'a, Ctx> {
    fn class_writer(&self) -> &ClassWriter {
        self.context.class_writer()
    }

    fn class_writer_mut(&mut self) -> &mut ClassWriter {
        self.context.class_writer_mut()
    }
}

impl<'a, Ctx: EncoderContext> WriteBuilder<'a> for CodeWriter<'a, Ctx> {
    type Context = Ctx;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(CodeWriter {
            context,
            label_positions: Vec::new(),
            state: WriteState::MaxStack,
        })
    }

    fn finish(mut self) -> Result<&'a mut Self::Context, EncodeError> {
        // write exception count 0 if no exception was written
        if EncodeError::can_write(self.state, &WriteState::ExceptionTable, Context::Code)? {
            self.write_exceptions(|_| Ok(()))?;
        }

        // write attribute count 0 if no attribute was written
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Code)? {
            self.write_attributes(|_| Ok(()))?;
        }

        if self.state == WriteState::Finished {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::Code,
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    MaxStack,
    MaxLocals,
    Instructions,
    ExceptionTable,
    Attributes,
    Finished,
}

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
