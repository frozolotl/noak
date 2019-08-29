use crate::encoding::{DecodeInto, Decoder};
use crate::error::*;
use crate::reader::cpool;
use std::fmt;

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

impl<'a> DecodeInto<'a> for EnclosingMethod {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(EnclosingMethod {
            class: decoder.read()?,
            method: decoder.read()?,
        })
    }
}

impl fmt::Debug for EnclosingMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EnclosingMethod").finish()
    }
}

pub struct NestHost {
    host_class: cpool::Index<cpool::Class>,
}

impl NestHost {
    pub fn host_class(&self) -> cpool::Index<cpool::Class> {
        self.host_class
    }
}

impl<'a> DecodeInto<'a> for NestHost {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(NestHost {
            host_class: decoder.read()?,
        })
    }
}

impl fmt::Debug for NestHost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NestHost").finish()
    }
}
