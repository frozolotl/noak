use crate::error::*;
use crate::reader::attributes::annotations::ElementValuePairIter;
use crate::reader::decoding::*;
use crate::reader::{attributes::code, cpool};
use std::fmt;
use std::ops::Range;

pub type TypeAnnotations<'a> = DecodeCountedCopy<'a, TypeAnnotation<'a>, u16>;
pub type TypeAnnotationIter<'a> = DecodeCounted<'a, TypeAnnotation<'a>, u16>;

#[derive(Clone)]
pub struct TypeAnnotation<'a> {
    target_type: TargetType,
    target_info: TargetInfo<'a>,
    target_path: TypePath<'a>,
    r#type: cpool::Index<cpool::Utf8<'a>>,
    pairs: ElementValuePairIter<'a>,
}

impl<'a> TypeAnnotation<'a> {
    pub fn target_type(&self) -> TargetType {
        self.target_type
    }

    pub fn target_info(&self) -> &TargetInfo<'a> {
        &self.target_info
    }

    pub fn target_path(&self) -> &TypePath<'a> {
        &self.target_path
    }

    pub fn r#type(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.r#type
    }

    pub fn pairs(&self) -> ElementValuePairIter<'a> {
        self.pairs.clone()
    }
}

impl<'a> Decode<'a> for TypeAnnotation<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        use TargetInfo as TI;
        use TargetType as TT;

        let target_type = decoder.read()?;
        let target_info = match target_type {
            TT::ClassTypeParameter | TT::MethodTypeParameter => TI::TypeParameter {
                parameter_index: decoder.read()?,
            },
            TT::ClassExtends => TI::SuperType {
                supertype_index: decoder.read()?,
            },
            TT::ClassTypeParameterBound | TT::MethodTypeParameterBound => TI::TypeParameterBound {
                type_parameter_index: decoder.read()?,
                bound_index: decoder.read()?,
            },
            TT::Field | TT::MethodReturn | TT::MethodReceiver => TI::Empty,
            TT::LocalVariable | TT::ResourceVariable => TI::LocalVariable(decoder.read()?),
            TT::MethodFormalParameter => TI::FormalParameter {
                formal_parameter_index: decoder.read()?,
            },
            TT::Throws => TI::Throws {
                throws_type_index: decoder.read()?,
            },
            TT::ExceptionParameter => TI::Catch {
                exception_table_index: decoder.read()?,
            },
            TT::InstanceOf | TT::New | TT::ConstructorReference | TT::MethodReference => {
                let offset: u16 = decoder.read()?;
                TI::Offset {
                    offset: code::Index::new(offset.into()),
                }
            }
            TT::Cast
            | TT::ConstructorInvocationTypeArgument
            | TT::MethodInvocationTypeArgument
            | TT::ConstructorReferenceTypeArgument
            | TT::MethodReferenceTypeArgument => {
                let offset: u16 = decoder.read()?;
                TI::TypeArgument {
                    offset: code::Index::new(offset.into()),
                    type_argument_index: decoder.read()?,
                }
            }
        };
        let target_path = decoder.read()?;
        let r#type = decoder.read()?;
        let pairs = decoder.read()?;

        Ok(TypeAnnotation {
            target_type,
            target_info,
            target_path,
            r#type,
            pairs,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TargetType {
    ClassTypeParameter,
    MethodTypeParameter,
    ClassExtends,
    ClassTypeParameterBound,
    MethodTypeParameterBound,
    Field,
    MethodReturn,
    MethodReceiver,
    MethodFormalParameter,
    Throws,
    LocalVariable,
    ResourceVariable,
    ExceptionParameter,
    InstanceOf,
    New,
    ConstructorReference,
    MethodReference,
    Cast,
    ConstructorInvocationTypeArgument,
    MethodInvocationTypeArgument,
    ConstructorReferenceTypeArgument,
    MethodReferenceTypeArgument,
}

impl<'a> Decode<'a> for TargetType {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        use TargetType::*;

        let tag: u8 = decoder.read()?;
        match tag {
            0x00 => Ok(ClassTypeParameter),
            0x01 => Ok(MethodTypeParameter),
            0x10 => Ok(ClassExtends),
            0x11 => Ok(ClassTypeParameterBound),
            0x12 => Ok(MethodTypeParameterBound),
            0x13 => Ok(Field),
            0x14 => Ok(MethodReturn),
            0x15 => Ok(MethodReceiver),
            0x16 => Ok(MethodFormalParameter),
            0x17 => Ok(Throws),
            0x40 => Ok(LocalVariable),
            0x41 => Ok(ResourceVariable),
            0x42 => Ok(ExceptionParameter),
            0x43 => Ok(InstanceOf),
            0x44 => Ok(New),
            0x45 => Ok(ConstructorReference),
            0x46 => Ok(MethodReference),
            0x47 => Ok(Cast),
            0x48 => Ok(ConstructorInvocationTypeArgument),
            0x49 => Ok(MethodInvocationTypeArgument),
            0x4A => Ok(ConstructorReferenceTypeArgument),
            0x4B => Ok(MethodReferenceTypeArgument),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidTag,
                decoder,
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TargetInfo<'a> {
    TypeParameter {
        parameter_index: u8,
    },
    SuperType {
        supertype_index: SuperTypeIndex,
    },
    TypeParameterBound {
        type_parameter_index: u8,
        bound_index: u8,
    },
    Empty,
    FormalParameter {
        formal_parameter_index: u8,
    },
    Throws {
        throws_type_index: u16,
    },
    LocalVariable(LocalVariableTargetTable<'a>),
    Catch {
        exception_table_index: u16,
    },
    Offset {
        offset: code::Index,
    },
    TypeArgument {
        offset: code::Index,
        type_argument_index: u8,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SuperTypeIndex {
    Class,
    Interface { index: u16 },
}

impl<'a> Decode<'a> for SuperTypeIndex {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let index = decoder.read()?;
        if index == u16::max_value() {
            Ok(SuperTypeIndex::Class)
        } else {
            Ok(SuperTypeIndex::Interface { index })
        }
    }
}

pub type LocalVariableTargetTable<'a> = DecodeCountedCopy<'a, LocalVariable, u16>;
pub type LocalVariableTargetIter<'a> = DecodeCounted<'a, LocalVariable, u16>;

#[derive(Clone)]
pub struct LocalVariable {
    start: u16,
    length: u16,
    index: u16,
}

impl LocalVariable {
    pub fn range(&self) -> Range<code::Index> {
        let start = code::Index::new(self.start.into());
        let end = code::Index::new(u32::from(self.start) + u32::from(self.length));
        start..end
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl<'a> Decode<'a> for LocalVariable {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(LocalVariable {
            start: decoder.read()?,
            length: decoder.read()?,
            index: decoder.read()?,
        })
    }
}

impl fmt::Debug for LocalVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalVariable").finish()
    }
}

pub type TypePath<'a> = DecodeCountedCopy<'a, TypePathSegment, u8>;
pub type TypePathSegmentIter<'a> = DecodeCounted<'a, TypePathSegment, u8>;

#[derive(Clone)]
pub struct TypePathSegment {
    kind: TypePathSegmentKind,
    type_argument_index: u8,
}

impl TypePathSegment {
    pub fn kind(&self) -> TypePathSegmentKind {
        self.kind
    }

    pub fn type_argument_index(&self) -> u8 {
        self.type_argument_index
    }
}

impl<'a> Decode<'a> for TypePathSegment {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(TypePathSegment {
            kind: decoder.read()?,
            type_argument_index: decoder.read()?,
        })
    }
}

impl fmt::Debug for TypePathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TypePathSegment").finish()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TypePathSegmentKind {
    ArrayElement,
    InnerType,
    WildcardBound,
    TypeArgument,
}

impl<'a> Decode<'a> for TypePathSegmentKind {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        use TypePathSegmentKind::*;

        let tag: u8 = decoder.read()?;
        match tag {
            0x00 => Ok(ArrayElement),
            0x01 => Ok(InnerType),
            0x02 => Ok(WildcardBound),
            0x03 => Ok(TypeArgument),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidTag,
                decoder,
            )),
        }
    }
}
