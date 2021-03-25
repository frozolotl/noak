use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{class, cpool, encoding::*, ClassWriter};

pub struct InterfaceWriter<'a, State: InterfaceWriterState::State> {
    class_writer: &'a mut ClassWriter<class::ClassWriterState::Interfaces>,
    _marker: PhantomData<State>,
}

impl<'a> InterfaceWriter<'a, InterfaceWriterState::End> {
    /// Writes the index to an interface implemented by this class.
    pub fn write_interface<I>(mut self, name: I) -> Result<InterfaceWriter<'a, InterfaceWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(InterfaceWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteAssembler<'a> for InterfaceWriter<'a, InterfaceWriterState::Start> {
    type Context = ClassWriter<class::ClassWriterState::Interfaces>;
    type Disassembler = InterfaceWriter<'a, InterfaceWriterState::End>;

    fn new(class_writer: &'a mut Self::Context) -> Result<Self, EncodeError> {
        Ok(InterfaceWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl<'a> WriteDisassembler<'a> for InterfaceWriter<'a, InterfaceWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Interfaces>;

    fn finish(self) -> Result<&'a mut Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

crate::__enc_state!(pub mod InterfaceWriterState: Start, End);
