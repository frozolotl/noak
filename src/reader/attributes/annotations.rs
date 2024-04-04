mod annotation;
mod type_annotation;

pub use annotation::*;
pub use type_annotation::*;

use crate::reader::decoding::*;
use crate::{mutf8, MStr};

use super::FromAttribute;

dec_structure! {
    pub struct AnnotationDefault<'input> into {
        value: ElementValue<'input>,
    }
}

impl<'input> FromAttribute<'input> for AnnotationDefault<'input> {
    const NAME: &'static MStr = mutf8!("AnnotationDefault");
}

dec_structure! {
    pub struct RuntimeInvisibleAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u16>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeInvisibleAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeInvisibleAnnotations");
}

dec_structure! {
    pub struct RuntimeInvisibleParameterAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u8>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeInvisibleParameterAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeInvisibleParameterAnnotations");
}

dec_structure! {
    pub struct RuntimeInvisibleTypeAnnotations<'input> into {
        annotations: DecodeMany<'input, TypeAnnotation<'input>, u16>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeInvisibleTypeAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeInvisibleTypeAnnotations");
}

dec_structure! {
    pub struct RuntimeVisibleAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u16>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeVisibleAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeVisibleAnnotations");
}

dec_structure! {
    pub struct RuntimeVisibleParameterAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u8>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeVisibleParameterAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeVisibleParameterAnnotations");
}

dec_structure! {
    pub struct RuntimeVisibleTypeAnnotations<'input> into {
        annotations: DecodeMany<'input, TypeAnnotation<'input>, u16>,
    }
}

impl<'input> FromAttribute<'input> for RuntimeVisibleTypeAnnotations<'input> {
    const NAME: &'static MStr = mutf8!("RuntimeVisibleTypeAnnotations");
}
