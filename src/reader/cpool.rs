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
                return Ok(item)
            }
        }

        Err(DecodeError::new(DecodeErrorKind::InvalidIndex))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item<'a>> {
        self.content
            .iter()
            .filter_map(|opt| opt.as_ref())
    }
}

/// An index into the constant pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(u16);

impl Index {
    pub fn new(index: u16) -> Index {
        Index(index)
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a> {
    Class {
        name: Index,
    },
    FieldRef {
        class: Index,
        name_type: Index,
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
    Utf8 {
        content: MaybeMUtf8<'a>,
    },
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
