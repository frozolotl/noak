use crate::encoding::{Decode, DecodeInto, Decoder};
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;
use std::fmt;

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
            iter: AnnotationIter { decoder: iter_decoder },
        })
    }
}

impl<'a> DecodeInto<'a> for Annotations<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        // skip the count
        decoder.advance(2)?;

        Ok(Annotations {
            iter: AnnotationIter { decoder },
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

#[derive(Clone)]
pub struct AnnotationIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for AnnotationIter<'a> {
    type Item = Annotation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for AnnotationIter<'a> {}

impl<'a> fmt::Debug for AnnotationIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AnnotationIter").finish()
    }
}

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
    pairs: ElementValuePairs<'a>,
}

impl<'a> Annotation<'a> {
    pub fn pairs(&self) -> ElementValuePairs<'a> {
        self.pairs.clone()
    }
}

impl<'a> Decode<'a> for Annotation<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let r#type = decoder.read()?;

        let pair_count: u16 = decoder.read()?;
        let remaining_start = decoder.bytes_remaining();
        let ev_decoder = decoder.clone();

        for _ in 0..pair_count {
            // skip name index
            decoder.advance(2)?;
            skip_element_value(decoder)?;
        }

        Ok(Annotation {
            r#type,
            pairs: ElementValuePairs {
                decoder: ev_decoder.limit(
                    remaining_start - decoder.bytes_remaining(),
                    Context::AttributeContent,
                )?,
            },
        })
    }
}

impl<'a> fmt::Debug for Annotation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Annotation").finish()
    }
}

fn skip_element_value(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let tag = decoder.read()?;
    match tag {
        b'Z' | b'B' | b'S' | b'I' | b'J' | b'F' | b'D' | b'C' | b's' | b'c' => {
            // skip constant pool index
            decoder.advance(2)?;
        }
        b'e' => {
            // skip type and const name indices
            decoder.advance(4)?;
        }
        b'@' => {
            // skip type index
            decoder.advance(2)?;

            let pair_count: u16 = decoder.read()?;
            for _ in 0..pair_count {
                // skip name index
                decoder.advance(2)?;
                skip_element_value(decoder)?;
            }
        }
        b'[' => {
            let count: u16 = decoder.read()?;
            for _ in 0..count {
                skip_element_value(decoder)?;
            }
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

#[derive(Clone)]
pub struct ElementValuePairs<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ElementValuePairs<'a> {
    type Item = ElementValuePair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for ElementValuePairs<'a> {}

impl<'a> fmt::Debug for ElementValuePairs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ElementValuePairs").finish()
    }
}

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
            b'c' => String(decoder.read()?),
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
}

#[derive(Clone)]
pub struct ElementArray<'a> {
    iter: ElementArrayIter<'a>,
}

impl<'a> Decode<'a> for ElementArray<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: u16 = decoder.read()?;

        let ea_decoder = decoder.clone();
        let remaining_start = decoder.bytes_remaining();
        for _ in 0..count {
            skip_element_value(decoder)?;
        }

        Ok(ElementArray {
            iter: ElementArrayIter {
                decoder: ea_decoder.limit(
                    remaining_start - decoder.bytes_remaining(),
                    Context::AttributeContent,
                )?,
            },
        })
    }
}

impl<'a> ElementArray<'a> {
    pub fn iter(&self) -> ElementArrayIter<'a> {
        self.iter.clone()
    }
}

impl<'a> fmt::Debug for ElementArray<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ElementArray").finish()
    }
}

#[derive(Clone)]
pub struct ElementArrayIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ElementArrayIter<'a> {
    type Item = ElementValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for ElementArrayIter<'a> {}

impl<'a> fmt::Debug for ElementArrayIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ElementArrayIter").finish()
    }
}
