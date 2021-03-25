use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start> {
    pub fn write_stack_map_table<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>, EncodeError>
    where
        F: FnOnce(&mut StackMapTableWriter<Ctx>) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("StackMapTable")?;

        let count_offset = self
            .context
            .class_writer()
            .encoder
            .position()
            .sub(self.context.class_writer().pool_end);
        self.context.class_writer_mut().encoder.write(0u16)?;

        let mut writer = StackMapTableWriter {
            context: self.context,
            last_position: 0,
            count: 0,
        };
        f(&mut writer)?;
        self.context = writer.context;

        let count = writer.count;
        let count_offset = count_offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(count_offset)
            .write(count)?;

        length_writer.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

pub struct StackMapTableWriter<Ctx> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    last_position: u32,
    count: u16,
}

impl<Ctx: EncoderContext> StackMapTableWriter<Ctx> {
    pub fn write_same(mut self, label: LabelRef) -> Result<Self, EncodeError> {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.write_same_extended(label);
        }

        self.increment_counter()?;
        self.context.class_writer_mut().encoder.write(offset as u8)?;
        Ok(self)
    }

    pub fn write_same_extended(mut self, label: LabelRef) -> Result<Self, EncodeError> {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.class_writer_mut().encoder.write(251)?.write(offset)?;
        Ok(self)
    }

    pub fn write_same1<F>(mut self, label: LabelRef, f: F) -> Result<Self, EncodeError>
    where
        F: WriteOnce<Same1Writer<Ctx, Same1WriterState::Start>>,
    {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.write_same1_extended(label, f);
        }

        self.increment_counter()?;
        self.context.class_writer_mut().encoder.write(64 + offset as u8)?;

        self.context = f.write_once(Same1Writer::new(self.context)?)?.finish()?;

        Ok(self)
    }

    pub fn write_same1_extended<F>(mut self, label: LabelRef, f: F) -> Result<Self, EncodeError>
    where
        F: WriteOnce<Same1Writer<Ctx, Same1WriterState::Start>>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.class_writer_mut().encoder.write(247)?.write(offset)?;

        self.context = f.write_once(Same1Writer::new(self.context)?)?.finish()?;

        Ok(self)
    }

    pub fn write_chop(mut self, label: LabelRef, count: u16) -> Result<Self, EncodeError> {
        if count == 0 || count > 3 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(251 - count)?
            .write(offset)?;

        Ok(self)
    }

    pub fn write_append<F>(mut self, label: LabelRef, f: F) -> Result<Self, EncodeError>
    where
        F: WriteOnce<AppendWriter<Ctx>>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;

        let type_offset = self
            .context
            .class_writer()
            .encoder
            .position()
            .sub(self.context.class_writer().pool_end);

        self.context
            .class_writer_mut()
            .encoder
            .write(0)? // placeholder
            .write(offset)?;

        let append_writer = f.write_once(AppendWriter::new(self.context)?)?;
        let count = append_writer.count;
        self.context = append_writer.finish()?;

        let type_offset = type_offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(type_offset)
            .write(251 + count)?;

        Ok(self)
    }

    pub fn write_full<F>(mut self, label: LabelRef, f: F) -> Result<Self, EncodeError>
    where
        F: WriteOnce<FullWriter<Ctx, FullWriterState::Locals>>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.class_writer_mut().encoder.write(255)?.write(offset)?;

        self.context = f.write_once(FullWriter::new(self.context)?)?.finish()?;

        Ok(self)
    }

    fn get_label_offset(&self, label: LabelRef) -> Result<u16, EncodeError> {
        let position = self.context.get_label_position(label)?;
        if position < self.last_position {
            return Err(EncodeError::with_context(
                EncodeErrorKind::NegativeOffset,
                Context::AttributeContent,
            ));
        }
        let offset = u16::try_from(position - self.last_position)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent))?;
        Ok(offset)
    }

    fn increment_counter(&mut self) -> Result<(), EncodeError> {
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::None))?;

        Ok(())
    }
}

