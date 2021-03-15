use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

crate::__dec_structure! {
    pub struct Exceptions<'a> into {
        iter: ExceptionIter<'a>,
    }
}

pub type ExceptionIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

pub type MethodParameters<'a> = DecodeCountedCopy<'a, MethodParameter<'a>, u8>;
pub type MethodParameterIter<'a> = DecodeCounted<'a, MethodParameter<'a>, u8>;

crate::__dec_structure! {
    pub struct MethodParameter<'a> {
        name: cpool::Index<cpool::Utf8<'static>>,
        access_flags: AccessFlags,
    }
}
