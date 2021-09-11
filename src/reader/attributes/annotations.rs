mod annotation;
mod type_annotation;

pub use annotation::*;
pub use type_annotation::*;

use crate::reader::attributes::annotations::ElementValue;
use crate::reader::decoding::*;

dec_structure! {
    pub struct AnnotationDefault<'a> into {
        value: ElementValue<'a>,
    }
}
