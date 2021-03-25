use std::marker::PhantomData;

use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    class, cpool,
    encoding::*,
    ClassWriter,
};

pub struct FieldWriter<'a, State: FieldWriterState::State> {
    class_writer: &'a mut ClassWriter<class::ClassWriterState::Fields>,
    _marker: PhantomData<State>,
}

impl<'a> FieldWriter<'a, FieldWriterState::AccessFlags> {
    pub fn access_flags(self, flags: AccessFlags) -> Result<FieldWriter<'a, FieldWriterState::Name>, EncodeError> {
        self.class_writer.encoder.write(flags)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> FieldWriter<'a, FieldWriterState::Name> {
    pub fn name<I>(mut self, name: I) -> Result<FieldWriter<'a, FieldWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> FieldWriter<'a, FieldWriterState::Descriptor> {
    pub fn descriptor<I>(mut self, descriptor: I) -> Result<FieldWriter<'a, FieldWriterState::Attributes>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> FieldWriter<'a, FieldWriterState::Attributes> {
    pub fn attributes<F>(self, f: F) -> Result<FieldWriter<'a, FieldWriterState::End>, EncodeError>
    where
        F: for<'f> CountedWrite<
            'f,
            AttributeWriter<'f, ClassWriter<class::ClassWriterState::Fields>, AttributeWriterState::Start>,
            u16,
        >,
    {
        let mut builder = CountedWriter::new(self.class_writer)?;
        f.write_to(&mut builder)?;
        builder.finish()?;

        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteAssembler<'a> for FieldWriter<'a, FieldWriterState::AccessFlags> {
    type Context = ClassWriter<class::ClassWriterState::Fields>;
    type Disassembler = FieldWriter<'a, FieldWriterState::End>;

    fn new(class_writer: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(FieldWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteDisassembler<'a> for FieldWriter<'a, FieldWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Fields>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

crate::__enc_state!(pub mod FieldWriterState: AccessFlags, Name, Descriptor, Attributes, End);
