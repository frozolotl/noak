use crate::encoding::{Decode, DecodeInto, Decoder};
use crate::error::*;
use crate::{header::AccessFlags, reader::cpool};
use std::fmt;
use std::iter::FusedIterator;

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

#[derive(Clone)]
pub struct InnerClasses<'a> {
    iter: InnerClassIter<'a>,
}

impl<'a> InnerClasses<'a> {
    pub fn iter(&self) -> InnerClassIter<'a> {
        self.iter.clone()
    }
}

impl<'a> DecodeInto<'a> for InnerClasses<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        // skip the count
        decoder.advance(2)?;

        Ok(InnerClasses {
            iter: InnerClassIter { decoder },
        })
    }
}

impl<'a> fmt::Debug for InnerClasses<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InnerClasses").finish()
    }
}

#[derive(Clone)]
pub struct InnerClassIter<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for InnerClassIter<'a> {
    type Item = InnerClass;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> FusedIterator for InnerClassIter<'a> {}

impl<'a> fmt::Debug for InnerClassIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InnerClassIter").finish()
    }
}

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
