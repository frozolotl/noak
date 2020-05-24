use crate::error::*;
use crate::mutf8::MString;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a, Ctx: EncoderContext> AttributeWriter<'a, Ctx> {
    pub fn write_source_file<I>(&mut self, file_name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let length_writer = self.attribute_writer("SourceFile")?;
        let file_name_index = file_name.insert(self.context)?;
        self.context
            .class_writer_mut()
            .encoder
            .write(file_name_index)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }

    pub fn write_source_debug_extension<I>(
        &mut self,
        debug_extension: I,
    ) -> Result<&mut Self, EncodeError>
    where
        I: Into<MString>,
    {
        let length_writer = self.attribute_writer("SourceDebugExtension")?;
        self.context
            .class_writer_mut()
            .encoder
            .write(debug_extension.into().as_bytes())?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }

    pub fn write_synthetic(&mut self) -> Result<&mut Self, EncodeError> {
        self.attribute_writer("Synthetic")?.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }

    pub fn write_deprecated(&mut self) -> Result<&mut Self, EncodeError> {
        self.attribute_writer("Deprecated")?.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }

    pub fn write_signature<I>(&mut self, signature: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let length_writer = self.attribute_writer("Signature")?;
        let signature_index = signature.insert(self.context)?;
        self.context
            .class_writer_mut()
            .encoder
            .write(signature_index)?;
        length_writer.finish(self.context)?;
        self.finished = true;
        Ok(self)
    }
}
