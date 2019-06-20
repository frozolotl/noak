use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::{attributes, cpool, Attributes};
use std::iter::FusedIterator;

pub struct Method<'a> {
    pub access_flags: AccessFlags,
    pub name: cpool::Index<cpool::Utf8<'a>>,
    pub descriptor: cpool::Index<cpool::Utf8<'a>>,
    attributes: Attributes<'a>,
}

impl<'a> Method<'a> {
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
