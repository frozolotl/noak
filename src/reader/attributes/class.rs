use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;

#[derive(Debug)]
pub struct EnclosingMethod {
    class: cpool::Index<cpool::Class>,
    method: cpool::Index<cpool::NameAndType>,
}

impl EnclosingMethod {
    pub fn class(&self) -> cpool::Index<cpool::Class> {
        self.class
    }

    pub fn method(&self) -> cpool::Index<cpool::NameAndType> {
        self.method
    }
}

impl<'a> Decode<'a> for EnclosingMethod {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(EnclosingMethod {
            class: decoder.read()?,
            method: decoder.read()?,
        })
    }
}

#[derive(Debug)]
pub struct NestHost {
    host_class: cpool::Index<cpool::Class>,
}

impl NestHost {
    pub fn host_class(&self) -> cpool::Index<cpool::Class> {
        self.host_class
    }
}

impl<'a> Decode<'a> for NestHost {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(NestHost {
            host_class: decoder.read()?,
        })
    }
}
