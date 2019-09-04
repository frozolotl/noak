use crate::encoding::*;
use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use std::fmt;
use std::ops::Range;

pub type LocalVariableTable<'a> = DecodeCountedCopy<'a, LocalVariable>;
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

pub type LocalVariableTypeTable<'a> = DecodeCountedCopy<'a, LocalVariableType>;
pub type LocalVariableTypeIter<'a> = DecodeCounted<'a, LocalVariableType>;

#[derive(Clone)]
pub struct LocalVariableType {
    start: code::Index,
    end: code::Index,
    name: cpool::Index<cpool::Utf8<'static>>,
    signature: cpool::Index<cpool::Utf8<'static>>,
    index: u16,
}

impl LocalVariableType {
    pub fn range(&self) -> Range<code::Index> {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    pub fn name(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.name
    }

    pub fn signature(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.signature
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl<'a> Decode<'a> for LocalVariableType {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let start: u16 = decoder.read()?;
        let end: u16 = decoder.read()?;
        let name = decoder.read()?;
        let signature = decoder.read()?;
        let index = decoder.read()?;

        Ok(LocalVariableType {
            start: code::Index::new(start.into()),
            end: code::Index::new(u32::from(start) + u32::from(end)),
            name,
            signature,
            index,
        })
    }
}

impl fmt::Debug for LocalVariableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalVariableType").finish()
    }
}
