use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

dec_structure! {
    pub struct Exceptions<'input> into {
        iter: ExceptionIter<'input>,
    }
}

pub type ExceptionIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class<'input>>, u16>;

pub type MethodParameters<'input> = DecodeMany<'input, MethodParameter<'input>, u8>;
pub type MethodParameterIter<'input> = DecodeManyIter<'input, MethodParameter<'input>, u8>;

dec_structure! {
    pub struct MethodParameter<'input> {
        name: cpool::Index<cpool::Utf8<'input>>,
        access_flags: AccessFlags,
    }
}
