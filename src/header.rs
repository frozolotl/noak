use crate::error::*;
use crate::reader::decoding::{Decode, Decoder};
use crate::writer::encoding::{Encode, Encoder};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Version {
    pub const V1_0_2: Version = Version { major: 45, minor: 3 };
    pub const V1_1: Version = Version { major: 45, minor: 3 };
    pub const V1_2: Version = Version { major: 46, minor: 0 };
    pub const V1_3: Version = Version { major: 47, minor: 0 };
    pub const V1_4: Version = Version { major: 48, minor: 0 };
    pub const V5_0: Version = Version { major: 49, minor: 0 };
    pub const V6: Version = Version { major: 50, minor: 0 };
    pub const V7: Version = Version { major: 51, minor: 0 };
    pub const V8: Version = Version { major: 52, minor: 0 };
    pub const V9: Version = Version { major: 53, minor: 0 };
    pub const V10: Version = Version { major: 54, minor: 0 };
    pub const V11: Version = Version { major: 55, minor: 0 };
    pub const V12: Version = Version { major: 56, minor: 0 };
    pub const V13: Version = Version { major: 57, minor: 0 };
    pub const V14: Version = Version { major: 58, minor: 0 };
    pub const V15: Version = Version { major: 59, minor: 0 };
    pub const V16: Version = Version { major: 60, minor: 0 };
    pub const V17: Version = Version { major: 61, minor: 0 };
    pub const V18: Version = Version { major: 62, minor: 0 };

    /// The latest version which is guaranteed to work with this library.
    /// Changes of this value are not considered breaking changes.
    #[must_use]
    pub const fn latest() -> Version {
        Version::V18
    }

    #[must_use]
    pub fn is_preview(self) -> bool {
        self.major >= Version::V12.major && self.minor == 65535
    }
}

bitflags! {
    pub struct AccessFlags: u16 {
        const PUBLIC       = 1;
        const PRIVATE      = 1 << 1;
        const PROTECTED    = 1 << 2;
        const STATIC       = 1 << 3;
        const FINAL        = 1 << 4;
        const SUPER        = 1 << 5;
        const SYNCHRONIZED = 1 << 5;
        const BRIDGE       = 1 << 6;
        const VOLATILE     = 1 << 6;
        const VARARGS      = 1 << 7;
        const TRANSIENT    = 1 << 7;
        const NATIVE       = 1 << 8;
        const INTERFACE    = 1 << 9;
        const ABSTRACT     = 1 << 10;
        const STRICT       = 1 << 11;
        const SYNTHETIC    = 1 << 12;
        const ANNOTATION   = 1 << 13;
        const ENUM         = 1 << 14;
        const MANDATED     = 1 << 15;
        const MODULE       = 1 << 15;
    }
}

impl<'a> Decode<'a> for AccessFlags {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(AccessFlags::from_bits(decoder.read()?).unwrap())
    }
}

impl Encode for AccessFlags {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write(self.bits())?;
        Ok(())
    }
}
