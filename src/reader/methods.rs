use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::decoding::*;
use crate::reader::{cpool, AttributeIter};
use std::fmt;

pub type MethodIter<'a> = DecodeCounted<'a, Method<'a>>;

#[derive(Clone)]
pub struct Method<'a> {
    access_flags: AccessFlags,
    name: cpool::Index<cpool::Utf8<'a>>,
    descriptor: cpool::Index<cpool::Utf8<'a>>,
    attributes: AttributeIter<'a>,
}

impl<'a> Method<'a> {
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    pub fn name(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.name
    }

    pub fn descriptor(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.descriptor
    }

    pub fn attribute_indices(&self) -> AttributeIter<'a> {
        self.attributes.clone()
    }
}

impl<'a> Decode<'a> for Method<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Method {
            access_flags: decoder.read()?,
            name: decoder.read()?,
            descriptor: decoder.read()?,
            attributes: decoder.read()?,
        })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        decoder.skip::<u16>()?; // access flags
        decoder.skip::<u16>()?; // name
        decoder.skip::<u16>()?; // descriptor
        decoder.skip::<AttributeIter>()?; // attributes
        Ok(())
    }
}

impl<'a> fmt::Debug for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Method").finish()
    }
}
