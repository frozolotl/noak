use crate::header::AccessFlags;
use crate::reader::decoding::*;
use crate::reader::{cpool, Attribute};

dec_structure! {
    pub struct Field<'input> {
        access_flags: AccessFlags,
        name: cpool::Index<cpool::Utf8<'input>>,
        descriptor: cpool::Index<cpool::Utf8<'input>>,
        attributes: DecodeMany<'input, Attribute<'input>, u16>,
    }
}

dec_structure! {
    pub struct Method<'input> {
        access_flags: AccessFlags,
        name: cpool::Index<cpool::Utf8<'input>>,
        descriptor: cpool::Index<cpool::Utf8<'input>>,
        attributes: DecodeMany<'input, Attribute<'input>, u16>,
    }
}
