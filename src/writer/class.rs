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

#[derive(Clone)]
pub struct ClassWriter {
    pub(crate) encoder: VecEncoder,
    state: WriteState,

    pool: ConstantPool,
    pub(crate) pool_end: Offset,
}

impl ClassWriter {
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::with_capacity(capacity),
            state: WriteState::Start,
            pool: ConstantPool::new(),
            pool_end: EMPTY_POOL_END,
        }
    }

    pub fn write_version(&mut self, version: Version) -> Result<&mut ClassWriter, EncodeError> {
        if self.state == WriteState::Start {
            self.encoder.write(0xCAFEBABEu32)?;
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

    fn write_empty_pool(&mut self) -> Result<&mut ClassWriter, EncodeError> {
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

    pub fn write_access_flags(
        &mut self,
        flags: AccessFlags,
    ) -> Result<&mut ClassWriter, EncodeError> {
        self.write_empty_pool()?;
        EncodeError::result_from_state(self.state, &WriteState::AccessFlags, Context::ClassInfo)?;
        self.encoder.write(flags)?;
        self.state = WriteState::ThisClass;
        Ok(self)
    }

    pub fn write_this_class<I>(&mut self, name: I) -> Result<&mut ClassWriter, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::ThisClass, Context::ClassInfo)?;
        let index = name.insert(self)?;
        self.encoder.write(index)?;
        self.state = WriteState::SuperClass;
        Ok(self)
    }

    pub fn write_super_class<I>(&mut self, name: I) -> Result<&mut ClassWriter, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        EncodeError::result_from_state(self.state, &WriteState::SuperClass, Context::ClassInfo)?;
        let index = name.insert(self)?;
        self.encoder.write(index)?;
        self.state = WriteState::Interfaces;
        Ok(self)
    }

    pub fn write_no_super_class(&mut self) -> Result<&mut ClassWriter, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteState::SuperClass, Context::ClassInfo)?;
        self.encoder
            .write::<Option<cpool::Index<cpool::Class>>>(None)?;
        self.state = WriteState::Interfaces;
        Ok(self)
    }

    pub fn write_interfaces<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, InterfaceWriter<'f>>) -> Result<(), EncodeError>,
    {
        EncodeError::result_from_state(self.state, &WriteState::Interfaces, Context::Interfaces)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Fields;
        Ok(self)
    }

    pub fn write_fields<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, FieldWriter<'f>>) -> Result<(), EncodeError>,
    {
        self.write_zero_interfaces()?;
        EncodeError::result_from_state(self.state, &WriteState::Fields, Context::Fields)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Methods;
        Ok(self)
    }

    pub fn write_methods<F>(&mut self, f: F) -> Result<&mut ClassWriter, EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, MethodWriter<'f>>) -> Result<(), EncodeError>,
    {
        self.write_zero_fields()?;
        EncodeError::result_from_state(self.state, &WriteState::Methods, Context::Methods)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Attributes;
        Ok(self)
    }

    pub fn write_attributes<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: for<'f> FnOnce(&mut CountedWriter<'f, AttributeWriter<'f>>) -> Result<(), EncodeError>,
    {
        self.write_zero_methods()?;
        EncodeError::result_from_state(self.state, &WriteState::Attributes, Context::Attributes)?;
        let mut builder = CountedWriter::new(self)?;
        f(&mut builder)?;
        self.state = WriteState::Finished;

        Ok(())
    }

    pub fn write_zero_interfaces(&mut self) -> Result<(), EncodeError> {
        if EncodeError::can_write(self.state, &WriteState::Interfaces, Context::Interfaces)? {
            self.write_interfaces(|_| Ok(()))?;
        }
        Ok(())
    }

    pub fn write_zero_fields(&mut self) -> Result<(), EncodeError> {
        if EncodeError::can_write(self.state, &WriteState::Fields, Context::Fields)? {
            self.write_fields(|_| Ok(()))?;
        }
        Ok(())
    }

    pub fn write_zero_methods(&mut self) -> Result<(), EncodeError> {
        if EncodeError::can_write(self.state, &WriteState::Methods, Context::Methods)? {
            self.write_methods(|_| Ok(()))?;
        }
        Ok(())
    }

    pub fn write_zero_attributes(&mut self) -> Result<(), EncodeError> {
        if EncodeError::can_write(self.state, &WriteState::Attributes, Context::Attributes)? {
            self.write_attributes(|_| Ok(()))?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        self.write_zero_attributes()?;
        Ok(self.encoder.into_inner())
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
