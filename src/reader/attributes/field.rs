use crate::reader::{cpool, decoding::*};

dec_structure! {
    pub struct ConstantValue<'a> into {
        value: cpool::Index<cpool::Item<'static>>,
    }
}
