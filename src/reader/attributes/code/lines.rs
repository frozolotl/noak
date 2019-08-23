use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use std::iter::FusedIterator;
use std::ops::Range;

#[derive(Clone)]
pub struct LineNumberTable<'a> {
    iter: LineNumberIter<'a>,
}

impl<'a> Decode<'a> for LineNumberTable<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: u16 = decoder.read()?;
        let limit = count as usize * 4;
        let ln_decoder = decoder.limit(limit, Context::AttributeContent)?;
        decoder.advance(limit)?;

        Ok(LineNumberTable {
            iter: LineNumberIter {
                decoder: ln_decoder,
            },
        })
    }
}

impl<'a> LineNumberTable<'a> {
    pub fn iter(&self) -> LineNumberIter<'a> {
        self.iter.clone()
    }
}

#[derive(Clone)]
pub struct LineNumberIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for LineNumberIter<'a> {
    type Item = LineNumber;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for LineNumberIter<'a> {}

#[derive(Debug, Clone)]
pub struct LineNumber {
    index: code::Index,
    line_number: u16,
}

impl LineNumber {
    pub fn index(&self) -> code::Index {
        self.index
    }

    pub fn line_number(&self) -> u16 {
        self.line_number
    }
}

impl<'a> Decode<'a> for LineNumber {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let index = decoder.read()?;
        let line_number = decoder.read()?;

        Ok(LineNumber {
            index,
            line_number,
        })
    }
}
