use crate::reader::cpool;

crate::__dec_structure! {
    pub struct SourceFile<'a> into {
        source_file: cpool::Index<cpool::Utf8<'static>>,
    }
}

crate::__dec_structure! {
    pub struct Signature<'a> into {
        signature: cpool::Index<cpool::Utf8<'static>>,
    }
}
