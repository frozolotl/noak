use crate::reader::decoding::*;
use crate::{header::AccessFlags, reader::cpool};

crate::__dec_structure! {
    pub struct EnclosingMethod<'a> into {
        class: cpool::Index<cpool::Class>,
        method: cpool::Index<cpool::NameAndType>,
    }
}

crate::__dec_structure! {
    pub struct NestHost<'a> into {
        host_class: cpool::Index<cpool::Class>,
    }
}

pub type NestMembers<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Class>, u16>;
pub type NestMemberIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

pub type InnerClasses<'a> = DecodeCountedCopy<'a, InnerClass<'a>, u16>;
pub type InnerClassIter<'a> = DecodeCounted<'a, InnerClass<'a>, u16>;

crate::__dec_structure! {
    pub struct InnerClass<'a> {
        outer_class: Option<cpool::Index<cpool::Class>>,
        inner_class: cpool::Index<cpool::Class>,
        inner_name: Option<cpool::Index<cpool::Utf8<'static>>>,
        inner_access_flags: AccessFlags,
    }
}

pub type BootstrapMethods<'a> = DecodeCountedCopy<'a, BootstrapMethod<'a>, u16>;
pub type BootstrapMethodIter<'a> = DecodeCounted<'a, BootstrapMethod<'a>, u16>;

crate::__dec_structure! {
    pub struct BootstrapMethod<'a> {
        method_ref: cpool::Index<cpool::MethodRef>,
        arguments: BootstrapArguments<'a>,
    }
}

pub type BootstrapArguments<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::MethodHandle>, u16>;
pub type BootstrapArgumentIter<'a> = DecodeCounted<'a, cpool::Index<cpool::MethodHandle>, u16>;
