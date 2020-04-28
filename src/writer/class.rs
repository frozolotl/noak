use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::writer::{
    attributes::AttributeWriter,
    cpool::{self, ConstantPool},
    encoding::*,
    fields::FieldWriter,
    interfaces::InterfaceWriter,
    methods::MethodWriter,
};

const CAFEBABE_END: Offset = Offset::new(4);
const POOL_START: Offset = CAFEBABE_END.offset(2 + 2);
const EMPTY_POOL_END: Offset = POOL_START.offset(2);

/// A class writer can build a class.
///
/// # Examples
/// ```
/// use noak::writer::ClassWriter;
/// use noak::AccessFlags;
///
/// let buf = ClassWriter::new()
///     .write_access_flags(AccessFlags::PUBLIC | AccessFlags::SUPER)?
///     .write_this_class("com/example/Example")?
///     .write_super_class("java/lang/Object")?
///     .write_attributes(|writer| {
///         writer.write(|writer| {
///             writer.write_source_file("Example.java")?;
///             Ok(())
///         })?;
///         Ok(())
///     })?
///     .finish()?;
/// # Ok::<(), noak::error::EncodeError>(())
/// ```
#[derive(Clone)]
pub struct ClassWriter {
    pub(crate) encoder: VecEncoder,
    state: WriteState,

    pool: ConstantPool,
    pub(crate) pool_end: Offset,
}

impl Default for ClassWriter {
    fn default() -> ClassWriter {
        ClassWriter::new()
    }
}

impl ClassWriter {
    /// Creates a new class writer with a sensitive initial capacity.
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    /// Creates a new class writer with a specific capacity.
    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::new(Vec::with_capacity(capacity)),
            state: WriteState::Start,
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
        }
    }

    /// Creates a new class writer and uses an existing buffer.
    /// The existing data on the buffer will be cleared.
    pub fn with_buffer(mut buffer: Vec<u8>) -> ClassWriter {
        buffer.clear();
        ClassWriter {
            encoder: VecEncoder::new(buffer),
            state: WriteState::Start,
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
        }
    }

    pub fn write_version(&mut self, version: Version) -> Result<&mut Self, EncodeError> {
        if self.state == WriteState::Start {
            self.encoder.write(0xCAFE_BABEu32)?;
            self.encoder.write(version.minor)?;
            self.encoder.write(version.major)?;
            self.state = WriteState::ConstantPool;
        } else {
            let mut encoder = self.encoder.replacing(CAFEBABE_END);
            encoder.write(version.minor)?;
            encoder.write(version.major)?;
        }
        Ok(self)
    }

    fn write_empty_pool(&mut self) -> Result<&mut Self, EncodeError> {
        if self.state == WriteState::Start {
            self.write_version(Version::latest())?;
        }

        if self.state == WriteState::ConstantPool {
            self.encoder.write(1u16)?;
            self.state = WriteState::AccessFlags;
        }
        Ok(self)
    }

    pub fn insert_constant<I: Into<cpool::Item>>(
        &mut self,
        item: I,
    ) -> Result<cpool::Index<I>, EncodeError> {
        self.write_empty_pool()?;

        let mut encoder = self.encoder.inserting(self.pool_end);

        let index = self.pool.insert(item, &mut encoder)?;
        self.pool_end = encoder.position();

        self.encoder.replacing(POOL_START).write(self.pool.len())?;

        Ok(index)
    }

    pub fn write_access_flags(&mut self, flags: AccessFlags) -> Result<&mut Self, EncodeError> {
        self.write_empty_pool()?;
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::ClassInfo)?;
        self.encoder.write(flags)?;
        self.state = WriteState::ThisClass;
        Ok(self)
    }

    pub fn write_this_class<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::ThisClass, Context::ClassInfo)?;
        let index = name.insert(self)?;
        self.encoder.write(index)?;
        self.state = WriteState::SuperClass;
        Ok(self)
    }

    pub fn write_super_class<I>(&mut self, name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::SuperClass, Context::ClassInfo)?;
        let index = name.insert(self)?;
        self.encoder.write(index)?;
        self.state = WriteState::Interfaces;
        Ok(self)
    }

    pub fn write_no_super_class(&mut self) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::SuperClass, Context::ClassInfo)?;
        self.encoder
            .write::<Option<cpool::Index<cpool::Class>>>(None)?;
        self.state = WriteState::Interfaces;
        Ok(self)
    }

    pub fn write_interfaces<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, InterfaceWriter<'f>, u16>,
        ) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Interfaces, Context::Interfaces)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Fields;
        Ok(self)
    }

    pub fn write_fields<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, FieldWriter<'f>, u16>) -> Result<(), EncodeError>,
    {
        self.write_zero_interfaces()?;
        EncodeError::result_from_state(self.state, &WriteState::Fields, Context::Fields)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Methods;
        Ok(self)
    }

    pub fn write_methods<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, MethodWriter<'f>, u16>) -> Result<(), EncodeError>,
    {
        self.write_zero_fields()?;
        EncodeError::result_from_state(self.state, &WriteState::Methods, Context::Methods)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Attributes;
        Ok(self)
    }

    pub fn write_attributes<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            &mut CountedWriter<'f, AttributeWriter<'f, ClassWriter>, u16>,
        ) -> Result<(), EncodeError>,
    {
        self.write_zero_methods()?;
        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Attributes)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(self)
    }

    fn write_zero_interfaces(&mut self) -> Result<(), EncodeError> {
        if EncodeError::can_write(self.state, &WriteState::Interfaces, Context::Interfaces)? {
            self.write_interfaces(|_| Ok(()))?;
        }
        Ok(())
    }

    fn write_zero_fields(&mut self) -> Result<(), EncodeError> {
        self.write_zero_interfaces()?;
        if EncodeError::can_write(self.state, &WriteState::Fields, Context::Fields)? {
            self.write_fields(|_| Ok(()))?;
        }
        Ok(())
    }

    fn write_zero_methods(&mut self) -> Result<(), EncodeError> {
        self.write_zero_fields()?;
        if EncodeError::can_write(self.state, &WriteState::Methods, Context::Methods)? {
            self.write_methods(|_| Ok(()))?;
        }
        Ok(())
    }

    fn write_zero_attributes(&mut self) -> Result<(), EncodeError> {
        self.write_zero_methods()?;
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Attributes)? {
            self.write_attributes(|_| Ok(()))?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<Vec<u8>, EncodeError> {
        self.write_zero_attributes()?;
        // we can't move the encoder out of the ClassWriter, so we just replace it
        // this should be fairly cheap or even optimized away by the compiler
        // any code that wouldn't work after calling this method should throw an error
        let encoder = std::mem::replace(&mut self.encoder, VecEncoder::new(Vec::new()));
        Ok(encoder.into_inner())
    }
}

impl EncoderContext for ClassWriter {
    fn class_writer(&self) -> &ClassWriter {
        self
    }

    fn class_writer_mut(&mut self) -> &mut ClassWriter {
        self
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteState {
    // Version numbers
    Start,
    ConstantPool,
    AccessFlags,
    ThisClass,
    SuperClass,
    Interfaces,
    Fields,
    Methods,
    Attributes,
    Finished,
}
