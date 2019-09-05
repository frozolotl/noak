use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::{DecodeInto, Decoder};
use std::fmt;

#[derive(Clone)]
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

impl fmt::Debug for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConstantValue").finish()
    }
}
