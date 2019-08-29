use crate::encoding::{DecodeInto, Decoder};
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;
use std::fmt;

#[derive(Clone)]
pub struct Exceptions<'a> {
    iter: ExceptionIter<'a>,
}

impl<'a> DecodeInto<'a> for Exceptions<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        // skip exception count
        decoder.advance(2)?;
        Ok(Exceptions {
            iter: ExceptionIter { decoder },
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

#[derive(Clone)]
pub struct ExceptionIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ExceptionIter<'a> {
    type Item = cpool::Index<cpool::Class>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for ExceptionIter<'a> {}

impl<'a> fmt::Debug for ExceptionIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExceptionIter").finish()
    }
}
