use std::marker::PhantomData;

use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    class, cpool,
    encoding::*,
    ClassWriter,
};

pub struct MethodWriter<'a, State: MethodWriterState::State> {
    class_writer: &'a mut ClassWriter<class::ClassWriterState::Methods>,
    _marker: PhantomData<State>,
}

impl<'a> MethodWriter<'a, MethodWriterState::AccessFlags> {
    pub fn access_flags(self, flags: AccessFlags) -> Result<MethodWriter<'a, MethodWriterState::Name>, EncodeError> {
        self.class_writer.encoder.write(flags)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> MethodWriter<'a, MethodWriterState::Name> {
    pub fn name<I>(mut self, name: I) -> Result<MethodWriter<'a, MethodWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> MethodWriter<'a, MethodWriterState::Descriptor> {
    pub fn descriptor<I>(
        mut self,
        descriptor: I,
    ) -> Result<MethodWriter<'a, MethodWriterState::Attributes>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> MethodWriter<'a, MethodWriterState::Attributes> {
    pub fn attributes<F>(self, f: F) -> Result<MethodWriter<'a, MethodWriterState::End>, EncodeError>
    where
        F: for<'f> CountedWrite<
            'f,
            AttributeWriter<'f, ClassWriter<class::ClassWriterState::Methods>, AttributeWriterState::Start>,
            u16,
        >,
    {
        let mut builder = CountedWriter::new(self.class_writer)?;
        f.write_to(&mut builder)?;
        builder.finish()?;

        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteAssembler<'a> for MethodWriter<'a, MethodWriterState::AccessFlags> {
    type Context = ClassWriter<class::ClassWriterState::Methods>;
    type Disassembler = MethodWriter<'a, MethodWriterState::End>;

    fn new(class_writer: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(MethodWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteDisassembler<'a> for MethodWriter<'a, MethodWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Methods>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

crate::__enc_state!(pub mod MethodWriterState: AccessFlags, Name, Descriptor, Attributes, End);
