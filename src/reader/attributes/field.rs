use crate::encoding::{DecodeInto, Decoder};
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

impl<'a> DecodeInto<'a> for ConstantValue {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(ConstantValue {
            value: decoder.read()?,
        })
    }
}
