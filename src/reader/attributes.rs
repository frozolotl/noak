pub mod annotations;
mod class;
mod code;
mod debug;
mod field;
mod method;
mod module;

use std::fmt;

pub use annotations::{
    AnnotationDefault, RuntimeInvisibleAnnotations, RuntimeInvisibleParameterAnnotations,
    RuntimeInvisibleTypeAnnotations, RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
};
pub use class::*;
pub use code::*;
pub use debug::*;
pub use field::*;
pub use method::*;
pub use module::*;

use crate::error::*;
use crate::reader::cpool;
use crate::reader::decoding::*;
use crate::MStr;

#[derive(Clone)]
pub struct Attribute<'input> {
    name: cpool::Index<cpool::Utf8<'input>>,
    content: Decoder<'input>,
}

impl<'input> Decode<'input> for Attribute<'input> {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
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

impl<'input> Attribute<'input> {
    #[must_use]
    pub fn name(&self) -> cpool::Index<cpool::Utf8<'input>> {
        self.name
    }

    #[must_use]
    pub fn content(&self) -> &'input [u8] {
        self.content.buf()
    }

    pub fn read_content(&self, pool: &cpool::ConstantPool<'input>) -> Result<AttributeContent<'input>, DecodeError> {
        let name = pool.get(self.name)?.content;
        let decoder = self.content.with_context(Context::AttributeContent);
        match name.as_bytes() {
            b"AnnotationDefault" => Ok(AttributeContent::AnnotationDefault(decoder.read_into()?)),
            b"BootstrapMethods" => Ok(AttributeContent::BootstrapMethods(decoder.read_into()?)),
            b"Code" => Ok(AttributeContent::Code(decoder.read_into()?)),
            b"ConstantValue" => Ok(AttributeContent::ConstantValue(decoder.read_into()?)),
            b"Deprecated" => Ok(AttributeContent::Deprecated(decoder.read_into()?)),
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
            b"SourceDebugExtension" => Ok(AttributeContent::SourceDebugExtension(decoder.read_into()?)),
            b"SourceFile" => Ok(AttributeContent::SourceFile(decoder.read_into()?)),
            b"StackMapTable" => Ok(AttributeContent::StackMapTable(decoder.read_into()?)),
            b"Synthetic" => Ok(AttributeContent::Synthetic(decoder.read_into()?)),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::UnknownAttributeName,
                &self.content,
            )),
        }
    }
}

impl<'input> fmt::Debug for Attribute<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute").finish()
    }
}

#[derive(Debug, Clone)]
pub enum AttributeContent<'input> {
    AnnotationDefault(AnnotationDefault<'input>),
    BootstrapMethods(BootstrapMethods<'input>),
    Code(Code<'input>),
    ConstantValue(ConstantValue<'input>),
    Deprecated(Deprecated<'input>),
    EnclosingMethod(EnclosingMethod<'input>),
    Exceptions(Exceptions<'input>),
    InnerClasses(InnerClasses<'input>),
    LineNumberTable(LineNumberTable<'input>),
    LocalVariableTable(LocalVariableTable<'input>),
    LocalVariableTypeTable(LocalVariableTypeTable<'input>),
    MethodParameters(MethodParameters<'input>),
    Module(Box<Module<'input>>),
    ModuleMainClass(ModuleMainClass<'input>),
    ModulePackages(ModulePackages<'input>),
    NestHost(NestHost<'input>),
    NestMembers(NestMembers<'input>),
    PermittedSubclasses(PermittedSubclasses<'input>),
    Record(Record<'input>),
    RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotations<'input>),
    RuntimeInvisibleParameterAnnotations(RuntimeInvisibleParameterAnnotations<'input>),
    RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotations<'input>),
    RuntimeVisibleAnnotations(RuntimeVisibleAnnotations<'input>),
    RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotations<'input>),
    RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotations<'input>),
    Signature(Signature<'input>),
    SourceDebugExtension(SourceDebugExtension<'input>),
    SourceFile(SourceFile<'input>),
    StackMapTable(StackMapTable<'input>),
    Synthetic(Synthetic<'input>),
}

macro_rules! impl_try_from {
    ($($attr:ident),* $(,)?) => {
        $(
            impl<'input> TryFrom<AttributeContent<'input>> for $attr<'input> {
                type Error = DecodeError;

                fn try_from(value: AttributeContent<'input>) -> Result<Self, Self::Error> {
                    match value {
                        AttributeContent::$attr(attr) => Ok(attr),
                        _ => Err(DecodeError::with_context(DecodeErrorKind::AttributeNotFound, Context::Attributes)),
                    }
                }
            }
        )*
    };
}

impl_try_from! {
    AnnotationDefault,
    BootstrapMethods,
    Code,
    ConstantValue,
    Deprecated,
    EnclosingMethod,
    Exceptions,
    InnerClasses,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    MethodParameters,
    ModuleMainClass,
    ModulePackages,
    NestHost,
    NestMembers,
    PermittedSubclasses,
    Record,
    RuntimeInvisibleAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeInvisibleTypeAnnotations,
    RuntimeVisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    Signature,
    SourceDebugExtension,
    SourceFile,
    StackMapTable,
    Synthetic,
}

impl<'input> TryFrom<AttributeContent<'input>> for Module<'input> {
    type Error = DecodeError;

    fn try_from(value: AttributeContent<'input>) -> Result<Self, Self::Error> {
        match value {
            AttributeContent::Module(module) => Ok(*module),
            _ => Err(DecodeError::with_context(
                DecodeErrorKind::AttributeNotFound,
                Context::Attributes,
            )),
        }
    }
}

impl<'input> TryFrom<AttributeContent<'input>> for Box<Module<'input>> {
    type Error = DecodeError;

    fn try_from(value: AttributeContent<'input>) -> Result<Self, Self::Error> {
        match value {
            AttributeContent::Module(module) => Ok(module),
            _ => Err(DecodeError::with_context(
                DecodeErrorKind::AttributeNotFound,
                Context::Attributes,
            )),
        }
    }
}

impl<'input> DecodeMany<'input, Attribute<'input>, u16> {
    // FIXME: don't decode every data structure
    pub fn find_attribute<A>(&self, pool: &cpool::ConstantPool<'input>) -> Option<A>
    where
        A: TryFrom<AttributeContent<'input>>,
    {
        self.iter()
            .find_map(|attribute| attribute.ok()?.read_content(pool).ok()?.try_into().ok())
    }
}
