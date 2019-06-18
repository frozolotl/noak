use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::{cpool, attributes};
use std::iter::FusedIterator;

pub struct Field {
    pub access_flags: AccessFlags,
    pub name: cpool::Index<cpool::Utf8<'static>>,
    pub descriptor: cpool::Index<cpool::Utf8<'static>>,
}

impl<'a> Decode<'a> for Field {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Field {
            access_flags: decoder.read()?,
            name: decoder.read()?,
            descriptor: decoder.read()?,
        })
    }
}

/// An iterator over the attributes of some item
#[derive(Clone)]
pub struct Fields<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Decode<'a> for Fields<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let field_decoder = decoder.clone();
        skip_fields(decoder)?;
        let field_length = field_decoder.bytes_remaining() - decoder.bytes_remaining();

        Ok(Fields {
            decoder: field_decoder.limit(field_length, Context::Fields)?,
        })
    }
}

fn skip_fields(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let count: u16 = decoder.read()?;

    for _ in 0..count {
        // skipping the access flags, name and descriptor
        decoder.advance(6)?;
        attributes::skip_attributes(decoder)?;
    }

    Ok(())
}

impl<'a> Iterator for Fields<'a> {
    type Item = Field;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> ExactSizeIterator for Fields<'a> {}

impl<'a> FusedIterator for Fields<'a> {}
