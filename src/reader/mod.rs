use crate::encoding::*;
use crate::error::*;
use crate::Version;

pub struct Class {
    version: Version,
}

impl Class {
    pub fn read(v: &[u8]) -> Result<Class, DecodeError> {
        let mut decoder = Decoder::new(v, Context::Start);
        let version = read_header(&mut decoder)?;

        Ok(Class { version })
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

fn read_header(decoder: &mut Decoder) -> Result<Version, DecodeError> {
    const MAGIC: u32 = 0xCAFE_BABE;

    let magic: u32 = decoder.read()?;
    if magic == MAGIC {
        let major = decoder.read()?;
        let minor = decoder.read()?;
        Ok(Version { major, minor })
    } else {
        Err(DecodeError::with_info(
            DecodeErrorKind::InvalidPrefix,
            decoder.file_position(),
            Context::Start,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_header() {
        let mut decoder = Decoder::new(&[
            // magic bytes
            0xCA, 0xFE, 0xBA, 0xBE,
            // major version
            0x00, 0x38,
            // minor version
            0x00, 0x00,
        ], Context::Start);

        let version = read_header(&mut decoder).unwrap();
        assert_eq!(version, Version { major: 0x38, minor: 0 });
    }

    #[test]
    fn invalid_header() {
        let mut decoder = Decoder::new(&[
            // invalid magic bytes
            0xBE, 0xBA, 0xFE, 0xCA,
        ], Context::Start);

        assert!(read_header(&mut decoder).is_err());
    }
}
