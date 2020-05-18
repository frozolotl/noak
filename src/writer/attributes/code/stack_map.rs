use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

impl<'a, 'b, Ctx: EncoderContext> AttributeWriter<'a, CodeWriter<'b, Ctx>> {
    pub fn write_stack_map_table<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut StackMapTableWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
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

        let count = writer.count;
        let count_offset = count_offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(count_offset)
            .write(count)?;

        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct StackMapTableWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    last_position: u32,
    count: u16,
}

impl<'a, 'b, Ctx: EncoderContext> StackMapTableWriter<'a, 'b, Ctx> {
    pub fn write_same(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.write_same_extended(label);
        }

        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(offset as u8)?;
        Ok(self)
    }

    pub fn write_same_extended(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(251)?
            .write(offset)?;
        Ok(self)
    }

    pub fn write_same1<F>(&mut self, label: LabelRef, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut Same1Writer<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        if offset >= 64 {
            return self.write_same1_extended(label, f);
        }

        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(64 + offset as u8)?;

        let mut writer = Same1Writer::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;

        Ok(self)
    }

    pub fn write_same1_extended<F>(
        &mut self,
        label: LabelRef,
        f: F,
    ) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut Same1Writer<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(247)?
            .write(offset)?;

        let mut writer = Same1Writer::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;

        Ok(self)
    }

    pub fn write_chop(&mut self, label: LabelRef, count: u16) -> Result<&mut Self, EncodeError> {
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

    pub fn write_append<F>(&mut self, label: LabelRef, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut AppendWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
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

        let mut writer = AppendWriter::new(self.context)?;
        f(&mut writer)?;
        let count = writer.count;
        writer.finish()?;

        let type_offset = type_offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(type_offset)
            .write(251 + count)?;

        Ok(self)
    }

    pub fn write_full<F>(&mut self, label: LabelRef, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut FullWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        let offset = self.get_label_offset(label)?;
        self.increment_counter()?;
        self.context
            .class_writer_mut()
            .encoder
            .write(255)?
            .write(offset)?;

        let mut writer = FullWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;

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
        let offset = u16::try_from(position - self.last_position).map_err(|_| {
            EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent)
        })?;
        Ok(offset)
    }

    fn increment_counter(&mut self) -> Result<(), EncodeError> {
        self.count = self.count.checked_add(1).ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::None)
        })?;

        Ok(())
    }
}

pub struct VerificationTypeWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    finished: bool,
}

impl<'a, 'b, Ctx: EncoderContext> VerificationTypeWriter<'a, 'b, Ctx> {
    pub fn write_top(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(0u8)?;
        Ok(self)
    }

    pub fn write_integer(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(1u8)?;
        Ok(self)
    }

    pub fn write_float(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(2u8)?;
        Ok(self)
    }

    pub fn write_double(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(3u8)?;
        Ok(self)
    }

    pub fn write_long(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(4u8)?;
        Ok(self)
    }

    pub fn write_null(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(5u8)?;
        Ok(self)
    }

    pub fn write_uninitialized_this(&mut self) -> Result<&mut Self, EncodeError> {
        self.context.class_writer_mut().encoder.write(6u8)?;
        Ok(self)
    }

    pub fn write_object<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.context.class_writer_mut().encoder.write(7u8)?;
        class.insert(&mut self.context)?;
        Ok(self)
    }

    pub fn write_uninitialized(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        let offset = self.context.get_label_position(label)?;
        self.context
            .class_writer_mut()
            .encoder
            .write(8u8)?
            .write(offset)?;
        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for VerificationTypeWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(VerificationTypeWriter {
            context,
            finished: false,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        if self.finished {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::AttributeContent,
            ))
        }
    }
}

pub struct Same1Writer<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    finished: bool,
}

impl<'a, 'b, Ctx: EncoderContext> Same1Writer<'a, 'b, Ctx> {
    pub fn write_stack_item<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut VerificationTypeWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        if self.finished {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        let mut writer = VerificationTypeWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;
        self.finished = true;
        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for Same1Writer<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(Same1Writer {
            context,
            finished: false,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        if self.finished {
            Ok(self.context)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                Context::AttributeContent,
            ))
        }
    }
}

pub struct AppendWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    count: u8,
}

impl<'a, 'b, Ctx: EncoderContext> AppendWriter<'a, 'b, Ctx> {
    pub fn write_local<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut VerificationTypeWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        if self.count >= 3 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        let mut writer = VerificationTypeWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;
        self.count += 1;

        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for AppendWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(AppendWriter { context, count: 0 })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
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

pub struct FullWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    offset: Offset,
    local_count: u16,
    stack_size: u16,
}

impl<'a, 'b, Ctx: EncoderContext> FullWriter<'a, 'b, Ctx> {
    pub fn write_local<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut VerificationTypeWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        if self.local_count == u16::max_value() {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::AttributeContent,
            ));
        }

        let mut writer = VerificationTypeWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;
        self.local_count += 1;

        let count_offset = self.offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(count_offset)
            .write(self.local_count)?;

        Ok(self)
    }

    pub fn write_stack_item<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(&mut VerificationTypeWriter<'f, 'g, Ctx>) -> Result<(), EncodeError>,
    {
        if self.local_count == 0 {
            // write zero locals
            self.context.class_writer_mut().encoder.write(0u16)?;
            self.local_count = u16::max_value();
        }

        let mut writer = VerificationTypeWriter::new(self.context)?;
        f(&mut writer)?;
        writer.finish()?;
        self.stack_size += 1;

        let count_offset = self.offset.add(self.context.class_writer().pool_end);
        self.context
            .class_writer_mut()
            .encoder
            .replacing(count_offset)
            .write(self.stack_size)?;

        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for FullWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let count_offset = context
            .class_writer()
            .encoder
            .position()
            .sub(context.class_writer().pool_end);

        Ok(FullWriter {
            context,
            offset: count_offset,
            local_count: 0,
            stack_size: 0,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        if self.local_count == 0 && self.stack_size == 0 {
            // write zero locals
            self.context.class_writer_mut().encoder.write(0u16)?;
        }

        if self.stack_size == 0 {
            // write zero stack size
            self.context.class_writer_mut().encoder.write(0u16)?;
        }

        Ok(self.context)
    }
}
