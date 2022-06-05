use std::fmt;

use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::reader::{
    cpool::{self, ConstantPool},
    decoding::*,
    items::{Field, Method},
    Attribute,
};

#[derive(Clone)]
pub struct Class<'input> {
    version: Version,
    pool: ConstantPool<'input>,
    access_flags: AccessFlags,
    this_class: cpool::Index<cpool::Class<'input>>,
    super_class: Option<cpool::Index<cpool::Class<'input>>>,
    interfaces: DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>,
    fields: DecodeMany<'input, Field<'input>, u16>,
    methods: DecodeMany<'input, Method<'input>, u16>,
    attributes: DecodeMany<'input, Attribute<'input>, u16>,
    buffer_size: usize,
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
        decoder.set_context(Context::ConstantPool);
        let pool = decoder.read()?;
        decoder.set_context(Context::ClassInfo);
        let access_flags = decoder.read()?;
        let this_class = decoder.read()?;
        let super_class = decoder.read()?;
        decoder.set_context(Context::Interfaces);
        let interfaces = decoder.read()?;
        decoder.set_context(Context::Fields);
        let fields = decoder.read()?;
        decoder.set_context(Context::Methods);
        let methods = decoder.read()?;
        decoder.set_context(Context::Attributes);
        let attributes = decoder.read()?;
        let buffer_size = decoder.file_position();

        Ok(Class {
            version,
            pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
            buffer_size,
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
    #[must_use]
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
    ///
    /// let item: &cpool::Utf8 = class.pool().get(index)?;
    /// println!("Item: {}", item.content.display());
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn pool(&self) -> &ConstantPool<'input> {
        &self.pool
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
    /// let flags = class.access_flags();
    /// assert!(flags.contains(AccessFlags::PUBLIC | AccessFlags::SUPER));
    ///
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Returns the index of this class name.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    /// use noak::AccessFlags;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// let this_class = class.this_class();
    /// println!("Class: {}", class.pool().retrieve(this_class)?.name.display());
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn this_class(&self) -> cpool::Index<cpool::Class<'input>> {
        self.this_class
    }

    pub fn super_class(&self) -> Option<cpool::Index<cpool::Class<'input>>> {
        self.super_class
    }

    /// Returns an iterator over the interface indices into the constant pool.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// for interface in class.interfaces() {
    ///     let interface = interface?;
    ///     println!("Interface: {}", class.pool().retrieve(interface)?.name.display());
    /// }
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn interfaces(&self) -> DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16> {
        self.interfaces.clone()
    }

    /// Returns an iterator over the fields of this class.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// for field in class.fields() {
    ///     let field = field?;
    ///     println!("Field name: {}", class.pool().retrieve(field.name())?.display());
    /// }
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn fields(&self) -> DecodeMany<'input, Field<'input>, u16> {
        self.fields.clone()
    }

    /// Returns an iterator over the methods of this class.
    ///
    /// ```no_run
    /// use noak::reader::Class;
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// for method in class.methods() {
    ///     let method = method?;
    ///     println!("Method name: {}", class.pool().retrieve(method.name())?.display());
    /// }
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn methods(&self) -> DecodeMany<'input, Method<'input>, u16> {
        self.methods.clone()
    }

    /// Returns an iterator over the attributes of this class.
    ///
    /// ```no_run
    /// use noak::reader::{AttributeContent, Class};
    ///
    /// # let data = &[];
    /// let mut class = Class::new(data)?;
    /// for attribute in class.attributes() {
    ///     let attribute = attribute?;
    ///     match attribute.read_content(class.pool())? {
    ///         AttributeContent::Deprecated(_) => println!("Class is deprecated"),
    ///         _ => {}
    ///     }
    /// }
    /// # Ok::<(), noak::error::DecodeError>(())
    /// ```
    pub fn attributes(&self) -> DecodeMany<'input, Attribute<'input>, u16> {
        self.attributes.clone()
    }

    /// The number of bytes used by the class file.
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl<'input> fmt::Debug for Class<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Class").finish()
    }
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
