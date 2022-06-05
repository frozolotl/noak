use crate::header::AccessFlags;
use crate::reader::cpool;
use crate::reader::decoding::*;

dec_structure! {
    pub struct ModulePackages<'input> into {
        packages: DecodeMany<'input, cpool::Index<cpool::Package<'input>>, u16>,
    }
}

dec_structure! {
    pub struct ModuleMainClass<'input> into {
        main_class: cpool::Index<cpool::Class<'input>>,
    }
}

dec_structure! {
    pub struct Module<'input> into {
        name: cpool::Index<cpool::Module<'input>>,
        flags: AccessFlags,
        version: Option<cpool::Index<cpool::Utf8<'input>>>,
        requires: DecodeMany<'input, Require<'input>, u16>,
        exports: DecodeMany<'input, Export<'input>, u16>,
        opens: DecodeMany<'input, Open<'input>, u16>,
        uses: DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>,
        provides: DecodeMany<'input, Provide<'input>, u16>,
    }
}

dec_structure! {
    pub struct Require<'input> {
        index: cpool::Index<cpool::Module<'input>>,
        flags: AccessFlags,
        version: cpool::Index<cpool::Utf8<'input>>,
    }
}

dec_structure! {
    pub struct Export<'input> {
        index: cpool::Index<cpool::Package<'input>>,
        flags: AccessFlags,
        exports_to: DecodeMany<'input, cpool::Index<cpool::Module<'input>>, u16>,
    }
}

dec_structure! {
    pub struct Open<'input> {
        index: cpool::Index<cpool::Package<'input>>,
        flags: AccessFlags,
        opens_to: DecodeMany<'input, cpool::Index<cpool::Module<'input>>, u16>,
    }
}

dec_structure! {
    pub struct Provide<'input> {
        index: cpool::Index<cpool::Class<'input>>,
        provides_with: DecodeMany<'input, cpool::Index<cpool::Class<'input>>, u16>,
    }
}
