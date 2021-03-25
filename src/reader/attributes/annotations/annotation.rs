use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type Annotations<'a> = DecodeCountedCopy<'a, Annotation<'a>, u16>;
pub type AnnotationIter<'a> = DecodeCounted<'a, Annotation<'a>, u16>;

pub type ParameterAnnotations<'a> = DecodeCountedCopy<'a, Annotations<'a>, u8>;
pub type ParameterAnnotationIter<'a> = DecodeCounted<'a, Annotations<'a>, u8>;

crate::__dec_structure! {
    pub struct Annotation<'a> {
        type_: cpool::Index<cpool::Utf8<'a>>,
        pairs: ElementValuePairIter<'a>,
    }
}

pub type ElementValuePairIter<'a> = DecodeCounted<'a, ElementValuePair<'a>, u16>;

crate::__dec_structure! {
    pub struct ElementValuePair<'a> {
        name: cpool::Index<cpool::Utf8<'a>>,
        value: ElementValue<'a>,
    }
}

#[derive(Debug, Clone)]
pub enum ElementValue<'a> {
    Boolean(cpool::Index<cpool::Integer>),
    Byte(cpool::Index<cpool::Integer>),
    Short(cpool::Index<cpool::Integer>),
    Int(cpool::Index<cpool::Integer>),
    Long(cpool::Index<cpool::Long>),
    Float(cpool::Index<cpool::Float>),
    Double(cpool::Index<cpool::Double>),
    Char(cpool::Index<cpool::Integer>),
    String(cpool::Index<cpool::Utf8<'a>>),
    Class(cpool::Index<cpool::Utf8<'a>>),
    Enum {
        type_name: cpool::Index<cpool::Utf8<'a>>,
        const_name: cpool::Index<cpool::Utf8<'a>>,
    },
    Annotation(Annotation<'a>),
    Array(ElementArray<'a>),
}

impl<'a> Decode<'a> for ElementValue<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
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

pub type ElementArray<'a> = DecodeCountedCopy<'a, ElementValue<'a>, u16>;
pub type ElementArrayIter<'a> = DecodeCounted<'a, ElementValue<'a>, u16>;
