use core::fmt;

use crate::{
    mutf8,
    reader::{cpool, decoding::*},
    MStr,
};

use super::FromAttribute;

dec_structure! {
    pub struct Deprecated<'input> into {}
}

impl<'input> FromAttribute<'input> for Deprecated<'input> {
    const NAME: &'static MStr = mutf8!("Deprecated");
}

dec_structure! {
    pub struct Signature<'input> into {
        signature: cpool::Index<cpool::Utf8<'input>>,
    }
}

impl<'input> FromAttribute<'input> for Signature<'input> {
    const NAME: &'static MStr = mutf8!("Signature");
}

dec_structure! {
    pub struct SourceFile<'input> into {
        source_file: cpool::Index<cpool::Utf8<'input>>,
    }
}

impl<'input> FromAttribute<'input> for SourceFile<'input> {
    const NAME: &'static MStr = mutf8!("SourceFile");
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

impl<'input> FromAttribute<'input> for SourceDebugExtension<'input> {
    const NAME: &'static MStr = mutf8!("SourceDebugExtension");
}

impl<'input> fmt::Debug for SourceDebugExtension<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceDebugExtension").finish()
    }
}

dec_structure! {
    pub struct Synthetic<'input> into {}
}
