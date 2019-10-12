use crate::error::*;
use crate::mutf8::MString;
use crate::writer::{encoding::*, ClassWriter};
use indexmap::IndexMap;
use std::{
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU16,
};

#[derive(Clone)]
pub(crate) struct ConstantPool {
    content: IndexMap<Item, Index<Item>>,
    len: u16,
}

impl ConstantPool {
    pub(crate) fn new() -> ConstantPool {
        ConstantPool {
            content: IndexMap::new(),
            len: 1,
        }
    }

    pub(crate) fn insert<I: Into<Item>, E: Encoder>(
        &mut self,
        item: I,
        mut encoder: E,
    ) -> Result<Index<I>, EncodeError> {
        if self.len == u16::max_value() {
            return Err(EncodeError::with_context(
                EncodeErrorKind::TooManyItems,
                Context::ConstantPool,
            ));
        }
        let item = item.into();
        let index = NonZeroU16::new(self.len).unwrap();
        self.len += if let Item::Long(_) | Item::Double(_) = item {
            2
        } else {
            1
        };

        encoder.write(&item)?;
        self.content.insert(
            item,
            Index {
                index,
                mark: PhantomData,
            },
        );

        // Another index has to be created as Index<I> and Index<Item> are different types
        Ok(Index {
            index,
            mark: PhantomData,
        })
    }

    pub(crate) fn len(&self) -> u16 {
        self.len
    }
}

impl fmt::Debug for ConstantPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConstantPool").finish()
    }
}

/// A 1-based index into the constant pool.
#[derive(Eq, PartialEq, Hash)]
pub struct Index<I> {
    index: NonZeroU16,
    mark: PhantomData<I>,
}

impl<I> Clone for Index<I> {
    fn clone(&self) -> Index<I> {
        Index {
            index: self.index,
            mark: PhantomData,
        }
    }
}

impl<I> Copy for Index<I> {}

impl<I> fmt::Debug for Index<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cpool::Index({})", self.index)
    }
}

impl<I> Encode for Index<I> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write(self.index.get())
    }
}

