use crate::error::*;
use crate::mutf8::MString;
use crate::writer::encoding::*;
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

impl<I> Index<I> {
    pub fn as_item(self) -> Index<Item> {
        Index {
            index: self.index,
            mark: PhantomData,
        }
    }

    pub fn as_u16(self) -> u16 {
        self.index.get()
    }
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
        encoder.write(self.index.get())?;
        Ok(())
    }
}

impl<I> Encode for Option<Index<I>> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        if let Some(index) = self {
            encoder.write(index.index.get())?;
        } else {
            encoder.write(0u16)?;
        }
        Ok(())
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

impl Class {
    pub fn by<I>(name: I) -> ClassInserter<I>
    where
        I: Insertable<Class>,
    {
        ClassInserter { name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FieldRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

impl FieldRef {
    pub fn by<Class, Nat>(class: Class, name_and_type: Nat) -> FieldRefInserter<Class, Nat>
    where
        Class: Insertable<self::Class>,
        Nat: Insertable<NameAndType>,
    {
        FieldRefInserter {
            class,
            name_and_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

impl MethodRef {
    pub fn by<Class, Nat>(class: Class, name_and_type: Nat) -> MethodRefInserter<Class, Nat>
    where
        Class: Insertable<self::Class>,
        Nat: Insertable<NameAndType>,
    {
        MethodRefInserter {
            class,
            name_and_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InterfaceMethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

impl InterfaceMethodRef {
    pub fn by<Class, Nat>(
        class: Class,
        name_and_type: Nat,
    ) -> InterfaceMethodRefInserter<Class, Nat>
    where
        Class: Insertable<self::Class>,
        Nat: Insertable<NameAndType>,
    {
        InterfaceMethodRefInserter {
            class,
            name_and_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct String {
    pub string: Index<Utf8>,
}

impl String {
    pub fn by<I>(content: I) -> StringInserter<I>
    where
        I: Insertable<String>,
    {
        StringInserter { string: content }
    }
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

impl Utf8 {
    pub fn by<I>(content: I) -> Utf8Inserter<I>
    where
        I: Insertable<Utf8>,
    {
        Utf8Inserter { content }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodHandle {
    pub kind: MethodKind,
    pub reference: Index<Item>,
}

impl MethodHandle {
    pub fn by<I>(kind: MethodKind, reference: I) -> MethodHandleInserter<I>
    where
        I: Insertable<Item>,
    {
        MethodHandleInserter { kind, reference }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodType {
    pub descriptor: Index<Utf8>,
}

impl MethodType {
    pub fn by<I>(descriptor: I) -> MethodTypeInserter<I>
    where
        I: Insertable<Utf8>,
    {
        MethodTypeInserter { descriptor }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Dynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

impl Dynamic {
    pub fn by<I>(bootstrap_method_attr: u16, name_and_type: I) -> DynamicInserter<I>
    where
        I: Insertable<NameAndType>,
    {
        DynamicInserter {
            bootstrap_method_attr,
            name_and_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InvokeDynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

impl InvokeDynamic {
    pub fn by<I>(bootstrap_method_attr: u16, name_and_type: I) -> InvokeDynamicInserter<I>
    where
        I: Insertable<NameAndType>,
    {
        InvokeDynamicInserter {
            bootstrap_method_attr,
            name_and_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Module {
    pub name: Index<Utf8>,
}

impl Module {
    pub fn by<I>(name: I) -> ModuleInserter<I>
    where
        I: Insertable<Utf8>,
    {
        ModuleInserter { name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Package {
    pub name: Index<Utf8>,
}

impl Package {
    pub fn by<I>(name: I) -> PackageInserter<I>
    where
        I: Insertable<Utf8>,
    {
        PackageInserter { name }
    }
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

        encoder.write(tag)?;
        Ok(())
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
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<O>, EncodeError>;
}

macro_rules! impl_insertable {
    ($($name:ident;)*) => {
        $(
            impl Insertable<$name> for $name {
                fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<$name>, EncodeError> {
                    context.class_writer_mut().insert_constant(self)
                }
            }

            impl Insertable<$name> for Index<$name> {
                fn insert<Ctx: EncoderContext>(self, _: &mut Ctx) -> Result<Index<$name>, EncodeError> {
                    Ok(self)
                }
            }
        )*
    }
}

impl_insertable! {
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

impl Insertable<Item> for Index<Item> {
    fn insert<Ctx: EncoderContext>(self, _: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(self)
    }
}

impl<I: Into<MString>> Insertable<Utf8> for I {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Utf8>, EncodeError> {
        context.class_writer_mut().insert_constant(Utf8 {
            content: self.into(),
        })
    }
}

impl<I: Insertable<Utf8>> Insertable<String> for I {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<String>, EncodeError> {
        let string = self.insert(context)?;
        context
            .class_writer_mut()
            .insert_constant(String { string })
    }
}

impl<I: Insertable<Utf8>> Insertable<Class> for I {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Class>, EncodeError> {
        let name = self.insert(context)?;
        context.class_writer_mut().insert_constant(Class { name })
    }
}

impl Insertable<Integer> for i32 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Integer>, EncodeError> {
        context
            .class_writer_mut()
            .insert_constant(Integer { value: self })
    }
}

impl Insertable<Item> for i32 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<i32 as Insertable<Integer>>::insert(self, context)?.as_item())
    }
}

impl Insertable<Long> for i64 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Long>, EncodeError> {
        context
            .class_writer_mut()
            .insert_constant(Long { value: self })
    }
}

impl Insertable<Item> for i64 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<i64 as Insertable<Long>>::insert(self, context)?.as_item())
    }
}

impl Insertable<Float> for f32 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Float>, EncodeError> {
        context
            .class_writer_mut()
            .insert_constant(Float { value: self })
    }
}

impl Insertable<Item> for f32 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<f32 as Insertable<Float>>::insert(self, context)?.as_item())
    }
}

impl Insertable<Double> for f64 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Double>, EncodeError> {
        context
            .class_writer_mut()
            .insert_constant(Double { value: self })
    }
}

impl Insertable<Item> for f64 {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<f64 as Insertable<Double>>::insert(self, context)?.as_item())
    }
}

pub struct Utf8Inserter<I> {
    content: I,
}

impl<I: Insertable<Utf8>> Insertable<Utf8> for Utf8Inserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Utf8>, EncodeError> {
        self.content.insert(context)
    }
}

impl<I: Insertable<Utf8>> Insertable<Item> for Utf8Inserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(self.content.insert(context)?.as_item())
    }
}

pub struct MethodHandleInserter<I> {
    kind: MethodKind,
    reference: I,
}

impl<I: Insertable<Item>> Insertable<MethodHandle> for MethodHandleInserter<I> {
    fn insert<Ctx: EncoderContext>(
        self,
        context: &mut Ctx,
    ) -> Result<Index<MethodHandle>, EncodeError> {
        let reference = self.reference.insert(context)?;
        context.class_writer_mut().insert_constant(MethodHandle {
            kind: self.kind,
            reference,
        })
    }
}

impl<I: Insertable<Item>> Insertable<Item> for MethodHandleInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<MethodHandle>>::insert(self, context)?.as_item())
    }
}

pub struct MethodTypeInserter<I> {
    descriptor: I,
}

impl<I: Insertable<Utf8>> Insertable<MethodType> for MethodTypeInserter<I> {
    fn insert<Ctx: EncoderContext>(
        self,
        context: &mut Ctx,
    ) -> Result<Index<MethodType>, EncodeError> {
        let descriptor = self.descriptor.insert(context)?;
        context
            .class_writer_mut()
            .insert_constant(MethodType { descriptor })
    }
}

impl<I: Insertable<Utf8>> Insertable<Item> for MethodTypeInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<MethodType>>::insert(self, context)?.as_item())
    }
}

pub struct DynamicInserter<I> {
    bootstrap_method_attr: u16,
    name_and_type: I,
}

impl<I: Insertable<NameAndType>> Insertable<Dynamic> for DynamicInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Dynamic>, EncodeError> {
        let name_and_type = self.name_and_type.insert(context)?;
        context.class_writer_mut().insert_constant(Dynamic {
            bootstrap_method_attr: self.bootstrap_method_attr,
            name_and_type,
        })
    }
}

impl<I: Insertable<NameAndType>> Insertable<Item> for DynamicInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<Dynamic>>::insert(self, context)?.as_item())
    }
}

pub struct InvokeDynamicInserter<I> {
    bootstrap_method_attr: u16,
    name_and_type: I,
}

impl<I: Insertable<NameAndType>> Insertable<InvokeDynamic> for InvokeDynamicInserter<I> {
    fn insert<Ctx: EncoderContext>(
        self,
        context: &mut Ctx,
    ) -> Result<Index<InvokeDynamic>, EncodeError> {
        let name_and_type = self.name_and_type.insert(context)?;
        context.class_writer_mut().insert_constant(InvokeDynamic {
            bootstrap_method_attr: self.bootstrap_method_attr,
            name_and_type,
        })
    }
}

impl<I: Insertable<NameAndType>> Insertable<Item> for InvokeDynamicInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<InvokeDynamic>>::insert(self, context)?.as_item())
    }
}

