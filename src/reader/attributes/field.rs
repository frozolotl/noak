use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;

#[derive(Debug)]
pub struct ConstantValue {
    pub value: cpool::Index<cpool::Item<'static>>,
}

impl<'a> Decode<'a> for ConstantValue {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(ConstantValue {
            value: decoder.read()?,
        })
    }
}
