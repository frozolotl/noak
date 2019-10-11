use crate::error::*;
use crate::header::AccessFlags;
use crate::mutf8::MString;
use crate::writer::{cpool, encoding::*, ClassWriter};

pub struct AnnotationWriter<'a> {
    class_writer: &'a mut ClassWriter,
    counter: CountedEncoder,
}

impl<'a> AnnotationWriter<'a> {
    pub(crate) fn new(class_writer: &'a mut ClassWriter) -> Result<AnnotationWriter<'a>, EncodeError> {
        let counter = CountedEncoder::new(&mut class_writer.encoder)?;
        Ok(AnnotationWriter {
            class_writer,
            counter,
        })
    }
}
