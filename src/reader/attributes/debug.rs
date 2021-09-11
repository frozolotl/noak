use crate::reader::{cpool, decoding::*};

dec_structure! {
    pub struct SourceFile<'a> into {
        source_file: cpool::Index<cpool::Utf8<'static>>,
    }
}

dec_structure! {
    pub struct Signature<'a> into {
        signature: cpool::Index<cpool::Utf8<'static>>,
    }
}
