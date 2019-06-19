use crate::encoding::{Decoder, Decode};
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Attribute<'a> {
    pub name: cpool::Index<cpool::Utf8<'a>>,
    // the content of the attribute; will be changed later
    content: Decoder<'a>,
}

impl<'a> Decode<'a> for Attribute<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let name = decoder.read()?;
        let length = decoder.read::<u32>()? as usize;
        let content_decoder = decoder.limit(length, Context::Attributes)?;
        decoder.advance(length)?;
        Ok(Attribute {
            name,
            content: content_decoder,
        })
    }
}

/// An iterator over the attributes of some item
#[derive(Clone)]
pub struct Attributes<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Decode<'a> for Attributes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let mut attribute_decoder = decoder.clone();
        attribute_decoder.advance(2)?;
        skip_attributes(decoder)?;
        let attribute_length = attribute_decoder.bytes_remaining() - decoder.bytes_remaining();

        Ok(Attributes {
            decoder: attribute_decoder.limit(attribute_length, Context::Attributes)?,
        })
    }
}

pub(in crate::reader) fn skip_attributes(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let count: u16 = decoder.read()?;

    for _ in 0..count {
        // skipping the name
        decoder.advance(2)?;
        let len: u32 = decoder.read()?;
        decoder.advance(len as usize)?;
    }

    Ok(())
}

impl<'a> Iterator for Attributes<'a> {
    type Item = Attribute<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for Attributes<'a> {}
