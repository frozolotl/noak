pub mod instructions;

pub use instructions::InstructionWriter;

use crate::error::*;
use crate::writer::{encoding::*, AttributeWriter, ClassWriter};
use std::convert::TryInto;
use std::fmt;

impl<'a> AttributeWriter<'a> {
    pub fn write_code<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut CodeWriter<'f>) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("Code")?;
        let mut writer = CodeWriter::new(self.class_writer)?;
        f(&mut writer)?;
        writer.finish()?;
        length_writer.finish(self.class_writer)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct CodeWriter<'a> {
    class_writer: &'a mut ClassWriter,
    /// The positions of labels, where `u32::max_value()` means "no label inserted" (`Option<u32>` would be too inefficient)
    label_positions: Vec<u32>,
    state: WriteState,
}

impl<'a> CodeWriter<'a> {
    pub fn write_max_stack(&mut self, max_stack: u16) -> Result<&mut CodeWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::MaxStack, Context::Code)?;

        self.class_writer.encoder.write(max_stack)?;
        self.state = WriteState::MaxLocals;
        Ok(self)
    }

    pub fn write_max_locals(
        &mut self,
        max_locals: u16,
    ) -> Result<&mut CodeWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::MaxLocals, Context::Code)?;

        self.class_writer.encoder.write(max_locals)?;
        self.state = WriteState::Instructions;
        Ok(self)
    }

    pub fn write_instructions<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut InstructionWriter<'f, 'g>) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Instructions, Context::Code)?;

        let length_writer = LengthWriter::new(self.class_writer)?;
        let mut writer = InstructionWriter::new(self)?;
        f(&mut writer)?;
        writer.finish()?;
        length_writer.finish(self.class_writer)?;

        self.class_writer.encoder.write(0u16)?;
        self.state = WriteState::Attributes;

        Ok(self)
    }

    fn new_label(&mut self) -> Result<(Label, LabelRef), EncodeError> {
        let index =
            self.label_positions.len().try_into().map_err(|_| {
                EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::Code)
            })?;
        self.label_positions.push(u32::max_value());
        Ok((Label(index), LabelRef(index)))
    }

    pub fn write_attributes<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, AttributeWriter<'f>>) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Code)?;
        let mut builder = CountedWriter::new(self.class_writer)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(())
    }
}

impl<'a> WriteBuilder<'a> for CodeWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(CodeWriter {
            class_writer,
            label_positions: Vec::new(),
            state: WriteState::MaxStack,
        })
    }

    fn finish(mut self) -> Result<&'a mut ClassWriter, EncodeError> {
        // write attribute count 0 if no attribute was written
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Code)? {
            self.write_attributes(|_| Ok(()))?;
        }

        if self.state == WriteState::Finished {
            Ok(self.class_writer)
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
