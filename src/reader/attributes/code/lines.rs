use crate::reader::attributes::code;
use crate::reader::decoding::*;

pub type LineNumberTable<'a> = DecodeCountedCopy<'a, Line<'a>, u16>;
pub type LineNumberIter<'a> = DecodeCounted<'a, Line<'a>, u16>;

crate::__dec_structure! {
    pub struct Line<'a> {
        start: code::Index,
        line_number: u16,
    }
}
