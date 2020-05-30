use crate::error::*;
use crate::reader::cpool::{self, ConstantPool, Index};
use crate::mutf8::MStr;

pub trait ToValue<'a> {
    type Target;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError>;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Class<'a> {
    pub name: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::Class> {
    type Target = Class<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Class {
            name: pool.retrieve(this.name)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldRef<'a> {
    pub class: Class<'a>,
    pub name_and_type: NameAndType<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::FieldRef> {
    type Target = FieldRef<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(FieldRef {
            class: pool.retrieve(this.class)?,
            name_and_type: pool.retrieve(this.name_and_type)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MethodRef<'a> {
    pub class: Class<'a>,
    pub name_and_type: NameAndType<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::MethodRef> {
    type Target = MethodRef<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(MethodRef {
            class: pool.retrieve(this.class)?,
            name_and_type: pool.retrieve(this.name_and_type)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct InterfaceMethodRef<'a> {
    pub class: Class<'a>,
    pub name_and_type: NameAndType<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::InterfaceMethodRef> {
    type Target = InterfaceMethodRef<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(InterfaceMethodRef {
            class: pool.retrieve(this.class)?,
            name_and_type: pool.retrieve(this.name_and_type)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct String<'a> {
    pub string: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::String> {
    type Target = String<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(String {
            string: pool.retrieve(this.string)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Integer {
    pub value: i32,
}

impl<'a> ToValue<'a> for Index<cpool::Integer> {
    type Target = Integer;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Integer {
            value: this.value,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Long {
    pub value: i64,
}

impl<'a> ToValue<'a> for Index<cpool::Long> {
    type Target = Long;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Long {
            value: this.value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Float {
    pub value: f32,
}

impl<'a> ToValue<'a> for Index<cpool::Float> {
    type Target = Float;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Float {
            value: this.value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Double {
    pub value: f64,
}

impl<'a> ToValue<'a> for Index<cpool::Double> {
    type Target = Double;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Double {
            value: this.value,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NameAndType<'a> {
    pub name: &'a MStr,
    pub descriptor: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::NameAndType> {
    type Target = NameAndType<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(NameAndType {
            name: pool.retrieve(this.name)?,
            descriptor: pool.retrieve(this.descriptor)?,
        })
    }
}

impl<'a> ToValue<'a> for Index<cpool::Utf8<'a>> {
    type Target = &'a MStr;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(this.content)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodHandle<'a> {
    pub kind: cpool::MethodKind,
    pub reference: cpool::Item<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::MethodHandle> {
    type Target = MethodHandle<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(MethodHandle {
            kind: this.kind,
            reference: pool.get(this.reference)?.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodType<'a> {
    pub descriptor: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::MethodType> {
    type Target = MethodType<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(MethodType {
            descriptor: pool.retrieve(this.descriptor)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic<'a> {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: NameAndType<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::Dynamic> {
    type Target = Dynamic<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Dynamic {
            bootstrap_method_attr: this.bootstrap_method_attr,
            name_and_type: pool.retrieve(this.name_and_type)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvokeDynamic<'a> {
    // actually an index into the bootstrap method table
    pub bootstrap_method_attr: u16,
    pub name_and_type: NameAndType<'a>,
}

impl<'a> ToValue<'a> for Index<cpool::InvokeDynamic> {
    type Target = InvokeDynamic<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(InvokeDynamic {
            bootstrap_method_attr: this.bootstrap_method_attr,
            name_and_type: pool.retrieve(this.name_and_type)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'a> {
    pub name: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::Module> {
    type Target = Module<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Module {
            name: pool.retrieve(this.name)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package<'a> {
    pub name: &'a MStr,
}

impl<'a> ToValue<'a> for Index<cpool::Package> {
    type Target = Package<'a>;

    fn retrieve_from(self, pool: &ConstantPool<'a>) -> Result<Self::Target, DecodeError> {
        let this = pool.get(self)?;
        Ok(Package {
            name: pool.retrieve(this.name)?,
        })
    }
}