pub struct ModuleInserter<I> {
    name: I,
}

impl<I: Insertable<Utf8>> Insertable<Module> for ModuleInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Module>, EncodeError> {
        let name = self.name.insert(context)?;
        context.class_writer_mut().insert_constant(Module { name })
    }
}

impl<I: Insertable<Utf8>> Insertable<Item> for ModuleInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<Module>>::insert(self, context)?.as_item())
    }
}

pub struct PackageInserter<I> {
    name: I,
}

impl<I: Insertable<Utf8>> Insertable<Package> for PackageInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Package>, EncodeError> {
        let name = self.name.insert(context)?;
        context.class_writer_mut().insert_constant(Package { name })
    }
}

impl<I: Insertable<Utf8>> Insertable<Item> for PackageInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<Package>>::insert(self, context)?.as_item())
    }
}

pub struct StringInserter<I> {
    string: I,
}

impl<I: Insertable<String>> Insertable<String> for StringInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<String>, EncodeError> {
        self.string.insert(context)
    }
}

impl<I: Insertable<String>> Insertable<Item> for StringInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(self.string.insert(context)?.as_item())
    }
}

pub struct ClassInserter<I> {
    name: I,
}

impl<I: Insertable<Class>> Insertable<Class> for ClassInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Class>, EncodeError> {
        self.name.insert(context)
    }
}

