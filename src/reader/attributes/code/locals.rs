use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;
use std::ops::Range;

dec_structure! {
    pub struct LocalVariableTable<'input> into {
        locals: DecodeMany<'input, LocalVariable, u16>,
    }
}

#[derive(Clone)]
pub struct LocalVariable {
    start: code::Index,
    end: code::Index,
    name: cpool::Index<cpool::Utf8<'static>>,
    descriptor: cpool::Index<cpool::Utf8<'static>>,
    index: u16,
}

impl LocalVariable {
    #[must_use]
    pub fn range(&self) -> Range<code::Index> {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    #[must_use]
    pub fn name(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.name
    }

    #[must_use]
    pub fn descriptor(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.descriptor
    }

    #[must_use]
    pub fn index(&self) -> u16 {
        self.index
    }
}

impl<'input> Decode<'input> for LocalVariable {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalVariable").finish()
    }
}

dec_structure! {
    pub struct LocalVariableTypeTable<'input> into {
        locals: DecodeMany<'input, LocalVariableType<'input>, u16>,
    }
}

#[derive(Clone)]
pub struct LocalVariableType<'input> {
    start: code::Index,
    end: code::Index,
    name: cpool::Index<cpool::Utf8<'input>>,
    signature: cpool::Index<cpool::Utf8<'input>>,
    index: u16,
}

impl<'input> LocalVariableType<'input> {
    #[must_use]
    pub fn range(&self) -> Range<code::Index> {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    #[must_use]
    pub fn name(&self) -> cpool::Index<cpool::Utf8<'input>> {
        self.name
    }

    #[must_use]
    pub fn signature(&self) -> cpool::Index<cpool::Utf8<'input>> {
        self.signature
    }

    #[must_use]
    pub fn index(&self) -> u16 {
        self.index
    }
}

impl<'input> Decode<'input> for LocalVariableType<'input> {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
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

impl<'input> fmt::Debug for LocalVariableType<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalVariableType").finish()
    }
}
