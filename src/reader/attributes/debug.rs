use core::fmt;

use crate::{
    reader::{cpool, decoding::*},
    MStr,
};

dec_structure! {
    pub struct SourceFile<'input> into {
        source_file: cpool::Index<cpool::Utf8<'input>>,
    }
}

dec_structure! {
    pub struct Signature<'input> into {
        signature: cpool::Index<cpool::Utf8<'input>>,
    }
}

#[derive(Clone)]
pub struct SourceDebugExtension<'input> {
    content: &'input MStr,
}

impl<'input> SourceDebugExtension<'input> {
    pub fn content(&self) -> &'input MStr {
        self.content
    }
}

impl<'input> DecodeInto<'input> for SourceDebugExtension<'input> {
    fn decode_into(decoder: Decoder<'input>) -> Result<Self, crate::error::DecodeError> {
        Ok(SourceDebugExtension {
            content: MStr::from_mutf8(decoder.buf())?,
        })
    }
}

impl<'input> fmt::Debug for SourceDebugExtension<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceDebugExtension").finish()
    }
}
