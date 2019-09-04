pub mod annotations;
mod class;
mod code;
mod debug;
mod field;
mod method;
mod module;

pub use annotations::{AnnotationDefault, Annotations, ParameterAnnotations};
pub use class::*;
pub use code::*;
pub use debug::*;
pub use field::*;
pub use method::*;
pub use module::*;

use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::mutf8::MStr;
use crate::reader::cpool;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct Attribute<'a> {
    name: cpool::Index<cpool::Utf8<'a>>,
    content: Decoder<'a>,
}

impl<'a> Decode<'a> for Attribute<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let name = decoder.read()?;
        let length = decoder.read::<u32>()? as usize;
        let content_decoder = decoder.limit(length, Context::Attributes)?;
        decoder.advance(length)?;
        Ok(Attribute {
            name,
            content: content_decoder,
        })
    }
}

impl<'a> Attribute<'a> {
    pub fn name(&self) -> cpool::Index<cpool::Utf8<'a>> {
        self.name
    }

    pub fn content(&self) -> &'a [u8] {
        self.content.buf()
    }

    pub fn read_content(
        &self,
        pool: &cpool::ConstantPool<'a>,
    ) -> Result<AttributeContent<'a>, DecodeError> {
        let name = pool.get(self.name)?.content;
        let decoder = self.content.with_context(Context::AttributeContent);
        match name.as_bytes() {
            b"AnnotationDefault" => Ok(AttributeContent::AnnotationDefault(decoder.read_into()?)),
            b"BootstrapMethods" => Ok(AttributeContent::BootstrapMethods(decoder.read_into()?)),
            b"Code" => Ok(AttributeContent::Code(decoder.read_into()?)),
            b"ConstantValue" => Ok(AttributeContent::ConstantValue(decoder.read_into()?)),
            b"Deprecated" => Ok(AttributeContent::Deprecated),
            b"EnclosingMethod" => Ok(AttributeContent::EnclosingMethod(decoder.read_into()?)),
            b"Exceptions" => Ok(AttributeContent::Exceptions(decoder.read_into()?)),
            b"InnerClasses" => Ok(AttributeContent::InnerClasses(decoder.read_into()?)),
            b"LineNumberTable" => Ok(AttributeContent::LineNumberTable(decoder.read_into()?)),
            b"LocalVariableTable" => Ok(AttributeContent::LocalVariableTable(decoder.read_into()?)),
            b"ModuleMainClass" => Ok(AttributeContent::ModuleMainClass(decoder.read_into()?)),
            b"ModulePackages" => Ok(AttributeContent::ModulePackages(decoder.read_into()?)),
            b"NestMembers" => Ok(AttributeContent::NestMembers(decoder.read_into()?)),
            b"NestHost" => Ok(AttributeContent::NestHost(decoder.read_into()?)),
            b"RuntimeInvisibleAnnotations" => Ok(AttributeContent::RuntimeInvisibleAnnotations(
                decoder.read_into()?,
            )),
            b"RuntimeInvisibleParameterAnnotations" => Ok(
                AttributeContent::RuntimeInvisibleParameterAnnotations(decoder.read_into()?),
            ),
            b"RuntimeVisibleAnnotations" => Ok(AttributeContent::RuntimeVisibleAnnotations(
                decoder.read_into()?,
            )),
            b"RuntimeVisibleParameterAnnotations" => Ok(
                AttributeContent::RuntimeVisibleParameterAnnotations(decoder.read_into()?),
            ),
            b"Signature" => Ok(AttributeContent::Signature(decoder.read_into()?)),
            b"SourceDebugExtension" => {
                let content = MStr::from_bytes(decoder.buf())?;
                Ok(AttributeContent::SourceDebugExtension(content))
            }
            b"SourceFile" => Ok(AttributeContent::SourceFile(decoder.read_into()?)),
            b"StackMapTable" => Ok(AttributeContent::StackMapTable(decoder.read_into()?),),
            b"Synthetic" => Ok(AttributeContent::Synthetic),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::UnknownAttributeName,
                &self.content,
            )),
        }
    }
}

/// An iterator over the attributes of some item
#[derive(Clone)]
pub struct Attributes<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Decode<'a> for Attributes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let mut attribute_decoder = decoder.clone();
        attribute_decoder.advance(2)?;
        skip_attributes(decoder)?;
        let attribute_length = attribute_decoder.bytes_remaining() - decoder.bytes_remaining();

        Ok(Attributes {
            decoder: attribute_decoder.limit(attribute_length, Context::Attributes)?,
        })
    }
}

pub(in crate::reader) fn skip_attributes(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let count: u16 = decoder.read()?;

    for _ in 0..count {
        // skipping the name
        decoder.advance(2)?;
        let len: u32 = decoder.read()?;
        decoder.advance(len as usize)?;
    }

    Ok(())
}

impl<'a> Iterator for Attributes<'a> {
    type Item = Attribute<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for Attributes<'a> {}

#[derive(Clone)]
pub enum AttributeContent<'a> {
    AnnotationDefault(AnnotationDefault<'a>),
    BootstrapMethods(BootstrapMethods<'a>),
    Code(Code<'a>),
    ConstantValue(ConstantValue),
    Deprecated,
    EnclosingMethod(EnclosingMethod),
    Exceptions(Exceptions<'a>),
    InnerClasses(InnerClasses<'a>),
    LineNumberTable(LineNumberTable<'a>),
    LocalVariableTable(LocalVariableTable<'a>),
    ModuleMainClass(ModuleMainClass),
    ModulePackages(ModulePackages<'a>),
    NestHost(NestHost),
    NestMembers(NestMembers<'a>),
    RuntimeInvisibleAnnotations(Annotations<'a>),
    RuntimeInvisibleParameterAnnotations(ParameterAnnotations<'a>),
    RuntimeVisibleAnnotations(Annotations<'a>),
    RuntimeVisibleParameterAnnotations(ParameterAnnotations<'a>),
    Signature(Signature),
    SourceDebugExtension(&'a MStr),
    SourceFile(SourceFile),
    StackMapTable(StackMapTable<'a>),
    Synthetic,
}
