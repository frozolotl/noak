use crate::encoding::*;
use crate::reader::cpool;

pub type ModulePackages<'a> = DecodeCountedCopy<'a, cpool::Index<cpool::Package>>;
pub type ModulePackageIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Package>>;
