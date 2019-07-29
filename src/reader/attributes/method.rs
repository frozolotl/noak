use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Exceptions<'a> {
    iter: ExceptionIter<'a>,
}

impl<'a> Decode<'a> for Exceptions<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: u16 = decoder.read()?;
        let limit = count as usize * 2;
        let exceptions_decoder = decoder.limit(limit, Context::AttributeContent)?;
        decoder.advance(limit)?;

        Ok(Exceptions {
            iter: ExceptionIter {
                decoder: exceptions_decoder,
            },
        })
    }
}

impl<'a> Exceptions<'a> {
    pub fn iter(&self) -> ExceptionIter<'a> {
        self.iter.clone()
    }
}

#[derive(Clone)]
pub struct ExceptionIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ExceptionIter<'a> {
    type Item = cpool::Index<cpool::Class>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for ExceptionIter<'a> {}
