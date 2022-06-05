use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

dec_structure! {
    pub struct Exceptions<'input> into {
        exceptions: DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>,
    }
}

dec_structure! {
    pub struct MethodParameters<'input> into {
        parameters: DecodeMany<'input, MethodParameter<'input>, u8>,
    }
}

dec_structure! {
    pub struct MethodParameter<'input> {
        name: cpool::Index<cpool::Utf8<'input>>,
        access_flags: AccessFlags,
    }
}
