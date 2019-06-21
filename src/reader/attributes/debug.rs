use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;

#[derive(Debug)]
pub struct SourceFile {
    source_file: cpool::Index<cpool::Utf8<'static>>,
}

impl SourceFile {
    pub fn source_file(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.source_file
    }
}

impl<'a> Decode<'a> for SourceFile {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(SourceFile {
            source_file: decoder.read()?,
        })
    }
}
