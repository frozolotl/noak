use std::fmt;
use std::marker::PhantomData;

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

const CAFEBABE_END: Offset = Offset::new(4);
const POOL_START: Offset = CAFEBABE_END.offset(2 + 2);
const EMPTY_POOL_END: Offset = POOL_START.offset(2);

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
    pub(crate) encoder: VecEncoder,
    pool: ConstantPool,
    pub(crate) pool_end: Offset,
    _marker: PhantomData<State>,
}

impl Default for ClassWriter<ClassWriterState::Start> {
    fn default() -> ClassWriter<ClassWriterState::Start> {
        ClassWriter::new()
    }
}

impl ClassWriter<ClassWriterState::Start> {
    /// Creates a new class writer with a sensitive initial capacity.
    pub fn new() -> ClassWriter<ClassWriterState::Start> {
        ClassWriter::with_capacity(2048)
    }

    /// Creates a new class writer with a specific capacity.
    pub fn with_capacity(capacity: usize) -> ClassWriter<ClassWriterState::Start> {
        ClassWriter {
            encoder: VecEncoder::new(Vec::with_capacity(capacity)),
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
            _marker: PhantomData,
        }
    }

    /// Creates a new class writer and uses an existing buffer.
    /// The existing data on the buffer will be cleared.
    pub fn with_buffer(mut buffer: Vec<u8>) -> ClassWriter<ClassWriterState::Start> {
        buffer.clear();
        ClassWriter {
            encoder: VecEncoder::new(buffer),
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
            _marker: PhantomData,
        }
    }

    pub fn version(mut self, version: Version) -> Result<ClassWriter<ClassWriterState::AccessFlags>, EncodeError> {
        self.encoder.write(0xCAFE_BABEu32)?;
        self.encoder.write(version.minor)?;
        self.encoder.write(version.major)?;

        // constant pool length
        self.encoder.write(1u16)?;

        Ok(ClassWriter {
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
            _marker: PhantomData,
        })
    }
}

impl<State: ClassWriterState::State> ClassWriter<State> {
    pub fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        let mut encoder = self.encoder.inserting(self.pool_end);

        let index = self.pool.insert(item, &mut encoder)?;
        self.pool_end = encoder.position();

        self.encoder.replacing(POOL_START).write(self.pool.len())?;

        Ok(index)
    }
}

impl ClassWriter<ClassWriterState::AccessFlags> {
    pub fn access_flags(mut self, flags: AccessFlags) -> Result<ClassWriter<ClassWriterState::ThisClass>, EncodeError> {
        self.encoder.write(flags)?;

        Ok(ClassWriter {
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
            _marker: PhantomData,
        })
    }

    pub fn no_super_class(mut self) -> Result<ClassWriter<ClassWriterState::Interfaces>, EncodeError> {
        self.encoder.write::<Option<cpool::Index<cpool::Class>>>(None)?;

        Ok(ClassWriter {
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
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
            encoder: self.encoder,
            pool: self.pool,
            pool_end: self.pool_end,
            _marker: PhantomData,
        })
    }
}

impl ClassWriter<ClassWriterState::End> {
    pub fn finish(self) -> Result<Vec<u8>, EncodeError> {
        Ok(self.encoder.into_inner())
    }
}

impl<State: ClassWriterState::State> EncoderContext for ClassWriter<State> {
    type State = State;

    fn class_writer(&self) -> &ClassWriter<Self::State> {
        self
    }

    fn class_writer_mut(&mut self) -> &mut ClassWriter<Self::State> {
        self
    }
}

impl<State: ClassWriterState::State> fmt::Debug for ClassWriter<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassWriter").finish()
    }
}

enc_state!(pub mod ClassWriterState: Start, AccessFlags, ThisClass, SuperClass, Interfaces, Fields, Methods, Attributes, End);
