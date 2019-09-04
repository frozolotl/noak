use crate::reader::decoding::*;
use crate::error::*;
use crate::reader::cpool;
use crate::header::AccessFlags;
use std::fmt;

#[derive(Clone)]
pub struct Exceptions<'a> {
    iter: ExceptionIter<'a>,
}

impl<'a> DecodeInto<'a> for Exceptions<'a> {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Exceptions {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a> Exceptions<'a> {
    pub fn iter(&self) -> ExceptionIter<'a> {
        self.iter.clone()
    }
}

impl<'a> fmt::Debug for Exceptions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Exceptions").finish()
    }
}

pub type ExceptionIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>>;

pub type MethodParameters<'a> = DecodeCountedCopy<'a, MethodParameter, u8>;
pub type MethodParameterIter<'a> = DecodeCounted<'a, MethodParameter, u8>;

#[derive(Clone)]
pub struct MethodParameter {
    name: cpool::Index<cpool::Utf8<'static>>,
    access_flags: AccessFlags,
}

impl MethodParameter {
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    pub fn name(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.name
    }
}

impl<'a> Decode<'a> for MethodParameter {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(MethodParameter {
            name: decoder.read()?,
            access_flags: decoder.read()?,
        })
    }
}

impl fmt::Debug for MethodParameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MethodParameter").finish()
    }
}
