use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;

pub type Annotations<'a> = DecodeCountedCopy<'a, Annotation<'a>>;
pub type AnnotationIter<'a> = DecodeCounted<'a, Annotation<'a>>;

pub type ParameterAnnotations<'a> = DecodeCountedCopy<'a, Annotations<'a>, u8>;
pub type ParameterAnnotationIter<'a> = DecodeCounted<'a, Annotations<'a>, u8>;

#[derive(Clone)]
pub struct Annotation<'a> {
    r#type: cpool::Index<cpool::Utf8<'a>>,
    pairs: ElementValuePairIter<'a>,
}

impl<'a> Annotation<'a> {
    pub fn r#type(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.r#type
    }

    pub fn pairs(&self) -> ElementValuePairIter<'a> {
        self.pairs.clone()
    }
}

impl<'a> Decode<'a> for Annotation<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let r#type = decoder.read()?;

        let pair_count: u16 = decoder.read()?;
        let ev_decoder = decoder.clone();

        for _ in 0..pair_count {
            decoder.skip::<cpool::Index<cpool::Utf8>>()?; // name
            decoder.skip::<ElementValue>()?;
        }

        Ok(Annotation {
            r#type,
            pairs: ElementValuePairIter::new(ev_decoder.clone(), pair_count),
        })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        decoder.skip::<cpool::Index<cpool::Utf8>>()?; // type
        decoder.skip::<ElementValuePairIter>()?;
        Ok(())
    }
}

impl<'a> fmt::Debug for Annotation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Annotation").finish()
    }
}

pub type ElementValuePairIter<'a> = DecodeCounted<'a, ElementValuePair<'a>>;

#[derive(Clone)]
pub struct ElementValuePair<'a> {
    name: cpool::Index<cpool::Utf8<'a>>,
    value: ElementValue<'a>,
}

impl<'a> ElementValuePair<'a> {
    pub fn name(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.name
    }

    pub fn value(&self) -> &ElementValue<'a> {
        &self.value
    }
}

impl<'a> Decode<'a> for ElementValuePair<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let name = decoder.read()?;
        let value = decoder.read()?;

        Ok(ElementValuePair { name, value })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        decoder.skip::<cpool::Index<cpool::Utf8>>()?; // name
        decoder.skip::<ElementValue>()?;

        Ok(())
    }
}

impl<'a> fmt::Debug for ElementValuePair<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ElementValuePair").finish()
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
            _ => {
                return Err(DecodeError::from_decoder(
                    DecodeErrorKind::InvalidTag,
                    decoder,
                ))
            }
        };
        Ok(value)
    }

    fn skip(decoder: &mut Decoder) -> Result<(), DecodeError> {
        let tag = decoder.read()?;
        match tag {
            b'Z' | b'B' | b'S' | b'I' | b'J' | b'F' | b'D' | b'C' | b's' | b'c' => {
                decoder.skip::<cpool::Index<cpool::Item>>()?;
            }
            b'e' => {
                decoder.skip::<cpool::Index<cpool::Utf8>>()?; // type name
                decoder.skip::<cpool::Index<cpool::Utf8>>()?; // const name
            }
            b'@' => {
                decoder.skip::<Annotation>()?;
            }
            b'[' => {
                decoder.skip::<ElementArray>()?;
            }
            _ => {
                return Err(DecodeError::from_decoder(
                    DecodeErrorKind::InvalidTag,
                    decoder,
                ))
            }
        }

        Ok(())
    }
}

pub type ElementArray<'a> = DecodeCountedCopy<'a, ElementValue<'a>>;
pub type ElementArrayIter<'a> = DecodeCounted<'a, ElementValue<'a>>;
