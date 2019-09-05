use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::{DecodeInto, Decoder};
use std::fmt;

#[derive(Clone)]
pub struct SourceFile {
    source_file: cpool::Index<cpool::Utf8<'static>>,
}

impl SourceFile {
    pub fn source_file(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.source_file
    }
}

impl<'a> DecodeInto<'a> for SourceFile {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(SourceFile {
            source_file: decoder.read()?,
        })
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SourceFile").finish()
    }
}

#[derive(Clone)]
pub struct Signature {
    signature: cpool::Index<cpool::Utf8<'static>>,
}

impl Signature {
    pub fn signature(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.signature
    }
}

impl<'a> DecodeInto<'a> for Signature {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(Signature {
            signature: decoder.read()?,
        })
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Signature").finish()
    }
}
