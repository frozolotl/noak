use crate::error::*;
use crate::encoding::{Decode, Decoder};
use crate::reader::cpool;

#[derive(Debug)]
pub struct SourceFile {
    pub source_file: cpool::Index<cpool::Utf8<'static>>,
}

impl<'a> Decode<'a> for SourceFile {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(SourceFile {
            source_file: decoder.read()?,
        })
    }
}
