use crate::encoding::*;
use crate::error::*;
use crate::reader::cpool;
use std::fmt;

#[derive(Clone)]
pub struct Exceptions<'a> {
    iter: ExceptionIter<'a>,
}

impl<'a> DecodeInto<'a> for Exceptions<'a> {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Exceptions {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a> Exceptions<'a> {
    pub fn iter(&self) -> ExceptionIter<'a> {
        self.iter.clone()
    }
}

impl<'a> fmt::Debug for Exceptions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Exceptions").finish()
    }
}

type ExceptionIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>>;
