use crate::encoding::{Decode, DecodeInto, Decoder};
use crate::error::*;
use crate::reader::attributes::code;
use std::iter::FusedIterator;
use std::fmt;

#[derive(Clone)]
pub struct LineNumberTable<'a> {
    iter: LineNumberIter<'a>,
}

impl<'a> LineNumberTable<'a> {
    pub fn iter(&self) -> LineNumberIter<'a> {
        self.iter.clone()
    }
}

impl<'a> DecodeInto<'a> for LineNumberTable<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        // skip the count
        decoder.advance(2)?;

        Ok(LineNumberTable {
            iter: LineNumberIter { decoder },
        })
    }
}

impl<'a> fmt::Debug for LineNumberTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LineNumberTable").finish()
    }
}

#[derive(Clone)]
pub struct LineNumberIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for LineNumberIter<'a> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for LineNumberIter<'a> {}

#[derive(Clone)]
pub struct Line {
    start: code::Index,
    line_number: u16,
}

impl Line {
    pub fn start(&self) -> code::Index {
        self.start
    }

    pub fn line_number(&self) -> u16 {
        self.line_number
    }
}

impl<'a> Decode<'a> for Line {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let start = code::Index::new(decoder.read::<u16>()?.into());
        let line_number = decoder.read()?;

        Ok(Line { start, line_number })
    }
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Line").finish()
    }
}
