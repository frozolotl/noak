pub mod annotations;
mod class;
mod code;
mod debug;
mod field;
mod method;
mod module;

pub use annotations::{AnnotationDefault, Annotations, ParameterAnnotations, TypeAnnotations};
pub use class::*;
pub use code::*;
pub use debug::*;
pub use field::*;
pub use method::*;
pub use module::*;

use crate::error::*;
use crate::mutf8::MStr;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type AttributeIter<'a> = DecodeCounted<'a, Attribute<'a>, u16>;

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

    pub fn read_content(&self, pool: &cpool::ConstantPool<'a>) -> Result<AttributeContent<'a>, DecodeError> {
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
            b"LocalVariableTypeTable" => Ok(AttributeContent::LocalVariableTypeTable(decoder.read_into()?)),
            b"MethodParameters" => Ok(AttributeContent::MethodParameters(decoder.read_into()?)),
            b"Module" => Ok(AttributeContent::Module(Box::new(decoder.read_into()?))),
            b"ModuleMainClass" => Ok(AttributeContent::ModuleMainClass(decoder.read_into()?)),
            b"ModulePackages" => Ok(AttributeContent::ModulePackages(decoder.read_into()?)),
            b"NestMembers" => Ok(AttributeContent::NestMembers(decoder.read_into()?)),
            b"NestHost" => Ok(AttributeContent::NestHost(decoder.read_into()?)),
            b"PermittedSubclasses" => Ok(AttributeContent::PermittedSubclasses(decoder.read_into()?)),
            b"Record" => Ok(AttributeContent::Record(decoder.read_into()?)),
            b"RuntimeInvisibleAnnotations" => Ok(AttributeContent::RuntimeInvisibleAnnotations(decoder.read_into()?)),
            b"RuntimeInvisibleParameterAnnotations" => Ok(AttributeContent::RuntimeInvisibleParameterAnnotations(
                decoder.read_into()?,
            )),
            b"RuntimeInvisibleTypeAnnotations" => {
                Ok(AttributeContent::RuntimeInvisibleTypeAnnotations(decoder.read_into()?))
            }
            b"RuntimeVisibleAnnotations" => Ok(AttributeContent::RuntimeVisibleAnnotations(decoder.read_into()?)),
            b"RuntimeVisibleParameterAnnotations" => Ok(AttributeContent::RuntimeVisibleParameterAnnotations(
                decoder.read_into()?,
            )),
            b"RuntimeVisibleTypeAnnotations" => {
                Ok(AttributeContent::RuntimeVisibleTypeAnnotations(decoder.read_into()?))
            }
            b"Signature" => Ok(AttributeContent::Signature(decoder.read_into()?)),
            b"SourceDebugExtension" => {
                let content = MStr::from_bytes(decoder.buf())?;
                Ok(AttributeContent::SourceDebugExtension(content))
            }
            b"SourceFile" => Ok(AttributeContent::SourceFile(decoder.read_into()?)),
            b"StackMapTable" => Ok(AttributeContent::StackMapTable(decoder.read_into()?)),
            b"Synthetic" => Ok(AttributeContent::Synthetic),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::UnknownAttributeName,
                &self.content,
            )),
        }
    }
}

#[derive(Clone)]
pub enum AttributeContent<'a> {
    AnnotationDefault(AnnotationDefault<'a>),
    BootstrapMethods(BootstrapMethods<'a>),
    Code(Code<'a>),
    ConstantValue(ConstantValue<'a>),
    Deprecated,
    EnclosingMethod(EnclosingMethod<'a>),
    Exceptions(Exceptions<'a>),
    InnerClasses(InnerClasses<'a>),
    LineNumberTable(LineNumberTable<'a>),
    LocalVariableTable(LocalVariableTable<'a>),
    LocalVariableTypeTable(LocalVariableTable<'a>),
    MethodParameters(MethodParameters<'a>),
    Module(Box<Module<'a>>),
    ModuleMainClass(ModuleMainClass<'a>),
    ModulePackages(ModulePackages<'a>),
    NestHost(NestHost<'a>),
    NestMembers(NestMembers<'a>),
    PermittedSubclasses(PermittedSubclasses<'a>),
    Record(Record<'a>),
    RuntimeInvisibleAnnotations(Annotations<'a>),
    RuntimeInvisibleParameterAnnotations(ParameterAnnotations<'a>),
    RuntimeInvisibleTypeAnnotations(TypeAnnotations<'a>),
    RuntimeVisibleAnnotations(Annotations<'a>),
    RuntimeVisibleParameterAnnotations(ParameterAnnotations<'a>),
    RuntimeVisibleTypeAnnotations(TypeAnnotations<'a>),
    Signature(Signature<'a>),
    SourceDebugExtension(&'a MStr),
    SourceFile(SourceFile<'a>),
    StackMapTable(StackMapTable<'a>),
    Synthetic,
}
