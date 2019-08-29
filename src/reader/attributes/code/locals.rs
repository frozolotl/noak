use crate::encoding::*;
use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use std::fmt;
use std::ops::Range;

#[derive(Clone)]
pub struct LocalVariableTable<'a> {
    iter: LocalVariableIter<'a>,
}

impl<'a> LocalVariableTable<'a> {
    pub fn iter(&self) -> LocalVariableIter<'a> {
        self.iter.clone()
    }
}

impl<'a> DecodeInto<'a> for LocalVariableTable<'a> {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(LocalVariableTable {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a> fmt::Debug for LocalVariableTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalVariableTable").finish()
    }
}

pub type LocalVariableIter<'a> = DecodeCounted<'a, LocalVariable>;

#[derive(Clone)]
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
            start: code::Index::new(start.into()),
            end: code::Index::new(u32::from(start) + u32::from(end)),
            name,
            descriptor,
            index,
        })
    }
}

impl fmt::Debug for LocalVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalVariable").finish()
    }
}
