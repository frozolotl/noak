use crate::reader::decoding::*;
use crate::{
    header::AccessFlags,
    reader::{cpool, AttributeIter},
};

dec_structure! {
    pub struct EnclosingMethod<'input> into {
        class: cpool::Index<cpool::Class<'input>>,
        method: Option<cpool::Index<cpool::NameAndType<'input>>>,
    }
}

dec_structure! {
    pub struct NestHost<'input> into {
        host_class: cpool::Index<cpool::Class<'input>>,
    }
}

pub type NestMembers<'input> = DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>;
pub type NestMemberIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class<'input>>, u16>;

pub type InnerClasses<'input> = DecodeMany<'input, InnerClass<'input>, u16>;
pub type InnerClassIter<'input> = DecodeManyIter<'input, InnerClass<'input>, u16>;

dec_structure! {
    pub struct InnerClass<'input> {
        inner_class: cpool::Index<cpool::Class<'input>>,
        outer_class: Option<cpool::Index<cpool::Class<'input>>>,
        inner_name: Option<cpool::Index<cpool::Utf8<'input>>>,
        inner_access_flags: AccessFlags,
    }
}

pub type BootstrapMethods<'input> = DecodeMany<'input, BootstrapMethod<'input>, u16>;
pub type BootstrapMethodIter<'input> = DecodeManyIter<'input, BootstrapMethod<'input>, u16>;

dec_structure! {
    pub struct BootstrapMethod<'input> {
        method_ref: cpool::Index<cpool::MethodRef<'input>>,
        arguments: BootstrapArguments<'input>,
    }
}

pub type BootstrapArguments<'input> = DecodeMany<'input, cpool::Index<cpool::MethodHandle<'input>>, u16>;
pub type BootstrapArgumentIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::MethodHandle<'input>>, u16>;

dec_structure! {
    pub struct Record<'input> into {
        components: RecordComponents<'input>,
    }
}

dec_structure! {
    pub struct RecordComponent<'input> {
        name: cpool::Index<cpool::Utf8<'input>>,
        descriptor: cpool::Index<cpool::Utf8<'input>>,
        attributes: AttributeIter<'input>,
    }
}

pub type RecordComponents<'input> = DecodeMany<'input, RecordComponent<'input>, u16>;
pub type RecordComponentIter<'input> = DecodeManyIter<'input, RecordComponent<'input>, u16>;

pub type PermittedSubclasses<'input> = DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>;
pub type PermittedSubclassesIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class<'input>>, u16>;
