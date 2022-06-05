mod annotation;
mod type_annotation;

pub use annotation::*;
pub use type_annotation::*;

use crate::reader::attributes::annotations::ElementValue;
use crate::reader::decoding::*;

dec_structure! {
    pub struct AnnotationDefault<'input> into {
        value: ElementValue<'input>,
    }
}

dec_structure! {
    pub struct RuntimeInvisibleAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u16>,
    }
}

dec_structure! {
    pub struct RuntimeInvisibleParameterAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u8>,
    }
}

dec_structure! {
    pub struct RuntimeInvisibleTypeAnnotations<'input> into {
        annotations: DecodeMany<'input, TypeAnnotation<'input>, u16>,
    }
}

dec_structure! {
    pub struct RuntimeVisibleAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u16>,
    }
}

dec_structure! {
    pub struct RuntimeVisibleParameterAnnotations<'input> into {
        annotations: DecodeMany<'input, Annotation<'input>, u8>,
    }
}

dec_structure! {
    pub struct RuntimeVisibleTypeAnnotations<'input> into {
        annotations: DecodeMany<'input, TypeAnnotation<'input>, u16>,
    }
}