impl<I> Encode for Option<Index<I>> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        if let Some(index) = self {
            encoder.write(index.index.get())
        } else {
            encoder.write(0u16)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Item {
    Class(Class),
    FieldRef(FieldRef),
    MethodRef(MethodRef),
    InterfaceMethodRef(InterfaceMethodRef),
    String(String),
    Integer(Integer),
    Long(Long),
    Float(Float),
    Double(Double),
    NameAndType(NameAndType),
    Utf8(Utf8),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    Dynamic(Dynamic),
    InvokeDynamic(InvokeDynamic),
    Module(Module),
    Package(Package),
}

impl Encode for Item {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Item::Utf8(v) => {
                encoder.write(1u8)?;
                let length = u16::try_from(v.content.len()).map_err(|_| {
                    EncodeError::with_context(EncodeErrorKind::StringTooLong, Context::ConstantPool)
                })?;
                encoder.write(length)?;
                encoder.write(v.content.as_bytes())?;
            }
            Item::Integer(v) => {
                encoder.write(3u8)?;
                encoder.write(v.value)?;
            }
            Item::Float(v) => {
                encoder.write(4u8)?;
                encoder.write(v.value)?;
            }
            Item::Long(v) => {
                encoder.write(5u8)?;
                encoder.write(v.value)?;
            }
            Item::Double(v) => {
                encoder.write(6u8)?;
                encoder.write(v.value)?;
            }
            Item::Class(v) => {
                encoder.write(7u8)?;
                encoder.write(v.name)?;
            }
            Item::String(v) => {
                encoder.write(8u8)?;
                encoder.write(v.string)?;
            }
            Item::FieldRef(v) => {
                encoder.write(9u8)?;
                encoder.write(v.class)?;
                encoder.write(v.name_and_type)?;
            }
            Item::MethodRef(v) => {
                encoder.write(10u8)?;
                encoder.write(v.class)?;
                encoder.write(v.name_and_type)?;
            }
            Item::InterfaceMethodRef(v) => {
                encoder.write(11u8)?;
                encoder.write(v.class)?;
                encoder.write(v.name_and_type)?;
            }
            Item::NameAndType(v) => {
                encoder.write(12u8)?;
                encoder.write(v.name)?;
                encoder.write(v.descriptor)?;
            }
            Item::MethodHandle(v) => {
                encoder.write(15u8)?;
                encoder.write(v.kind)?;
                encoder.write(v.reference)?;
            }
            Item::MethodType(v) => {
                encoder.write(16u8)?;
                encoder.write(v.descriptor)?;
            }
            Item::Dynamic(v) => {
                encoder.write(17u8)?;
                encoder.write(v.bootstrap_method_attr)?;
                encoder.write(v.name_and_type)?;
            }
            Item::InvokeDynamic(v) => {
                encoder.write(18u8)?;
                encoder.write(v.bootstrap_method_attr)?;
                encoder.write(v.name_and_type)?;
            }
            Item::Module(v) => {
                encoder.write(19u8)?;
                encoder.write(v.name)?;
            }
            Item::Package(v) => {
                encoder.write(20u8)?;
                encoder.write(v.name)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Class {
    pub name: Index<Utf8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FieldRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InterfaceMethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct String {
    pub string: Index<Utf8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Integer {
    pub value: i32,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Long {
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Float {
    pub value: f32,
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}

impl Eq for Float {}

impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Double {
    pub value: f64,
}

impl PartialEq for Double {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}

impl Eq for Double {}

impl Hash for Double {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NameAndType {
    pub name: Index<Utf8>,
    pub descriptor: Index<Utf8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Utf8 {
    pub content: MString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodHandle {
    pub kind: MethodKind,
    pub reference: Index<Item>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodType {
    pub descriptor: Index<Utf8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Dynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InvokeDynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Module {
    pub name: Index<Utf8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Package {
    pub name: Index<Utf8>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MethodKind {
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface,
}

impl Encode for MethodKind {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        use MethodKind::*;
        let tag: u8 = match self {
            GetField => 1,
            GetStatic => 2,
            PutField => 3,
            PutStatic => 4,
            InvokeVirtual => 5,
            InvokeStatic => 6,
            InvokeSpecial => 7,
            NewInvokeSpecial => 8,
            InvokeInterface => 9,
        };

        encoder.write(tag)
    }
}

macro_rules! impl_into_item {
    ($($name:ident;)*) => {
        $(
            impl From<$name> for Item {
                fn from(item: $name) -> Item {
                    Item::$name(item)
                }
            }
        )*
    }
}

impl_into_item! {
    Class;
    FieldRef;
    MethodRef;
    InterfaceMethodRef;
    String;
    Integer;
    Long;
    Float;
    Double;
    NameAndType;
    Utf8;
    MethodHandle;
    MethodType;
    Dynamic;
    InvokeDynamic;
    Module;
    Package;
}

pub trait Insertable<O> {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<O>, EncodeError>;
}

macro_rules! impl_insertable {
    ($($name:ident;)*) => {
        $(
            impl Insertable<$name> for $name {
                fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<$name>, EncodeError> {
                    class_writer.insert_constant(self)
                }
            }
        )*
    }
}

impl_insertable! {
    Item;
    Class;
    FieldRef;
    MethodRef;
    InterfaceMethodRef;
    String;
    Integer;
    Long;
    Float;
    Double;
    NameAndType;
    Utf8;
    MethodHandle;
    MethodType;
    Dynamic;
    InvokeDynamic;
    Module;
    Package;
}

impl<I: Into<MString>> Insertable<Utf8> for I {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Utf8>, EncodeError> {
        class_writer.insert_constant(Utf8 {
            content: self.into(),
        })
    }
}

impl<I: Insertable<Utf8>> Insertable<Class> for I {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Class>, EncodeError> {
        let name = self.insert(class_writer)?;
        class_writer.insert_constant(Class { name })
    }
}

impl Insertable<Integer> for i32 {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Integer>, EncodeError> {
        class_writer.insert_constant(Integer { value: self })
    }
}

impl Insertable<Long> for i64 {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Long>, EncodeError> {
        class_writer.insert_constant(Long { value: self })
    }
}

impl Insertable<Float> for f32 {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Float>, EncodeError> {
        class_writer.insert_constant(Float { value: self })
    }
}

impl Insertable<Double> for f64 {
    fn insert(self, class_writer: &mut ClassWriter) -> Result<Index<Double>, EncodeError> {
        class_writer.insert_constant(Double { value: self })
    }
}
