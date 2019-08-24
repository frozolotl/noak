use crate::encoding::{DecodeInto, Decoder};
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Exceptions<'a> {
    iter: ExceptionIter<'a>,
}

impl<'a> DecodeInto<'a> for Exceptions<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        let _count: u16 = decoder.read()?;

        Ok(Exceptions {
            iter: ExceptionIter {
                decoder: decoder,
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
