use std::fmt;

use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::mutf8::MStr;
use crate::reader::{
    attributes::AttributeIter,
    cpool::{self, ConstantPool},
    decoding::*,
    items::{FieldIter, MethodIter},
};

#[derive(Clone)]
pub struct Class<'input> {
    read_level: ReadLevel,
    decoder: Decoder<'input>,
    version: Version,
    pool: LazyDecodeRef<ConstantPool<'input>>,
    access_flags: AccessFlags,

    this_class: Option<cpool::Index<cpool::Class>>,
    super_class: Option<cpool::Index<cpool::Class>>,
    interfaces: Option<InterfaceIter<'input>>,
    fields: Option<FieldIter<'input>>,
    methods: Option<MethodIter<'input>>,
    attributes: Option<AttributeIter<'input>>,
}

impl<'input> Class<'input> {
    /// Initializes a class reader.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn new(v: &'input [u8]) -> Result<Class<'input>, DecodeError> {
        let mut decoder = Decoder::new(v, Context::Start);
        let version = read_header(&mut decoder)?;

        Ok(Class {
            read_level: ReadLevel::Start,
            decoder,
            version,
            pool: LazyDecodeRef::NotRead,
            access_flags: AccessFlags::empty(),
            this_class: None,
            super_class: None,
            interfaces: None,
            fields: None,
            methods: None,
            attributes: None,
        })
    }

    /// Returns the class version.
    ///
    /// ```no_run
    /// use noak::{Version, reader::Class};
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// assert_eq!(class.version(), Version::latest());
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the constant pool of this class.
    ///
    /// ```no_run
    /// use noak::reader::{cpool, Class};
    ///
    /// # let data = &[];
    /// # let index = cpool::Index::new(10)?;
    /// let mut class = Class::new(data)?;
    /// let pool = class.pool()?;
    ///
    /// let item: &cpool::Utf8 = pool.get(index)?;
    /// println!("Item: {}", item.content.display());
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn pool(&mut self) -> Result<&ConstantPool<'input>, DecodeError> {
        if self.read_level < ReadLevel::ConstantPool {
            self.read_level = ReadLevel::ConstantPool;
        }

        self.pool.get(&mut self.decoder)
    }

    fn read_info(&mut self) -> Result<(), DecodeError> {
        if self.read_level < ReadLevel::Info {
            // advance the decoder
            self.pool()?;

            self.decoder.set_context(Context::ClassInfo);
            self.access_flags = self.decoder.read()?;
            self.this_class = Some(self.decoder.read()?);
            self.super_class = self.decoder.read()?;
            self.interfaces = Some(self.decoder.read()?);
            self.read_level = ReadLevel::Info;
        }

        Ok(())
    }

    /// Returns the access flags of the class.
    ///
    /// The flags returned may not be valid in this context.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    /// use noak::AccessFlags;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// let flags = class.access_flags()?;
    /// assert!(flags.contains(AccessFlags::PUBLIC | AccessFlags::SUPER));
    ///
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn access_flags(&mut self) -> Result<AccessFlags, DecodeError> {
        self.read_info()?;
        Ok(self.access_flags)
    }

    /// Returns the index of this class name.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    /// use noak::AccessFlags;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// let class_index = class.this_class_index()?;
    /// let pool = class.pool()?;
    /// let name_index = pool.get(class_index)?.name;
    /// let class_name = pool.get(name_index)?.content;
    /// println!("Class: {}", class_name.display());
    ///
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn this_class_index(&mut self) -> Result<cpool::Index<cpool::Class>, DecodeError> {
        self.read_info()?;
        Ok(self.this_class.unwrap())
    }

    pub fn this_class_name(&mut self) -> Result<&'input MStr, DecodeError> {
        let index = self.this_class_index()?;
        let pool = self.pool()?;
        Ok(pool.get(pool.get(index)?.name)?.content)
    }

    pub fn super_class_index(&mut self) -> Result<Option<cpool::Index<cpool::Class>>, DecodeError> {
        self.read_info()?;
        Ok(self.super_class)
    }

    pub fn super_class_name(&mut self) -> Result<Option<&'input MStr>, DecodeError> {
        if let Some(index) = self.super_class_index()? {
            let pool = self.pool()?;
            Ok(Some(pool.get(pool.get(index)?.name)?.content))
        } else {
            Ok(None)
        }
    }

    /// Returns an iterator over the interface indices into the constant pool.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// for interface in class.interfaces()? {
    ///     println!("Interface: {}", class.pool()?.retrieve(interface?)?.name.display());
    /// }
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn interfaces(&mut self) -> Result<InterfaceIter<'input>, DecodeError> {
        self.read_info()?;
        Ok(self.interfaces.clone().unwrap())
    }

    pub fn fields(&mut self) -> Result<FieldIter<'input>, DecodeError> {
        self.read_info()?;
        if self.read_level < ReadLevel::Fields {
            self.decoder.set_context(Context::Fields);
            self.fields = Some(self.decoder.read()?);
            self.read_level = ReadLevel::Fields;
        }
        Ok(self.fields.clone().unwrap())
    }

    pub fn methods(&mut self) -> Result<MethodIter<'input>, DecodeError> {
        if self.read_level < ReadLevel::Methods {
            self.fields()?;
            self.decoder.set_context(Context::Methods);
            self.methods = Some(self.decoder.read()?);
            self.read_level = ReadLevel::Methods;
        }
        Ok(self.methods.clone().unwrap())
    }

    pub fn attributes(&mut self) -> Result<AttributeIter<'input>, DecodeError> {
        if self.read_level < ReadLevel::Attributes {
            self.methods()?;
            self.decoder.set_context(Context::Attributes);
            self.attributes = Some(self.decoder.read()?);
            self.read_level = ReadLevel::Attributes;
        }
        Ok(self.attributes.clone().unwrap())
    }

    /// The count of bytes used by the class file.
    pub fn buffer_size(&mut self) -> Result<usize, DecodeError> {
        self.attributes()?;
        Ok(self.decoder.file_position())
    }
}

impl<'input> fmt::Debug for Class<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Class").finish()
    }
}

/// How much of the class is already read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ReadLevel {
    // Version numbers
    Start,
    ConstantPool,
    // Access Flags, Class Name, Super Class
    Info,
    // The field table
    Fields,
    // The method table
    Methods,
    // The attribute table
    Attributes,
}

fn read_header(decoder: &mut Decoder<'_>) -> Result<Version, DecodeError> {
    let magic: u32 = decoder.read()?;
    if magic == 0xCAFE_BABE {
        let minor = decoder.read()?;
        let major = decoder.read()?;
        Ok(Version { major, minor })
    } else {
        Err(DecodeError::from_decoder(DecodeErrorKind::InvalidPrefix, decoder))
    }
}

pub type InterfaceIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class>, u16>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_header() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
                // magic bytes
                0xCA, 0xFE, 0xBA, 0xBE,
                // minor version
                0x00, 0x00,
                // major version
                0x00, 0x38,
        ], Context::Start);

        let version = read_header(&mut decoder).unwrap();
        assert_eq!(version, Version { major: 0x38, minor: 0 });
    }

    #[test]
    fn invalid_header() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
                // invalid magic bytes
                0xBE, 0xBA, 0xFE, 0xCA,
        ], Context::Start);

        assert!(read_header(&mut decoder).is_err());
    }
}
