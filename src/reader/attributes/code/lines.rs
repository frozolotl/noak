use crate::encoding::*;
use crate::error::*;
use crate::reader::attributes::code;
use std::fmt;
use std::iter::FusedIterator;

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
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(LineNumberTable {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a> fmt::Debug for LineNumberTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LineNumberTable").finish()
    }
}

pub type LineNumberIter<'a> = DecodeCounted<'a, Line>;

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
