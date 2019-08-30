use crate::encoding::*;
use crate::error::*;
use crate::reader::cpool;
use std::fmt;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Annotations<'a> {
    iter: AnnotationIter<'a>,
}

impl<'a> Decode<'a> for Annotations<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: u16 = decoder.read()?;
        let iter_decoder = decoder.clone();
        for _ in 0..count {
            decoder.read::<Annotation>()?;
        }

        Ok(Annotations {
            iter: AnnotationIter::new(iter_decoder, count),
        })
    }
}

impl<'a> DecodeInto<'a> for Annotations<'a> {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Annotations {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a> Annotations<'a> {
    pub fn iter(&self) -> AnnotationIter<'a> {
        self.iter.clone()
    }
}

impl<'a> fmt::Debug for Annotations<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Annotations").finish()
    }
}

pub type AnnotationIter<'a> = DecodeCounted<'a, Annotation<'a>>;

#[derive(Clone)]
pub struct ParameterAnnotations<'a> {
    iter: ParameterAnnotationIter<'a>,
}

impl<'a> DecodeInto<'a> for ParameterAnnotations<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        // skip the count
        decoder.advance(1)?;

        Ok(ParameterAnnotations {
            iter: ParameterAnnotationIter { decoder },
        })
    }
}

impl<'a> ParameterAnnotations<'a> {
    pub fn iter(&self) -> ParameterAnnotationIter<'a> {
        self.iter.clone()
    }
}

impl<'a> fmt::Debug for ParameterAnnotations<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParameterAnnotations").finish()
    }
}

#[derive(Clone)]
pub struct ParameterAnnotationIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ParameterAnnotationIter<'a> {
    type Item = Annotations<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for ParameterAnnotationIter<'a> {}

impl<'a> fmt::Debug for ParameterAnnotationIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParameterAnnotationIter").finish()
    }
}

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
            let _name = decoder.skip::<cpool::Index<cpool::Utf8>>()?;
            let _value = decoder.skip::<ElementValue>()?;
        }

        Ok(Annotation {
            r#type,
            pairs: ElementValuePairIter::new(
                ev_decoder.clone(),
                pair_count
            )
        })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        let _type = decoder.skip::<cpool::Index<cpool::Utf8>>()?;
        let _pairs = decoder.skip::<ElementValuePairIter>()?;
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
        let _name = decoder.skip::<cpool::Index<cpool::Utf8>>()?;
        let _value = decoder.skip::<ElementValue>()?;

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
                let _index = decoder.skip::<cpool::Index<cpool::Item>>()?;
            }
            b'e' => {
                let _type_name = decoder.skip::<cpool::Index<cpool::Utf8>>()?;
                let _const_name = decoder.skip::<cpool::Index<cpool::Utf8>>()?;
            }
            b'@' => {
                let _annotation = decoder.skip::<Annotation>()?;
            }
            b'[' => {
                let _array = decoder.skip::<ElementArray>()?;
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
