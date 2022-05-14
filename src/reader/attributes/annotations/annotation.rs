use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type Annotations<'input> = DecodeMany<'input, Annotation<'input>, u16>;
pub type AnnotationIter<'input> = DecodeManyIter<'input, Annotation<'input>, u16>;

pub type ParameterAnnotations<'input> = DecodeMany<'input, Annotations<'input>, u8>;
pub type ParameterAnnotationIter<'input> = DecodeManyIter<'input, Annotations<'input>, u8>;

dec_structure! {
    pub struct Annotation<'input> {
        type_: cpool::Index<cpool::Utf8<'input>>,
        pairs: ElementValuePairIter<'input>,
    }
}

pub type ElementValuePairIter<'input> = DecodeManyIter<'input, ElementValuePair<'input>, u16>;

dec_structure! {
    pub struct ElementValuePair<'input> {
        name: cpool::Index<cpool::Utf8<'input>>,
        value: ElementValue<'input>,
    }
}

#[derive(Debug, Clone)]
pub enum ElementValue<'input> {
    Boolean(cpool::Index<cpool::Integer>),
    Byte(cpool::Index<cpool::Integer>),
    Short(cpool::Index<cpool::Integer>),
    Int(cpool::Index<cpool::Integer>),
    Long(cpool::Index<cpool::Long>),
    Float(cpool::Index<cpool::Float>),
    Double(cpool::Index<cpool::Double>),
    Char(cpool::Index<cpool::Integer>),
    String(cpool::Index<cpool::Utf8<'input>>),
    Class(cpool::Index<cpool::Utf8<'input>>),
    Enum {
        type_name: cpool::Index<cpool::Utf8<'input>>,
        const_name: cpool::Index<cpool::Utf8<'input>>,
    },
    Annotation(Annotation<'input>),
    Array(ElementArray<'input>),
}

impl<'input> Decode<'input> for ElementValue<'input> {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
        use ElementValue::*;

        let tag = decoder.read()?;
        let value = match tag {
            b'Z' => Boolean(decoder.read()?),
            b'B' => Byte(decoder.read()?),
            b'S' => Short(decoder.read()?),
            b'I' => Int(decoder.read()?),
            b'J' => Long(decoder.read()?),
            b'F' => Float(decoder.read()?),
            b'D' => Double(decoder.read()?),
            b'C' => Char(decoder.read()?),
            b's' => String(decoder.read()?),
            b'c' => Class(decoder.read()?),
            b'e' => Enum {
                type_name: decoder.read()?,
                const_name: decoder.read()?,
            },
            b'@' => Annotation(decoder.read()?),
            b'[' => Array(decoder.read()?),
            _ => return Err(DecodeError::from_decoder(DecodeErrorKind::InvalidTag, decoder)),
        };
        Ok(value)
    }
}

pub type ElementArray<'input> = DecodeMany<'input, ElementValue<'input>, u16>;
pub type ElementArrayIter<'input> = DecodeManyIter<'input, ElementValue<'input>, u16>;
