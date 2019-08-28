use crate::encoding::{DecodeInto, Decoder};
use crate::error::*;
use crate::reader::attributes::annotations::ElementValue;

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
