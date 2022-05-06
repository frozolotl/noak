pub mod value;

pub use value::ToValue;

use crate::error::*;
use crate::mutf8::MStr;
use crate::reader::decoding::*;
use std::{fmt, marker::PhantomData, num::NonZeroU16};

#[derive(Clone)]
pub struct ConstantPool<'a> {
    content: Vec<Option<Item<'a>>>,
}

impl<'a> ConstantPool<'a> {
    pub fn retrieve<I>(&self, at: Index<I>) -> Result<<Index<I> as ToValue<'a>>::Target, DecodeError>
    where
        Index<I>: ToValue<'a>,
    {
        at.retrieve_from(self)
    }

    pub fn get<I: TryFromItem<'a>>(&self, at: Index<I>) -> Result<&I, DecodeError> {
        let pos = at.index.get() as usize;
        if pos != 0 && pos <= self.content.len() {
            if let Some(item) = &self.content[pos - 1] {
                return I::try_from_item(item)
                    .ok_or_else(|| DecodeError::with_context(DecodeErrorKind::TagMismatch, Context::ConstantPool));
            }
        }

        Err(DecodeError::with_context(
            DecodeErrorKind::InvalidIndex,
            Context::ConstantPool,
        ))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item<'a>> {
        self.content.iter().filter_map(|opt| opt.as_ref())
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = (Index<Item<'a>>, &Item<'a>)> {
        self.content
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|item| (Index::new(i as u16 + 1).unwrap(), item)))
    }
}

impl<'a> Decode<'a> for ConstantPool<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<ConstantPool<'a>, DecodeError> {
        decoder.set_context(Context::ConstantPool);
        let length = decoder.read::<u16>()?;
        if length == 0 {
            return Err(DecodeError::from_decoder(DecodeErrorKind::InvalidLength, decoder));
        }
        let length = length as usize - 1;
        let mut content = Vec::with_capacity(length);
        while content.len() < length {
            let item = decoder.read()?;
            let push_extra = matches!(item, Item::Long(_) | Item::Double(_));

            content.push(Some(item));
            if push_extra {
                content.push(None);
            }
        }

        Ok(ConstantPool { content })
    }
}

impl<'a> fmt::Debug for ConstantPool<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConstantPool").finish()
    }
}

/// A 1-based index into the constant pool.
#[derive(PartialEq)]
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

impl<I> Index<I> {
    pub fn new(index: u16) -> Result<Index<I>, DecodeError> {
        if let Some(v) = NonZeroU16::new(index) {
            Ok(Index {
                index: v,
                mark: PhantomData,
            })
        } else {
            Err(DecodeError::new(DecodeErrorKind::InvalidIndex))
        }
    }

    pub fn as_u16(self) -> u16 {
        self.index.get()
    }
}

impl<I> fmt::Debug for Index<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("cpool::Index").field(&self.index).finish()
    }
}

impl<'a, I: 'a> Decode<'a> for Index<I> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let index = Index::new(decoder.read()?).map_err(|err| DecodeError::from_decoder(err.kind(), decoder))?;
        Ok(index)
    }
}

impl<'a, I: 'a> Decode<'a> for Option<Index<I>> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Index::new(decoder.read()?).ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item<'a> {
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
    Utf8(Utf8<'a>),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    Dynamic(Dynamic),
    InvokeDynamic(InvokeDynamic),
    Module(Module),
    Package(Package),
}

impl<'a> Decode<'a> for Item<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Item<'a>, DecodeError> {
        let tag: u8 = decoder.read()?;
        match tag {
            1 => {
                let len: u16 = decoder.read()?;
                let buf = decoder.split_bytes_off(len as usize)?;
                Ok(Item::Utf8(Utf8 {
                    content: MStr::from_bytes(buf)?,
                }))
            }
            3 => Ok(Item::Integer(Integer { value: decoder.read()? })),
            4 => Ok(Item::Float(Float { value: decoder.read()? })),
            5 => Ok(Item::Long(Long { value: decoder.read()? })),
            6 => Ok(Item::Double(Double { value: decoder.read()? })),
            7 => Ok(Item::Class(Class { name: decoder.read()? })),
            8 => Ok(Item::String(String {
                string: decoder.read()?,
            })),
            9 => Ok(Item::FieldRef(FieldRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            })),
            10 => Ok(Item::MethodRef(MethodRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            })),
            11 => Ok(Item::InterfaceMethodRef(InterfaceMethodRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            })),
            12 => Ok(Item::NameAndType(NameAndType {
                name: decoder.read()?,
                descriptor: decoder.read()?,
            })),
            15 => Ok(Item::MethodHandle(MethodHandle {
                kind: decoder.read()?,
                reference: decoder.read()?,
            })),
            16 => Ok(Item::MethodType(MethodType {
                descriptor: decoder.read()?,
            })),
            17 => Ok(Item::Dynamic(Dynamic {
                bootstrap_method_attr: decoder.read()?,
                name_and_type: decoder.read()?,
            })),
            18 => Ok(Item::InvokeDynamic(InvokeDynamic {
                bootstrap_method_attr: decoder.read()?,
                name_and_type: decoder.read()?,
            })),
            19 => Ok(Item::Module(Module { name: decoder.read()? })),
            20 => Ok(Item::Package(Package { name: decoder.read()? })),
            _ => Err(DecodeError::from_decoder(DecodeErrorKind::InvalidTag, decoder)),
        }
    }
}

