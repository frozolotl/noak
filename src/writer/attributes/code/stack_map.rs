use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{
    attributes::{code::*, AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::Start> {
    pub fn stack_map_table<F>(
        mut self,
        f: F,
    ) -> Result<AttributeWriter<CodeWriter<Ctx, CodeWriterState::Attributes>, AttributeWriterState::End>, EncodeError>
    where
        F: FnOnce(&mut StackMapTableWriter<Ctx>) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("StackMapTable")?;

        let count_offset = self.context.encoder().position();
        self.context.encoder().write(0u16)?;

        let mut writer = StackMapTableWriter {
            context: self.context,
            last_position: 0,
            count: 0,
        };
        f(&mut writer)?;
        self.context = writer.context;

        let count = writer.count;
        self.context.encoder().replacing(count_offset).write(count)?;

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
    pub fn same(&mut self, label: LabelRef) -> Result<(), EncodeError> {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.same_extended(label);
        }

        self.increment_counter()?;
        self.context.encoder().write(offset as u8)?;
        Ok(())
    }

    pub fn same_extended(&mut self, label: LabelRef) -> Result<(), EncodeError> {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.encoder().write(251u8)?.write(offset)?;
        Ok(())
    }

    pub fn same1<F>(&mut self, label: LabelRef, f: F) -> Result<(), EncodeError>
    where
        F: for<'ctx> FnOnce(
            Same1Writer<'ctx, Ctx, Same1WriterState::Start>,
        ) -> Result<Same1Writer<'ctx, Ctx, Same1WriterState::End>, EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.same1_extended(label, f);
        }

        self.increment_counter()?;
        self.context.encoder().write(64 + offset as u8)?;

        f(Same1Writer::new(&mut self.context)?)?.finish()?;

        Ok(())
    }

    pub fn same1_extended<F>(&mut self, label: LabelRef, f: F) -> Result<(), EncodeError>
    where
        F: for<'ctx> FnOnce(
            Same1Writer<'ctx, Ctx, Same1WriterState::Start>,
        ) -> Result<Same1Writer<'ctx, Ctx, Same1WriterState::End>, EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.encoder().write(247u8)?.write(offset)?;

        f(Same1Writer::new(&mut self.context)?)?.finish()?;

        Ok(())
    }

    pub fn chop(&mut self, label: LabelRef, count: u16) -> Result<(), EncodeError> {
        if count == 0 || count > 3 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context.encoder().write(251 - count)?.write(offset)?;

        Ok(())
    }

    pub fn append<F>(&mut self, label: LabelRef, f: F) -> Result<(), EncodeError>
    where
        F: for<'ctx> FnOnce(AppendWriter<'ctx, Ctx>) -> Result<AppendWriter<'ctx, Ctx>, EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;

        let type_offset = self.context.encoder().position();

        self.context
            .encoder()
            .write(0u8)? // placeholder for frame type
            .write(offset)?;

        let append_writer = f(AppendWriter::new(&mut self.context)?)?;
        let count = append_writer.count;
        append_writer.finish()?;

        self.context.encoder().replacing(type_offset).write(251 + count)?;

        Ok(())
    }

    pub fn full<F>(&mut self, label: LabelRef, f: F) -> Result<(), EncodeError>
    where
        F: for<'ctx> FnOnce(
            FullWriter<'ctx, Ctx, FullWriterState::Locals>,
        ) -> Result<FullWriter<'ctx, Ctx, FullWriterState::End>, EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;

        self.context.encoder().write(255u8)?.write(offset)?;

        f(FullWriter::new(&mut self.context)?)?.finish()?;

        Ok(())
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

impl<Ctx> fmt::Debug for StackMapTableWriter<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackMapTableWriter").finish()
    }
}

