use crate::error::*;
use crate::reader::decoding::*;
use crate::{header::AccessFlags, reader::cpool};
use std::fmt;

#[derive(Clone)]
pub struct EnclosingMethod {
    class: cpool::Index<cpool::Class>,
    method: cpool::Index<cpool::NameAndType>,
}

impl EnclosingMethod {
    pub fn class(&self) -> cpool::Index<cpool::Class> {
        self.class
    }

    pub fn method(&self) -> cpool::Index<cpool::NameAndType> {
        self.method
    }
}

impl<'a> DecodeInto<'a> for EnclosingMethod {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(EnclosingMethod {
            class: decoder.read()?,
            method: decoder.read()?,
        })
    }
}

impl fmt::Debug for EnclosingMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EnclosingMethod").finish()
    }
}

#[derive(Clone)]
pub struct NestHost {
    host_class: cpool::Index<cpool::Class>,
}

impl NestHost {
    pub fn host_class(&self) -> cpool::Index<cpool::Class> {
        self.host_class
    }
}

impl<'a> DecodeInto<'a> for NestHost {
    fn decode_into(mut decoder: Decoder) -> Result<Self, DecodeError> {
        Ok(NestHost {
            host_class: decoder.read()?,
        })
    }
}

impl fmt::Debug for NestHost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NestHost").finish()
    }
}

pub type NestMembers<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Class>>;
pub type NestMemberIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

pub type InnerClasses<'a> = DecodeCountedCopy<'a, InnerClass>;
pub type InnerClassIter<'a> = DecodeCounted<'a, InnerClass, u16>;

#[derive(Clone)]
pub struct InnerClass {
    outer_class: Option<cpool::Index<cpool::Class>>,
    inner_class: cpool::Index<cpool::Class>,
    inner_name: Option<cpool::Index<cpool::Utf8<'static>>>,
    inner_access_flags: AccessFlags,
}

impl InnerClass {
    pub fn outer_class(&self) -> Option<cpool::Index<cpool::Class>> {
        self.outer_class
    }

    pub fn inner_class(&self) -> cpool::Index<cpool::Class> {
        self.inner_class
    }

    pub fn inner_name(&self) -> Option<cpool::Index<cpool::Utf8<'static>>> {
        self.inner_name
    }

    pub fn inner_access_flags(&self) -> AccessFlags {
        self.inner_access_flags
    }
}

impl<'a> Decode<'a> for InnerClass {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let inner_class = decoder.read()?;
        let outer_class = decoder.read()?;
        let inner_name = decoder.read()?;
        let inner_access_flags = decoder.read()?;

        Ok(InnerClass {
            inner_class,
            outer_class,
            inner_name,
            inner_access_flags,
        })
    }
}

impl fmt::Debug for InnerClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InnerClass").finish()
    }
}

pub type BootstrapMethods<'a> = DecodeCountedCopy<'a, BootstrapMethod<'a>>;
pub type BootstrapMethodIter<'a> = DecodeCounted<'a, BootstrapMethod<'a>, u16>;

#[derive(Clone)]
pub struct BootstrapMethod<'a> {
    method_ref: cpool::Index<cpool::MethodRef>,
    arguments: BootstrapArguments<'a>,
}

impl<'a> BootstrapMethod<'a> {
    pub fn method_ref(&self) -> cpool::Index<cpool::MethodRef> {
        self.method_ref
    }

    pub fn arguments(&self) -> BootstrapArguments<'a> {
        self.arguments.clone()
    }
}

impl<'a> Decode<'a> for BootstrapMethod<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(BootstrapMethod {
            method_ref: decoder.read()?,
            arguments: decoder.read()?,
        })
    }

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        decoder.skip::<u16>()?; // method reference
        decoder.skip::<BootstrapArguments>()?;
        Ok(())
    }
}

impl<'a> fmt::Debug for BootstrapMethod<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BootstrapMethod").finish()
    }
}

pub type BootstrapArguments<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::MethodHandle>>;
pub type BootstrapArgumentIter<'a> = DecodeCounted<'a, cpool::Index<cpool::MethodHandle>, u16>;
