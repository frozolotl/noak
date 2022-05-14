use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

pub type ModulePackages<'input> = DecodeMany<'input, cpool::Index<cpool::Package>, u16>;
pub type ModulePackageIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Package>, u16>;

dec_structure! {
    pub struct ModuleMainClass<'input> into {
        main_class: cpool::Index<cpool::Class>,
    }
}

dec_structure! {
    pub struct Module<'input> into {
        name: cpool::Index<cpool::Module>,
        flags: AccessFlags,
        version: Option<cpool::Index<cpool::Utf8<'static>>>,
        requires: Requires<'input>,
        exports: Exports<'input>,
        opens: Opens<'input>,
        uses: Uses<'input>,
        provides: Provides<'input>,
    }
}

pub type Requires<'input> = DecodeMany<'input, Require<'input>, u16>;
pub type RequireIter<'input> = DecodeManyIter<'input, Require<'input>, u16>;

dec_structure! {
    pub struct Require<'input> {
        index: cpool::Index<cpool::Module>,
        flags: AccessFlags,
        version: cpool::Index<cpool::Utf8<'static>>,
    }
}

pub type Exports<'input> = DecodeMany<'input, Export<'input>, u16>;
pub type ExportIter<'input> = DecodeManyIter<'input, Export<'input>, u16>;

pub type ExportsToIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Export<'input> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        exports_to: ExportsToIter<'input>,
    }
}

pub type Opens<'input> = DecodeMany<'input, Open<'input>, u16>;
pub type OpenIter<'input> = DecodeManyIter<'input, Open<'input>, u16>;

pub type OpensToIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Module>, u16>;

dec_structure! {
    pub struct Open<'input> {
        index: cpool::Index<cpool::Package>,
        flags: AccessFlags,
        opens_to: OpensToIter<'input>,
    }
}

pub type Uses<'input> = DecodeMany<'input, cpool::Index<cpool::Class>, u16>;
pub type UseIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class>, u16>;

pub type Provides<'input> = DecodeMany<'input, Provide<'input>, u16>;
pub type ProvideIter<'input> = DecodeManyIter<'input, Provide<'input>, u16>;

pub type ProvidesWithIter<'input> = DecodeManyIter<'input, cpool::Index<cpool::Class>, u16>;

dec_structure! {
    pub struct Provide<'input> {
        index: cpool::Index<cpool::Class>,
        provides_with: ProvidesWithIter<'input>,
    }
}