pub struct VerificationTypeWriter<'ctx, Ctx, State: VerificationTypeWriterState::State> {
    context: &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<'ctx, Ctx: EncoderContext> VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start> {
    pub fn top(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(0u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn integer(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(1u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn float(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(2u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn double(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(3u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn long(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(4u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn null(self) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(5u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn uninitialized_this(
        self,
    ) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        self.context.encoder().write(6u8)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn object<I>(
        self,
        class: I,
    ) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(self.context)?;
        self.context.encoder().write(7u8)?.write(index)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn uninitialized(
        self,
        label: LabelRef,
    ) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        self.context.encoder().write(8u8)?.write(offset)?;

        Ok(VerificationTypeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteAssembler
    for VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start>
{
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(VerificationTypeWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteDisassembler
    for VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>
{
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<'ctx, Ctx, State: VerificationTypeWriterState::State> fmt::Debug for VerificationTypeWriter<'ctx, Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationTypeWriter").finish()
    }
}

enc_state!(pub mod VerificationTypeWriterState: Start, End);

pub struct Same1Writer<'ctx, Ctx, State: Same1WriterState::State> {
    context: &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<'ctx, Ctx: EncoderContext> Same1Writer<'ctx, Ctx, Same1WriterState::Start> {
    pub fn stack_item<F>(mut self, f: F) -> Result<Same1Writer<'ctx, Ctx, Same1WriterState::End>, EncodeError>
    where
        F: FnOnce(
            VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start>,
        ) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError>,
    {
        self.context = f(VerificationTypeWriter::new(self.context)?)?.finish()?;

        Ok(Same1Writer {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteAssembler for Same1Writer<'ctx, Ctx, Same1WriterState::Start> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = Same1Writer<'ctx, Ctx, Same1WriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(Same1Writer {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteDisassembler for Same1Writer<'ctx, Ctx, Same1WriterState::End> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<'ctx, Ctx, State: Same1WriterState::State> fmt::Debug for Same1Writer<'ctx, Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Same1Writer").finish()
    }
}

enc_state!(pub mod Same1WriterState: Start, End);

pub struct AppendWriter<'ctx, Ctx> {
    context: &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>,
    count: u8,
}

impl<'ctx, Ctx: EncoderContext> AppendWriter<'ctx, Ctx> {
    pub fn local<F>(mut self, f: F) -> Result<Self, EncodeError>
    where
        F: FnOnce(
            VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start>,
        ) -> Result<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::End>, EncodeError>,
    {
        if self.count >= 3 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        self.context = f(VerificationTypeWriter::new(self.context)?)?.finish()?;
        self.count += 1;

        Ok(self)
    }
}

impl<'ctx, Ctx: EncoderContext> WriteAssembler for AppendWriter<'ctx, Ctx> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = AppendWriter<'ctx, Ctx>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(AppendWriter { context, count: 0 })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteDisassembler for AppendWriter<'ctx, Ctx> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;

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

impl<'ctx, Ctx> fmt::Debug for AppendWriter<'ctx, Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppendWriter").finish()
    }
}

pub struct FullWriter<'ctx, Ctx, State: FullWriterState::State> {
    context: &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>,
    _marker: PhantomData<State>,
}

impl<'ctx, Ctx: EncoderContext> FullWriter<'ctx, Ctx, FullWriterState::Locals> {
    pub fn locals<F>(mut self, f: F) -> Result<FullWriter<'ctx, Ctx, FullWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self.context)?;
        f(&mut builder)?;
        self.context = builder.finish()?;

        Ok(FullWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> FullWriter<'ctx, Ctx, FullWriterState::Stack> {
    pub fn stack<F>(mut self, f: F) -> Result<FullWriter<'ctx, Ctx, FullWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<VerificationTypeWriter<'ctx, Ctx, VerificationTypeWriterState::Start>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self.context)?;
        f(&mut builder)?;
        self.context = builder.finish()?;

        Ok(FullWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteAssembler for FullWriter<'ctx, Ctx, FullWriterState::Locals> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;
    type Disassembler = FullWriter<'ctx, Ctx, FullWriterState::End>;

    fn new(context: Self::Context) -> Result<Self, EncodeError> {
        Ok(FullWriter {
            context,
            _marker: PhantomData,
        })
    }
}

impl<'ctx, Ctx: EncoderContext> WriteDisassembler for FullWriter<'ctx, Ctx, FullWriterState::End> {
    type Context = &'ctx mut CodeWriter<Ctx, CodeWriterState::Attributes>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.context)
    }
}

impl<'ctx, Ctx, State: FullWriterState::State> fmt::Debug for FullWriter<'ctx, Ctx, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FullWriter").finish()
    }
}

enc_state!(pub mod FullWriterState: Locals, Stack, End);
