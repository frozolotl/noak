use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::decoding::*;
use crate::reader::{cpool, AttributeIter};
use std::fmt;

pub type FieldIter<'a> = DecodeCounted<'a, Field<'a>>;

#[derive(Clone)]
pub struct Field<'a> {
    access_flags: AccessFlags,
    name: cpool::Index<cpool::Utf8<'a>>,
    descriptor: cpool::Index<cpool::Utf8<'a>>,
    attributes: AttributeIter<'a>,
}

impl<'a> Field<'a> {
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

impl<'a> Decode<'a> for Field<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Field {
            access_flags: decoder.read()?,
            name: decoder.read()?,
            descriptor: decoder.read()?,
            attributes: decoder.read()?,
        })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        let _access_flags = decoder.skip::<u16>()?;
        let _name = decoder.skip::<u16>()?;
        let _descriptor = decoder.skip::<u16>()?;
        let _attributes = decoder.skip::<AttributeIter>()?;
        Ok(())
    }
}

impl<'a> fmt::Debug for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Field").finish()
    }
}
