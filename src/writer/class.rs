use std::marker::PhantomData;
use std::{fmt, io};

use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool::{self, ConstantPool},
    encoding::*,
    fields::{FieldWriter, FieldWriterState},
    interfaces::{InterfaceWriter, InterfaceWriterState},
    methods::{MethodWriter, MethodWriterState},
};

/// A class writer can build a class.
///
/// # Examples
/// ```
/// use noak::writer::ClassWriter;
/// use noak::{AccessFlags, Version};
///
/// let buf = ClassWriter::new()
///     .version(Version::latest())?
///     .access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
///     .this_class("com/example/Example")?
///     .super_class("java/lang/Object")?
///     .interfaces(|_interfaces| Ok(()))?
///     .fields(|_fields| Ok(()))?
///     .methods(|_methods| Ok(()))?
///     .attributes(|attributes| {
///         attributes.begin(|attribute| attribute.source_file("Example.java"))?;
///         Ok(())
///     })?
///     .finish()?;
/// # Ok::<(), noak::error::EncodeError>(())
/// ```
#[derive(Clone)]
pub struct ClassWriter<State: ClassWriterState::State> {
    /// Everything from the start to the constant pool
    start_encoder: VecEncoder,
    encoder: VecEncoder,
    pool: ConstantPool,
    _marker: PhantomData<State>,
}

impl Default for ClassWriter<ClassWriterState::Start> {
    fn default() -> ClassWriter<ClassWriterState::Start> {
        ClassWriter::new()
    }
}

impl ClassWriter<ClassWriterState::Start> {
    /// Creates a new class writer with a sensitive initial capacity.
    #[must_use]
    pub fn new() -> ClassWriter<ClassWriterState::Start> {
        ClassWriter {
            start_encoder: VecEncoder::new(Vec::with_capacity(1024)),
            encoder: VecEncoder::new(Vec::with_capacity(1024)),
            pool: ConstantPool::new(),
            _marker: PhantomData,
        }
    }

    pub fn version(mut self, version: Version) -> Result<ClassWriter<ClassWriterState::AccessFlags>, EncodeError> {
        self.start_encoder.write(0xCAFE_BABEu32)?;
        self.start_encoder.write(version.minor)?;
        self.start_encoder.write(version.major)?;

        // constant pool length
        self.start_encoder.write(1u16)?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::AccessFlags> {
    pub fn access_flags(mut self, flags: AccessFlags) -> Result<ClassWriter<ClassWriterState::ThisClass>, EncodeError> {
        self.encoder.write(flags)?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::ThisClass> {
    pub fn this_class<I>(mut self, name: I) -> Result<ClassWriter<ClassWriterState::SuperClass>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(&mut self)?;
        self.encoder.write(index)?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::SuperClass> {
    pub fn super_class<I>(mut self, name: I) -> Result<ClassWriter<ClassWriterState::Interfaces>, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = name.insert(&mut self)?;
        self.encoder.write(index)?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }

    pub fn no_super_class(mut self) -> Result<ClassWriter<ClassWriterState::Interfaces>, EncodeError> {
        self.encoder.write::<Option<cpool::Index<cpool::Class>>>(None)?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::Interfaces> {
    pub fn interfaces<F>(mut self, f: F) -> Result<ClassWriter<ClassWriterState::Fields>, EncodeError>
    where
        F: FnOnce(&mut ManyWriter<InterfaceWriter<InterfaceWriterState::Start>, u16>) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::Fields> {
    pub fn fields<F>(mut self, f: F) -> Result<ClassWriter<ClassWriterState::Methods>, EncodeError>
    where
        F: FnOnce(&mut ManyWriter<FieldWriter<FieldWriterState::AccessFlags>, u16>) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::Methods> {
    pub fn methods<F>(mut self, f: F) -> Result<ClassWriter<ClassWriterState::Attributes>, EncodeError>
    where
        F: FnOnce(&mut ManyWriter<MethodWriter<MethodWriterState::AccessFlags>, u16>) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::Attributes> {
    pub fn attributes<F>(mut self, f: F) -> Result<ClassWriter<ClassWriterState::End>, EncodeError>
    where
        F: FnOnce(
            &mut ManyWriter<
                AttributeWriter<ClassWriter<ClassWriterState::Attributes>, AttributeWriterState::Start>,
                u16,
            >,
        ) -> Result<(), EncodeError>,
    {
        let mut builder = ManyWriter::new(self)?;
        f(&mut builder)?;
        self = builder.finish()?;

        Ok(ClassWriter {
            start_encoder: self.start_encoder,
            encoder: self.encoder,
            pool: self.pool,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::End> {
    pub fn into_bytes(self) -> Result<Vec<u8>, EncodeError> {
        let mut buf = self.start_encoder.into_inner();
        buf.extend_from_slice(self.encoder.inner());
        Ok(buf)
    }

    pub fn write_bytes_to<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.start_encoder.inner())?;
        writer.write_all(self.encoder.inner())?;
        Ok(())
    }
}

impl<State: ClassWriterState::State> InternalEncoderContext for ClassWriter<State> {
    fn encoder(&mut self) -> &mut VecEncoder {
        &mut self.encoder
    }

    fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        let index = self.pool.insert(item, &mut self.start_encoder)?;
        self.start_encoder
            .replacing(Offset::new(4 + 2 + 2))
            .write(self.pool.len())?;
        Ok(index)
    }
}

impl<State: ClassWriterState::State> fmt::Debug for ClassWriter<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassWriter").finish()
    }
}

enc_state!(pub mod ClassWriterState: Start, AccessFlags, ThisClass, SuperClass, Interfaces, Fields, Methods, Attributes, End);
