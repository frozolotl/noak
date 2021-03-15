use crate::reader::cpool;

crate::__dec_structure! {
    pub struct ConstantValue<'a> into {
        value: cpool::Index<cpool::Item<'static>>,
    }
}
