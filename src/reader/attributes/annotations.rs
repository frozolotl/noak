mod annotation;
mod type_annotation;

pub use annotation::*;
pub use type_annotation::*;

use crate::reader::attributes::annotations::ElementValue;

crate::__dec_structure! {
    pub struct AnnotationDefault<'a> into {
        value: ElementValue<'a>,
    }
}