impl<I: Insertable<Class>> Insertable<Item> for ClassInserter<I> {
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(self.name.insert(context)?.as_item())
    }
}

impl<Name, Descriptor> Insertable<NameAndType> for (Name, Descriptor)
where
    Name: Insertable<Utf8>,
    Descriptor: Insertable<Utf8>,
{
    fn insert<Ctx: EncoderContext>(
        self,
        context: &mut Ctx,
    ) -> Result<Index<NameAndType>, EncodeError> {
        let name = self.0.insert(context)?;
        let descriptor = self.1.insert(context)?;
        context
            .class_writer_mut()
            .insert_constant(NameAndType { name, descriptor })
    }
}

impl<Name, Descriptor> Insertable<Item> for (Name, Descriptor)
where
    Name: Insertable<Utf8>,
    Descriptor: Insertable<Utf8>,
{
    fn insert<Ctx: EncoderContext>(self, context: &mut Ctx) -> Result<Index<Item>, EncodeError> {
        Ok(<Self as Insertable<NameAndType>>::insert(self, context)?.as_item())
    }
}

macro_rules! ref_inserter {
    ($name:ident, $out:ident) => {
        pub struct $name<Class, Nat> {
            class: Class,
            name_and_type: Nat,
        }

        impl<Class, Nat> Insertable<$out> for $name<Class, Nat>
        where
            Class: Insertable<self::Class>,
            Nat: Insertable<NameAndType>,
        {
            fn insert<Ctx: EncoderContext>(
                self,
                context: &mut Ctx,
            ) -> Result<Index<$out>, EncodeError> {
                let class = self.class.insert(context)?;
                let name_and_type = self.name_and_type.insert(context)?;
                context.class_writer_mut().insert_constant($out {
                    class,
                    name_and_type,
                })
            }
        }

        impl<Class, Nat> Insertable<Item> for $name<Class, Nat>
        where
            Class: Insertable<self::Class>,
            Nat: Insertable<NameAndType>,
        {
            fn insert<Ctx: EncoderContext>(
                self,
                context: &mut Ctx,
            ) -> Result<Index<Item>, EncodeError> {
                Ok(<Self as Insertable<$out>>::insert(self, context)?.as_item())
            }
        }
    };
}

ref_inserter!(FieldRefInserter, FieldRef);
ref_inserter!(MethodRefInserter, MethodRef);
ref_inserter!(InterfaceMethodRefInserter, InterfaceMethodRef);
