use std::fmt;
use std::marker::PhantomData;

use crate::error::*;
use crate::header::AccessFlags;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    class, cpool,
    encoding::*,
    ClassWriter,
};

pub struct FieldWriter<State: FieldWriterState::State> {
    class_writer: ClassWriter<class::ClassWriterState::Fields>,
    _marker: PhantomData<State>,
}

impl FieldWriter<FieldWriterState::AccessFlags> {
    pub fn access_flags(mut self, flags: AccessFlags) -> Result<FieldWriter<FieldWriterState::Name>, EncodeError> {
        self.class_writer.encoder().write(flags)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl FieldWriter<FieldWriterState::Name> {
    pub fn name<I>(mut self, name: I) -> Result<FieldWriter<FieldWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder().write(index)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl FieldWriter<FieldWriterState::Descriptor> {
    pub fn descriptor<I>(mut self, descriptor: I) -> Result<FieldWriter<FieldWriterState::Attributes>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder().write(index)?;
        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl FieldWriter<FieldWriterState::Attributes> {
    pub fn attributes<F>(mut self, f: F) -> Result<FieldWriter<FieldWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<
                AttributeWriter<ClassWriter<class::ClassWriterState::Fields>, AttributeWriterState::Start>,
                u16,
            >,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self.class_writer)?;
        f(&mut builder)?;
        self.class_writer = builder.finish()?;

        Ok(FieldWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteAssembler for FieldWriter<FieldWriterState::AccessFlags> {
    type Context = ClassWriter<class::ClassWriterState::Fields>;

    fn new(class_writer: Self::Context) -> Result<Self, EncodeError> {
        Ok(FieldWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteDisassembler for FieldWriter<FieldWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Fields>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

impl<State: FieldWriterState::State> fmt::Debug for FieldWriter<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldWriter").finish()
    }
}

enc_state!(pub mod FieldWriterState: AccessFlags, Name, Descriptor, Attributes, End);
