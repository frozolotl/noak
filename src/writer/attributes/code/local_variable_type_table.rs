use crate::error::*;
use crate::writer::{attributes::code::*, cpool, encoding::*};

impl<'a, 'b, Ctx: EncoderContext> AttributeWriter<'a, CodeWriter<'b, Ctx>> {
    pub fn write_local_variable_type_table<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f, 'g> FnOnce(
            &mut CountedWriter<LocalVariableTypeWriter<'f, 'g, Ctx>, u16>,
        ) -> Result<(), EncodeError>,
    {
        let length_writer = self.attribute_writer("LocalVariableTypeTable")?;
        let mut builder = CountedWriter::new(self.context)?;
        f(&mut builder)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}

pub struct LocalVariableTypeWriter<'a, 'b, Ctx> {
    context: &'a mut CodeWriter<'b, Ctx>,
    state: WriteState,
}

impl<'a, 'b, Ctx: EncoderContext> LocalVariableTypeWriter<'a, 'b, Ctx> {
    pub fn write_start(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::Start, Context::AttributeContent)?;

        let offset = self.context.get_label_position(label)?;
        let offset_u16 = u16::try_from(offset).map_err(|_| {
            EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent)
        })?;

        self.context.class_writer_mut().encoder.write(offset_u16)?;
        self.state = WriteState::Length { start: offset };
        Ok(self)
    }

    pub fn write_end(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        let start = match self.state {
            WriteState::Start => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::ValuesMissing,
                    Context::AttributeContent,
                ))
            }
            WriteState::Length { start } => start,
            _ => {
                return Err(EncodeError::with_context(
                    EncodeErrorKind::CantChangeAnymore,
                    Context::AttributeContent,
                ))
            }
        };

        let offset = self.context.get_label_position(label)?;

        if offset < start {
            return Err(EncodeError::with_context(
                EncodeErrorKind::NegativeOffset,
                Context::AttributeContent,
            ));
        }

        let length = u16::try_from(offset - start).map_err(|_| {
            EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::AttributeContent)
        })?;

        self.context.class_writer_mut().encoder.write(length)?;
        self.state = WriteState::Name;
        Ok(self)
    }

    pub fn write_name<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Name, Context::AttributeContent)?;

        let index = name.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        self.state = WriteState::Signature;
        Ok(self)
    }

    pub fn write_signature<I>(&mut self, signature: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::Signature,
            Context::AttributeContent,
        )?;

        let index = signature.insert(self.context)?;
        self.context.class_writer_mut().encoder.write(index)?;

        self.state = WriteState::Index;
        Ok(self)
    }

    pub fn write_index(&mut self, index: u16) -> Result<&mut Self, EncodeError>
    {
        EncodeError::result_from_state(
            self.state,
            &WriteState::Index,
            Context::AttributeContent,
        )?;

        self.context.class_writer_mut().encoder.write(index)?;

        self.state = WriteState::Finished;
        Ok(self)
    }
}

impl<'a, 'b, Ctx: EncoderContext> WriteBuilder<'a> for LocalVariableTypeWriter<'a, 'b, Ctx> {
    type Context = CodeWriter<'b, Ctx>;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(LocalVariableTypeWriter {
            context,
            state: WriteState::Start,
        })
    }

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        EncodeError::result_from_state(
            self.state,
            &WriteState::Finished,
            Context::AttributeContent,
        )?;

        Ok(self.context)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    Start,
    Length { start: u32 },
    Name,
    Signature,
    Index,
    Finished,
}
