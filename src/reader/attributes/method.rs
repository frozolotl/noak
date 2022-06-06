use crate::header::AccessFlags;
use crate::mutf8;
use crate::reader::cpool;
use crate::reader::decoding::*;
use crate::MStr;

use super::FromAttribute;

dec_structure! {
    pub struct Exceptions<'input> into {
        exceptions: DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>,
    }
}

impl<'input> FromAttribute<'input> for Exceptions<'input> {
    const NAME: &'static MStr = mutf8!("Exceptions");
}

dec_structure! {
    pub struct MethodParameters<'input> into {
        parameters: DecodeMany<'input, MethodParameter<'input>, u8>,
    }
}

impl<'input> FromAttribute<'input> for MethodParameters<'input> {
    const NAME: &'static MStr = mutf8!("MethodParameters");
}

dec_structure! {
    pub struct MethodParameter<'input> {
        name: cpool::Index<cpool::Utf8<'input>>,
        access_flags: AccessFlags,
    }
}
