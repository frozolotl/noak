use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

dec_structure! {
    pub struct Exceptions<'a> into {
        iter: ExceptionIter<'a>,
    }
}

pub type ExceptionIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Class>, u16>;

pub type MethodParameters<'a> = DecodeMany<'a, MethodParameter<'a>, u8>;
pub type MethodParameterIter<'a> = DecodeManyIter<'a, MethodParameter<'a>, u8>;

dec_structure! {
    pub struct MethodParameter<'a> {
        name: cpool::Index<cpool::Utf8<'static>>,
        access_flags: AccessFlags,
    }
}
