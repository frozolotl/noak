use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type ModulePackages<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Package>, u16>;
pub type ModulePackageIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Package>, u16>;

dec_structure! {
    pub struct ModuleMainClass<'a> into {
        main_class: cpool::Index<cpool::Class>,
    }
}

dec_structure! {
    pub struct Module<'a> into {
        name: cpool::Index<cpool::Module>,
        flags: AccessFlags,
        version: Option<cpool::Index<cpool::Utf8<'static>>>,
        requires: Requires<'a>,
        exports: Exports<'a>,
        opens: Opens<'a>,
        uses: Uses<'a>,
        provides: Provides<'a>,
    }
}

pub type Requires<'a> = DecodeCountedCopy<'a, Require<'a>, u16>;
pub type RequireIter<'a> = DecodeCounted<'a, Require<'a>, u16>;

dec_structure! {
    pub struct Require<'a> {
        index: cpool::Index<cpool::Module>,
        flags: AccessFlags,
        version: cpool::Index<cpool::Utf8<'static>>,
    }
}

pub type Exports<'a> = DecodeCountedCopy<'a, Export<'a>, u16>;
pub type ExportIter<'a> = DecodeCounted<'a, Export<'a>, u16>;

pub type ExportsToIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Export<'a> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        exports_to: ExportsToIter<'a>,
    }
}

pub type Opens<'a> = DecodeCountedCopy<'a, Open<'a>, u16>;
pub type OpenIter<'a> = DecodeCounted<'a, Open<'a>, u16>;

pub type OpensToIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Open<'a> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        opens_to: OpensToIter<'a>,
    }
}

pub type Uses<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Class>, u16>;
pub type UseIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

pub type Provides<'a> = DecodeCountedCopy<'a, Provide<'a>, u16>;
pub type ProvideIter<'a> = DecodeCounted<'a, Provide<'a>, u16>;

pub type ProvidesWithIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

dec_structure! {
    pub struct Provide<'a> {
        index: cpool::Index<cpool::Class>,
        provides_with: ProvidesWithIter<'a>,
    }
}
