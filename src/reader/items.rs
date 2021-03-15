use crate::header::AccessFlags;
use crate::reader::decoding::*;
use crate::reader::{cpool, AttributeIter};

pub type FieldIter<'a> = DecodeCounted<'a, Field<'a>, u16>;

crate::__dec_structure! {
    pub struct Field<'a> {
        access_flags: AccessFlags,
        name: cpool::Index<cpool::Utf8<'a>>,
        descriptor: cpool::Index<cpool::Utf8<'a>>,
        attributes: AttributeIter<'a>,
    }
}

pub type MethodIter<'a> = DecodeCounted<'a, Method<'a>, u16>;

crate::__dec_structure! {
    pub struct Method<'a> {
        access_flags: AccessFlags,
        name: cpool::Index<cpool::Utf8<'a>>,
        descriptor: cpool::Index<cpool::Utf8<'a>>,
        attributes: AttributeIter<'a>,
    }
}
