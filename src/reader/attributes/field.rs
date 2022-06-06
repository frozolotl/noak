use crate::{
    mutf8,
    reader::{cpool, decoding::*},
    MStr,
};

use super::FromAttribute;

dec_structure! {
    pub struct ConstantValue<'input> into {
        value: cpool::Index<cpool::Item<'input>>,
    }
}

impl<'input> FromAttribute<'input> for ConstantValue<'input> {
    const NAME: &'static MStr = mutf8!("ConstantValue");
}
