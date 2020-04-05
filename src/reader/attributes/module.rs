use crate::error::*;
use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;

pub type ModulePackages<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Package>>;
pub type ModulePackageIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Package>, u16>;

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
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(ModuleMainClass {
            main_class: decoder.read()?,
        })
    }
}

#[derive(Clone)]
pub struct Module<'a> {
    name: cpool::Index<cpool::Module>,
    flags: AccessFlags,
    version: Option<cpool::Index<cpool::Utf8<'static>>>,
    requires: Requires<'a>,
    exports: Exports<'a>,
    opens: Opens<'a>,
    uses: Uses<'a>,
    provides: Provides<'a>,
}

impl<'a> Module<'a> {
    pub fn name(&self) -> cpool::Index<cpool::Module> {
        self.name
    }

    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    pub fn version(&self) -> Option<cpool::Index<cpool::Utf8<'static>>> {
        self.version
    }

    pub fn requires(&self) -> Requires<'a> {
        self.requires.clone()
    }

    pub fn exports(&self) -> Exports<'a> {
        self.exports.clone()
    }

    pub fn opens(&self) -> Opens<'a> {
        self.opens.clone()
    }

    pub fn uses(&self) -> Uses<'a> {
        self.uses.clone()
    }

    pub fn provides(&self) -> Provides<'a> {
        self.provides.clone()
    }
}

impl<'a> DecodeInto<'a> for Module<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Module {
            name: decoder.read()?,
            flags: decoder.read()?,
            version: decoder.read()?,
            requires: decoder.read()?,
            exports: decoder.read()?,
            opens: decoder.read()?,
            uses: decoder.read()?,
            provides: decoder.read()?,
        })
    }
}

impl<'a> fmt::Debug for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Module").finish()
    }
}

pub type Requires<'a> = DecodeCountedCopy<'a, Require>;
pub type RequireIter<'a> = DecodeCounted<'a, Require, u16>;

#[derive(Clone)]
pub struct Require {
    index: cpool::Index<cpool::Module>,
    flags: AccessFlags,
    version: cpool::Index<cpool::Utf8<'static>>,
}

impl Require {
    pub fn index(&self) -> cpool::Index<cpool::Module> {
        self.index
    }

    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    pub fn version(&self) -> cpool::Index<cpool::Utf8<'static>> {
        self.version
    }
}

impl<'a> Decode<'a> for Require {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Require {
            index: decoder.read()?,
            flags: decoder.read()?,
            version: decoder.read()?,
        })
    }
}

impl fmt::Debug for Require {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Require").finish()
    }
}

pub type Exports<'a> = DecodeCountedCopy<'a, Export<'a>>;
pub type ExportIter<'a> = DecodeCounted<'a, Export<'a>, u16>;

pub type ExportsToIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Module>, u16>;

#[derive(Clone)]
pub struct Export<'a> {
    index: cpool::Index<cpool::Package>,
    flags: AccessFlags,
    exports_to: ExportsToIter<'a>,
}

impl<'a> Export<'a> {
    pub fn index(&self) -> cpool::Index<cpool::Package> {
        self.index
    }

    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    pub fn exports_to(&self) -> ExportsToIter<'a> {
        self.exports_to.clone()
    }
}

impl<'a> Decode<'a> for Export<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Export {
            index: decoder.read()?,
            flags: decoder.read()?,
            exports_to: decoder.read()?,
        })
    }
}

impl<'a> fmt::Debug for Export<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Export").finish()
    }
}

pub type Opens<'a> = DecodeCountedCopy<'a, Open<'a>>;
pub type OpenIter<'a> = DecodeCounted<'a, Open<'a>, u16>;

pub type OpensToIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Module>, u16>;

#[derive(Clone)]
pub struct Open<'a> {
    index: cpool::Index<cpool::Package>,
    flags: AccessFlags,
    opens_to: OpensToIter<'a>,
}

impl<'a> Open<'a> {
    pub fn index(&self) -> cpool::Index<cpool::Package> {
        self.index
    }

    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    pub fn opens_to(&self) -> OpensToIter<'a> {
        self.opens_to.clone()
    }
}

impl<'a> Decode<'a> for Open<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Open {
            index: decoder.read()?,
            flags: decoder.read()?,
            opens_to: decoder.read()?,
        })
    }
}

impl<'a> fmt::Debug for Open<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Open").finish()
    }
}

pub type Uses<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Class>>;
pub type UseIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

pub type Provides<'a> = DecodeCountedCopy<'a, Provide<'a>>;
pub type ProvideIter<'a> = DecodeCounted<'a, Provide<'a>, u16>;

pub type ProvidesWithIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

#[derive(Clone)]
pub struct Provide<'a> {
    index: cpool::Index<cpool::Class>,
    provides_with: ProvidesWithIter<'a>,
}

impl<'a> Provide<'a> {
    pub fn index(&self) -> cpool::Index<cpool::Class> {
        self.index
    }

    pub fn provides_with(&self) -> ProvidesWithIter<'a> {
        self.provides_with.clone()
    }
}

impl<'a> Decode<'a> for Provide<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(Provide {
            index: decoder.read()?,
            provides_with: decoder.read()?,
        })
    }
}

impl<'a> fmt::Debug for Provide<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Provide").finish()
    }
}
