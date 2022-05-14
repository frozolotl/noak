use crate::reader::{cpool, decoding::*};

dec_structure! {
    pub struct ConstantValue<'input> into {
        value: cpool::Index<cpool::Item<'static>>,
    }
}
