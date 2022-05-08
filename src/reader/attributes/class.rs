use crate::reader::decoding::*;
use crate::{
    header::AccessFlags,
    reader::{cpool, AttributeIter},
};

dec_structure! {
    pub struct EnclosingMethod<'a> into {
        class: cpool::Index<cpool::Class>,
        method: cpool::Index<cpool::NameAndType>,
    }
}

dec_structure! {
    pub struct NestHost<'a> into {
        host_class: cpool::Index<cpool::Class>,
    }
}

pub type NestMembers<'a> = DecodeMany<'a, cpool::Index<cpool::Class>, u16>;
pub type NestMemberIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Class>, u16>;

pub type InnerClasses<'a> = DecodeMany<'a, InnerClass<'a>, u16>;
pub type InnerClassIter<'a> = DecodeManyIter<'a, InnerClass<'a>, u16>;

dec_structure! {
    pub struct InnerClass<'a> {
        inner_class: cpool::Index<cpool::Class>,
        outer_class: Option<cpool::Index<cpool::Class>>,
        inner_name: Option<cpool::Index<cpool::Utf8<'static>>>,
        inner_access_flags: AccessFlags,
    }
}

pub type BootstrapMethods<'a> = DecodeMany<'a, BootstrapMethod<'a>, u16>;
pub type BootstrapMethodIter<'a> = DecodeManyIter<'a, BootstrapMethod<'a>, u16>;

dec_structure! {
    pub struct BootstrapMethod<'a> {
        method_ref: cpool::Index<cpool::MethodRef>,
        arguments: BootstrapArguments<'a>,
    }
}

pub type BootstrapArguments<'a> = DecodeMany<'a, cpool::Index<cpool::MethodHandle>, u16>;
pub type BootstrapArgumentIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::MethodHandle>, u16>;

dec_structure! {
    pub struct Record<'a> into {
        components: RecordComponents<'a>,
    }
}

dec_structure! {
    pub struct RecordComponent<'a> {
        name: cpool::Index<cpool::Utf8<'static>>,
        descriptor: cpool::Index<cpool::Utf8<'static>>,
        attributes: AttributeIter<'a>,
    }
}

pub type RecordComponents<'a> = DecodeMany<'a, RecordComponent<'a>, u16>;
pub type RecordComponentIter<'a> = DecodeManyIter<'a, RecordComponent<'a>, u16>;

pub type PermittedSubclasses<'a> = DecodeMany<'a, cpool::Index<cpool::Class>, u16>;
pub type PermittedSubclassesIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Class>, u16>;
