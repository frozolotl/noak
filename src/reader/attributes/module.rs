use crate::encoding::*;
use crate::error::*;
use crate::reader::cpool;

pub type ModulePackages<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Package>>;
pub type ModulePackageIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Package>>;

#[derive(Clone)]
pub struct ModuleMainClass {
    main_class: cpool::Index<cpool::Class>,
}

impl ModuleMainClass {
    pub fn main_class(&self) -> cpool::Index<cpool::Class> {
        self.main_class
    }
}

impl<'a> DecodeInto<'a> for ModuleMainClass {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<ModuleMainClass, DecodeError> {
        Ok(ModuleMainClass {
            main_class: decoder.read()?,
        })
    }
}