pub struct VerificationTypeWriter<Ctx, State: VerificationTypeWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    length: u16,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start> {
    pub fn write_top(mut self) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(0u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_integer(
        mut self,
    ) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(1u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_float(mut self) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(2u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_double(
        mut self,
    ) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(3u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 2,
            _marker: PhantomData,
        })
    }

    pub fn write_long(mut self) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(4u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 2,
            _marker: PhantomData,
        })
    }

    pub fn write_null(mut self) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(5u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_uninitialized_this(
        mut self,
    ) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.class_writer_mut().encoder.write(6u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_object<I>(
        mut self,
        class: I,
    ) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.context.class_writer_mut().encoder.write(7u8)?;
        class.insert(&mut self.context)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }

    pub fn write_uninitialized(
        mut self,
        label: LabelRef,
    ) -> Result<VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        self.context.class_writer_mut().encoder.write(8u8)?.write(offset)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            length: 1,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = VerificationTypeWriter<Ctx, VerificationTypeWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(VerificationTypeWriter {
            context,
            length: 1, // unused at that point
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for VerificationTypeWriter<Ctx, VerificationTypeWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod VerificationTypeWriterState: Start, End);

pub struct Same1Writer<Ctx: EncoderContext, State: Same1WriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> Same1Writer<Ctx, Same1WriterState::Start> {
    pub fn write_stack_item<F>(mut self, f: F) -> Result<Same1Writer<Ctx, Same1WriterState::End>, EncodeError>
    where
        F: WriteOnce<VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start>>,
    {
        self.context = f.write_once(VerificationTypeWriter::new(self.context)?)?.finish()?;

        Ok(Same1Writer {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for Same1Writer<Ctx, Same1WriterState::Start> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = Same1Writer<Ctx, Same1WriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(Same1Writer {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for Same1Writer<Ctx, Same1WriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod Same1WriterState: Start, End);

pub struct AppendWriter<Ctx> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    count: u8,
}

impl<Ctx: EncoderContext> AppendWriter<Ctx> {
    pub fn write_local<F>(mut self, f: F) -> Result<Self, EncodeError>
    where
        F: WriteOnce<VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start>>,
    {
        if self.count >= 3 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        self.context = f.write_once(VerificationTypeWriter::new(self.context)?)?.finish()?;
        self.count += 1;

        Ok(self)
    }
}

impl<Ctx: EncoderContext> WriteAssembler for AppendWriter<Ctx> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = AppendWriter<Ctx>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(AppendWriter { context, count: 0 })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for AppendWriter<Ctx> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        if self.count > 0 {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::AttributeContent,
            ))
        }
    }
}

pub struct FullWriter<Ctx, State: FullWriterState::State> {
    context: CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<Ctx: EncoderContext> FullWriter<Ctx, FullWriterState::Locals> {
    pub fn write_locals<F>(mut self, f: F) -> Result<FullWriter<Ctx, FullWriterState::Locals>, EncodeError>
    where
        F: CountedWrite<VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start>, u16>,
    {
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        self.context = builder.finish()?;

        Ok(FullWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> FullWriter<Ctx, FullWriterState::Stack> {
    pub fn write_stack<F>(mut self, f: F) -> Result<FullWriter<Ctx, FullWriterState::End>, EncodeError>
    where
        F: CountedWrite<VerificationTypeWriter<Ctx, VerificationTypeWriterState::Start>, u16>,
    {
        let mut builder = CountedWriter::new(self.context)?;
        f.write_to(&mut builder)?;
        self.context = builder.finish()?;

        Ok(FullWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteAssembler for FullWriter<Ctx, FullWriterState::Locals> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = FullWriter<Ctx, FullWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(FullWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for FullWriter<Ctx, FullWriterState::End> {
    type Context = CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

crate::__enc_state!(pub mod FullWriterState: Locals, Stack, End);
