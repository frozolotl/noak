use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::{attributes, cpool, Attributes};
use std::fmt;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Method<'a> {
    access_flags: AccessFlags,
    name: cpool::Index<cpool::Utf8<'a>>,
    descriptor: cpool::Index<cpool::Utf8<'a>>,
    attributes: Attributes<'a>,
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

    pub fn attribute_indices(&self) -> Attributes<'a> {
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
}

impl<'a> fmt::Debug for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Method").finish()
    }
}

/// An iterator over the methods of a class
#[derive(Clone)]
pub struct Methods<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Decode<'a> for Methods<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let mut method_decoder = decoder.clone();
        method_decoder.advance(2)?;
        skip_methods(decoder)?;
        let method_length = method_decoder.bytes_remaining() - decoder.bytes_remaining();

        Ok(Methods {
            decoder: method_decoder.limit(method_length, Context::Fields)?,
        })
    }
}

fn skip_methods(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let count: u16 = decoder.read()?;

    for _ in 0..count {
        // skipping the access flags, name and descriptor
        decoder.advance(6)?;
        attributes::skip_attributes(decoder)?;
    }

    Ok(())
}

impl<'a> Iterator for Methods<'a> {
    type Item = Method<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for Methods<'a> {}

impl<'a> fmt::Debug for Methods<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Methods").finish()
    }
}
