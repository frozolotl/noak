use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use std::iter::FusedIterator;
use std::ops::Range;

#[derive(Clone)]
pub struct LocalVariableTable<'a> {
    iter: LocalVariableIter<'a>,
}

impl<'a> Decode<'a> for LocalVariableTable<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: u16 = decoder.read()?;
        let limit = count as usize * 10;
        let lv_decoder = decoder.limit(limit, Context::AttributeContent)?;
        decoder.advance(limit)?;

        Ok(LocalVariableTable {
            iter: LocalVariableIter {
                decoder: lv_decoder,
            },
        })
    }
}

impl<'a> LocalVariableTable<'a> {
    pub fn iter(&self) -> LocalVariableIter<'a> {
        self.iter.clone()
    }
}

#[derive(Clone)]
pub struct LocalVariableIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for LocalVariableIter<'a> {
    type Item = LocalVariable;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for LocalVariableIter<'a> {}

#[derive(Debug, Clone)]
pub struct LocalVariable {
    start: code::Index,
    end: code::Index,
    name: cpool::Index<cpool::Utf8<'static>>,
    descriptor: cpool::Index<cpool::Utf8<'static>>,
    index: u16,
}

impl LocalVariable {
    pub fn range(&self) -> Range<code::Index> {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    pub fn name(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.name
    }

    pub fn descriptor(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.descriptor
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl<'a> Decode<'a> for LocalVariable {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let start: u16 = decoder.read()?;
        let end: u16 = decoder.read()?;
        let name = decoder.read()?;
        let descriptor = decoder.read()?;
        let index = decoder.read()?;

        Ok(LocalVariable {
            start: code::Index::new(start as u32),
            end: code::Index::new(start as u32 + end as u32),
            name,
            descriptor,
            index,
        })
    }
}