pub trait TryFromItem<'a>: Sized {
    fn try_from_item<'b>(item: &'b Item<'a>) -> Option<&'b Self>;
}

macro_rules! impl_try_from_item {
    ($($name:ident;)*) => {
        $(
            impl<'a> TryFromItem<'a> for $name {
                fn try_from_item<'b>(item: &'b Item<'a>) -> Option<&'b Self> {
                    if let Item::$name(v) = item {
                        Some(v)
                    } else {
                        None
                    }
                }
            }
        )*
    }
}

impl_try_from_item! {
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
    MethodHandle;
    MethodType;
    Dynamic;
    InvokeDynamic;
    Module;
    Package;
}

impl<'a> TryFromItem<'a> for Utf8<'a> {
    fn try_from_item<'b>(item: &'b Item<'a>) -> Option<&'b Self> {
        if let Item::Utf8(v) = item {
            Some(v)
        } else {
            None
        }
    }
}

impl<'a> TryFromItem<'a> for Item<'a> {
    fn try_from_item<'b>(item: &'b Item<'a>) -> Option<&'b Self> {
        Some(item)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethodRef {
    pub class: Index<Class>,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct String {
    pub string: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Long {
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Double {
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAndType {
    pub name: Index<Utf8<'static>>,
    pub descriptor: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Utf8<'a> {
    pub content: &'a MStr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodHandle {
    pub kind: MethodKind,
    pub reference: Index<Item<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodType {
    pub descriptor: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvokeDynamic {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: Index<NameAndType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: Index<Utf8<'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<'a> Decode<'a> for MethodKind {
    fn decode(decoder: &mut Decoder<'a>) -> Result<MethodKind, DecodeError> {
        let tag: u8 = decoder.read()?;
        use MethodKind::*;

        match tag {
            1 => Ok(GetField),
            2 => Ok(GetStatic),
            3 => Ok(PutField),
            4 => Ok(PutStatic),
            5 => Ok(InvokeVirtual),
            6 => Ok(InvokeStatic),
            7 => Ok(InvokeSpecial),
            8 => Ok(NewInvokeSpecial),
            9 => Ok(InvokeInterface),
            _ => Err(DecodeError::from_decoder(DecodeErrorKind::InvalidTag, decoder)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            // length
            0x00, 0x01
        ], Context::ConstantPool);
        let pool: ConstantPool<'_> = decoder.read().unwrap();
        assert_eq!(pool.iter().count(), 0);
    }

    #[test]
    fn negative_length() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            // length
            0x00, 0x00
        ], Context::ConstantPool);
        assert!(decoder.read::<ConstantPool<'_>>().is_err());
    }

    #[test]
    fn iteration_and_decoding() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            // length
            0x00, 0x06,
            // integer
            0x03,
            0x00, 0x00, 0x00, 0x05,
            // utf8
            0x01,
            0x00, 0x0B,
            b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
            // long (takes up two spaces)
            0x05,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
            // string
            0x08,
            0x00, 0x02,
            // random bytes which should not be read
            0xAB, 0xC4, 0x12, 0x4B, 0xFF, 0x00,
        ], Context::ConstantPool);
        let pool: ConstantPool<'_> = decoder.read().unwrap();
        let mut iter = pool.iter();
        assert_eq!(iter.next(), Some(&Item::Integer(Integer { value: 5 })));
        assert_eq!(
            iter.next(),
            Some(&Item::Utf8(Utf8 {
                content: MStr::from_bytes(b"hello world").unwrap(),
            }))
        );
        assert_eq!(iter.next(), Some(&Item::Long(Long { value: 0xFF })));
        assert_eq!(
            iter.next(),
            Some(&Item::String(String {
                string: Index::new(2).unwrap(),
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn get() {
        // just to test that it will work with non-'static strings
        let some_string = b"hello world".to_vec();
        let content = vec![
            Some(Item::Integer(Integer { value: 2 })),
            Some(Item::Long(Long { value: 3 })),
            None,
            Some(Item::Integer(Integer { value: 4 })),
            Some(Item::String(String {
                string: Index::new(6).unwrap(),
            })),
            Some(Item::Utf8(Utf8 {
                content: MStr::from_bytes(&some_string).unwrap(),
            })),
        ];

        let pool = ConstantPool { content };
        assert_eq!(pool.get(Index::new(1).unwrap()), Ok(&Integer { value: 2 }));
        assert_eq!(pool.get(Index::new(2).unwrap()), Ok(&Long { value: 3 }));
        assert_eq!(pool.get(Index::new(4).unwrap()), Ok(&Integer { value: 4 }));
        let string: &String = pool.get(Index::new(5).unwrap()).unwrap();
        assert_eq!(
            pool.get(string.string),
            Ok(&Utf8 {
                content: MStr::from_bytes(&some_string).unwrap(),
            })
        );

        assert!(pool.get::<Double>(Index::new(4).unwrap()).is_err());
        assert!(pool.get::<Item<'_>>(Index::new(3).unwrap()).is_err());
        assert!(pool.get::<Item<'_>>(Index::new(7).unwrap()).is_err());
        assert!(Index::<Item<'_>>::new(0).is_err());
    }
}
