use crate::encoding::{DecodeInto, Decoder};
use crate::error::*;
use crate::reader::attributes::annotations::ElementValue;
use std::fmt;

pub struct AnnotationDefault<'a> {
    value: ElementValue<'a>,
}

impl<'a> AnnotationDefault<'a> {
    pub fn value(&self) -> &ElementValue<'a> {
        &self.value
    }
}

impl<'a> DecodeInto<'a> for AnnotationDefault<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(AnnotationDefault {
            value: decoder.read()?,
        })
    }
}

impl<'a> fmt::Debug for AnnotationDefault<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AnnotationDefault").finish()
    }
}
