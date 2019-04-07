pub mod cpool;

use crate::encoding::*;
use crate::error::*;
use crate::header::{AccessFlags, Version};
use cpool::{ConstantPool, Index};

pub struct Class<'a> {
    version: Version,
    pool: ConstantPool<'a>,
    access_flags: AccessFlags,

}

impl<'a> Class<'a> {
    pub fn read(v: &[u8]) -> Result<Class, DecodeError> {
        let mut decoder = Decoder::new(v, Context::Start);
        let version = read_header(&mut decoder)?;
        decoder.set_context(Context::ConstantPool);
        let pool = decoder.read()?;
        let access_flags = AccessFlags::from_bits(decoder.read()?).unwrap();

        Ok(Class { version, pool, access_flags })
    }

    pub fn version(&self) -> Version {
        self.version
    }
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
        let mut decoder = Decoder::new(
            &[
                // magic bytes
                0xCA, 0xFE, 0xBA, 0xBE, // major version
                0x00, 0x38, // minor version
                0x00, 0x00,
            ],
            Context::Start,
        );

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
        let mut decoder = Decoder::new(
            &[
                // invalid magic bytes
                0xBE, 0xBA, 0xFE, 0xCA,
            ],
            Context::Start,
        );

        assert!(read_header(&mut decoder).is_err());
    }
}
