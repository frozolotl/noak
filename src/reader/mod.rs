pub mod cpool;

use crate::encoding::*;
use crate::error::*;
use crate::header::{AccessFlags, Version};
use crate::mutf8::MStr;
use cpool::ConstantPool;

pub struct Class<'a> {
    read_level: ReadLevel,
    decoder: Decoder<'a>,
    version: Version,
    pool: LazyDecodeRef<ConstantPool<'a>>,
    access_flags: AccessFlags,

    this_class: Option<cpool::Index<cpool::Class>>,
    super_class: Option<cpool::Index<cpool::Class>>,
}

impl<'a> Class<'a> {
    pub fn new(v: &'a [u8]) -> Result<Class, DecodeError> {
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
        })
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn pool(&mut self) -> Result<&ConstantPool<'a>, DecodeError> {
        if self.read_level < ReadLevel::ConstantPool {
            self.read_level = ReadLevel::ConstantPool;
        }

        self.pool.get(&mut self.decoder)
    }

    fn read_info(&mut self) -> Result<(), DecodeError> {
        if self.read_level < ReadLevel::Info {
            // advance the decoder
            self.pool()?;

            self.access_flags = AccessFlags::from_bits(self.decoder.read()?).unwrap();
            self.this_class = Some(self.decoder.read()?);
            self.super_class = Some(self.decoder.read()?);
            self.read_level = ReadLevel::Info;
        }

        Ok(())
    }

    pub fn access_flags(&mut self) -> Result<AccessFlags, DecodeError> {
        self.read_info()?;
        Ok(self.access_flags)
    }

    pub fn this_class_index(&mut self) -> Result<cpool::Index<cpool::Class>, DecodeError> {
        self.read_info()?;
        Ok(self.this_class.unwrap())
    }

    pub fn this_class_name(&mut self) -> Result<&'a MStr, DecodeError> {
        let index = self.this_class_index()?;
        let pool = self.pool()?;
        Ok(pool.get(pool.get(index)?.name)?.content)
    }

    pub fn super_class_index(&mut self) -> Result<cpool::Index<cpool::Class>, DecodeError> {
        self.read_info()?;
        Ok(self.super_class.unwrap())
    }

    pub fn super_class_name(&mut self) -> Result<&'a MStr, DecodeError> {
        let index = self.super_class_index()?;
        let pool = self.pool()?;
        Ok(pool.get(pool.get(index)?.name)?.content)
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
    // Interface table
    Interfaces,
    // The field table
    Fields,
    // The method table
    Methods,
    // The attribute table
    Attributes,
}

fn read_header(decoder: &mut Decoder) -> Result<Version, DecodeError> {
    let magic: u32 = decoder.read()?;
    if magic == 0xCAFE_BABE {
        let major = decoder.read()?;
        let minor = decoder.read()?;
        Ok(Version { major, minor })
    } else {
        Err(DecodeError::from_decoder(
            DecodeErrorKind::InvalidPrefix,
            decoder,
        ))
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
                // major version
                0x00, 0x38,
                // minor version
                0x00, 0x00,
        ], Context::Start);

        let version = read_header(&mut decoder).unwrap();
        assert_eq!(
            version,
            Version {
                major: 0x38,
                minor: 0
            }
        );
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
