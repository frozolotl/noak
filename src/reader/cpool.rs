use crate::encoding::*;
use crate::error::*;
use crate::mutf8::MaybeMUtf8;

pub struct ConstantPool<'a> {
    content: Vec<Option<Item<'a>>>,
}

impl<'a> ConstantPool<'a> {
    pub fn get(&self, at: Index) -> Result<&Item<'a>, DecodeError> {
        let pos = at.0 as usize;
        if pos < self.content.len() {
            if let Some(item) = &self.content[pos] {
                return Ok(item);
            }
        }

        Err(DecodeError::with_context(
            DecodeErrorKind::InvalidIndex,
            Context::ConstantPool(at.0),
        ))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item<'a>> {
        self.content.iter().filter_map(|opt| opt.as_ref())
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = (Index, &Item<'a>)> {
        self.content
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|item| (Index(i as u16), item)))
    }
}

impl<'a> Decode<'a> for ConstantPool<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<ConstantPool<'a>, DecodeError> {
        let length: u16 = decoder.read()?;
        if length == 0 {
            return Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidLength,
                decoder,
            ));
        }
        let mut content = Vec::with_capacity(length as usize - 1);
        for _ in 0..length - 1 {
            let item = decoder.read()?;
            let push_extra = if let Item::Long(_) | Item::Double(_) = item {
                true
            } else {
                false
            };
            content.push(Some(item));
            if push_extra {
                content.push(None);
            }
        }

        Ok(ConstantPool { content })
    }
}

/// An index into the constant pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(u16);

impl<'a> Decode<'a> for Index {
    fn decode(decoder: &mut Decoder) -> Result<Index, DecodeError> {
        Ok(Index(decoder.read()?))
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a> {
    Class {
        name: Index,
    },
    FieldRef {
        class: Index,
        name_and_type: Index,
    },
    MethodRef {
        class: Index,
        name_and_type: Index,
    },
    InterfaceMethodRef {
        class: Index,
        name_and_type: Index,
    },
    String {
        string: Index,
    },
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    NameAndType {
        name: Index,
        descriptor: Index,
    },
    Utf8(MaybeMUtf8<'a>),
    MethodHandle {
        kind: MethodKind,
        reference: Index,
    },
    MethodType {
        descriptor: Index,
    },
    Dynamic {
        bootstrap_method_attr: Index,
        name_and_type: Index,
    },
    InvokeDynamic {
        // actually an index into the bootstrap method table
        bootstrap_method_attr: u16,
        name_and_type: Index,
    },
    Module {
        name: Index,
    },
    Package {
        name: Index,
    },
}

impl<'a> Decode<'a> for Item<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Item<'a>, DecodeError> {
        let tag: u8 = decoder.read()?;
        match tag {
            1 => {
                let len: u16 = decoder.read()?;
                let buf = decoder.split_bytes_off(len as usize)?;
                Ok(Item::Utf8(MaybeMUtf8::new(buf)))
            }
            3 => Ok(Item::Integer(decoder.read()?)),
            4 => Ok(Item::Float(decoder.read()?)),
            5 => Ok(Item::Long(decoder.read()?)),
            6 => Ok(Item::Double(decoder.read()?)),
            7 => Ok(Item::Class {
                name: decoder.read()?,
            }),
            8 => Ok(Item::String {
                string: decoder.read()?,
            }),
            9 => Ok(Item::FieldRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            }),
            10 => Ok(Item::MethodRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            }),
            11 => Ok(Item::InterfaceMethodRef {
                class: decoder.read()?,
                name_and_type: decoder.read()?,
            }),
            12 => Ok(Item::NameAndType {
                name: decoder.read()?,
                descriptor: decoder.read()?,
            }),
            15 => Ok(Item::MethodHandle {
                kind: decoder.read()?,
                reference: decoder.read()?,
            }),
            16 => Ok(Item::MethodType {
                descriptor: decoder.read()?,
            }),
            17 => Ok(Item::Dynamic {
                bootstrap_method_attr: decoder.read()?,
                name_and_type: decoder.read()?,
            }),
            18 => Ok(Item::InvokeDynamic {
                bootstrap_method_attr: decoder.read()?,
                name_and_type: decoder.read()?,
            }),
            19 => Ok(Item::Module {
                name: decoder.read()?,
            }),
            20 => Ok(Item::Package {
                name: decoder.read()?,
            }),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidTag,
                decoder,
            )),
        }
    }
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
    fn decode(decoder: &mut Decoder) -> Result<MethodKind, DecodeError> {
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
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidTag,
                decoder,
            )),
        }
    }
}
