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

pub struct MethodWriter<State: MethodWriterState::State> {
    class_writer: ClassWriter<class::ClassWriterState::Methods>,
    _marker: PhantomData<State>,
}

impl MethodWriter<MethodWriterState::AccessFlags> {
    pub fn access_flags(mut self, flags: AccessFlags) -> Result<MethodWriter<MethodWriterState::Name>, EncodeError> {
        self.class_writer.encoder().write(flags)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl MethodWriter<MethodWriterState::Name> {
    pub fn name<I>(mut self, name: I) -> Result<MethodWriter<MethodWriterState::Descriptor>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder().write(index)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl MethodWriter<MethodWriterState::Descriptor> {
    pub fn descriptor<I>(mut self, descriptor: I) -> Result<MethodWriter<MethodWriterState::Attributes>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let index = descriptor.insert(&mut self.class_writer)?;
        self.class_writer.encoder().write(index)?;
        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl MethodWriter<MethodWriterState::Attributes> {
    pub fn attributes<F>(mut self, f: F) -> Result<MethodWriter<MethodWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<
                AttributeWriter<ClassWriter<class::ClassWriterState::Methods>, AttributeWriterState::Start>,
                u16,
            >,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self.class_writer)?;
        f(&mut builder)?;
        self.class_writer = builder.finish()?;

        Ok(MethodWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteAssembler for MethodWriter<MethodWriterState::AccessFlags> {
    type Context = ClassWriter<class::ClassWriterState::Methods>;
    type Disassembler = MethodWriter<MethodWriterState::End>;

    fn new(class_writer: Self::Context) -> Result<Self, EncodeError> {
        Ok(MethodWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteDisassembler for MethodWriter<MethodWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Methods>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

impl<State: MethodWriterState::State> fmt::Debug for MethodWriter<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MethodWriter").finish()
    }
}

enc_state!(pub mod MethodWriterState: AccessFlags, Name, Descriptor, Attributes, End);
