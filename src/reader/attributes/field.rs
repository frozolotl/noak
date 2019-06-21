use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;

#[derive(Debug)]
pub struct ConstantValue {
    value: cpool::Index<cpool::Item<'static>>,
}

impl ConstantValue {
    pub fn value(&self) -> cpool::Index<cpool::Item<'static>> {
        self.value
    }
}

impl<'a> Decode<'a> for ConstantValue {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(ConstantValue {
            value: decoder.read()?,
        })
    }
}
