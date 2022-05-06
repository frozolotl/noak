use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type ModulePackages<'a> = DecodeMany<'a, cpool::Index<cpool::Package>, u16>;
pub type ModulePackageIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Package>, u16>;

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

pub type Requires<'a> = DecodeMany<'a, Require<'a>, u16>;
pub type RequireIter<'a> = DecodeManyIter<'a, Require<'a>, u16>;

dec_structure! {
    pub struct Require<'a> {
        index: cpool::Index<cpool::Module>,
        flags: AccessFlags,
        version: cpool::Index<cpool::Utf8<'static>>,
    }
}

pub type Exports<'a> = DecodeMany<'a, Export<'a>, u16>;
pub type ExportIter<'a> = DecodeManyIter<'a, Export<'a>, u16>;

pub type ExportsToIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Export<'a> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        exports_to: ExportsToIter<'a>,
    }
}

pub type Opens<'a> = DecodeMany<'a, Open<'a>, u16>;
pub type OpenIter<'a> = DecodeManyIter<'a, Open<'a>, u16>;

pub type OpensToIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Open<'a> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        opens_to: OpensToIter<'a>,
    }
}

pub type Uses<'a> = DecodeMany<'a, cpool::Index<cpool::Class>, u16>;
pub type UseIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Class>, u16>;

pub type Provides<'a> = DecodeMany<'a, Provide<'a>, u16>;
pub type ProvideIter<'a> = DecodeManyIter<'a, Provide<'a>, u16>;

pub type ProvidesWithIter<'a> = DecodeManyIter<'a, cpool::Index<cpool::Class>, u16>;

dec_structure! {
    pub struct Provide<'a> {
        index: cpool::Index<cpool::Class>,
        provides_with: ProvidesWithIter<'a>,
    }
}
