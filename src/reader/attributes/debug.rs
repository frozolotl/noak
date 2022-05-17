use crate::reader::{cpool, decoding::*};

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
