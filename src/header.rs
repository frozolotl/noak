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
    /// The latest version which is guaranteed to work with this library.
    pub fn latest() -> Version {
        Version {
            major: 56,
            minor: 0,
        }
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
