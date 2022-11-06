use std::fmt;
use std::marker::PhantomData;

use crate::error::*;
use crate::writer::{class, cpool, encoding::*, ClassWriter};

pub struct InterfaceWriter<State: InterfaceWriterState::State> {
    class_writer: ClassWriter<class::ClassWriterState::Interfaces>,
    _marker: PhantomData<State>,
}

impl InterfaceWriter<InterfaceWriterState::Start> {
    /// Writes the index to an interface implemented by this class.
    pub fn interface<I>(mut self, name: I) -> Result<InterfaceWriter<InterfaceWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(&mut self.class_writer)?;
        self.class_writer.encoder().write(index)?;
        Ok(InterfaceWriter {
            class_writer: self.class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteAssembler for InterfaceWriter<InterfaceWriterState::Start> {
    type Context = ClassWriter<class::ClassWriterState::Interfaces>;

    fn new(class_writer: Self::Context) -> Result<Self, EncodeError> {
        Ok(InterfaceWriter {
            class_writer,
            _marker: PhantomData,
        })
    }
}

impl WriteDisassembler for InterfaceWriter<InterfaceWriterState::End> {
    type Context = ClassWriter<class::ClassWriterState::Interfaces>;

    fn finish(self) -> Result<Self::Context, EncodeError> {
        Ok(self.class_writer)
    }
}

impl<State: InterfaceWriterState::State> fmt::Debug for InterfaceWriter<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceWriter").finish()
    }
}

enc_state!(pub mod InterfaceWriterState: Start, End);
