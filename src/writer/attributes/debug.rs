use crate::error::*;
use crate::mutf8::MString;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a> AttributeWriter<'a> {
    pub fn write_source_file<I>(&mut self, file_name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let mut writer = self.attribute_writer("SourceFile")?;
        let file_name_index = file_name.insert(writer.class_writer())?;
        writer.write(file_name_index)?;
        writer.finish()?;
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
        let mut writer = self.attribute_writer("SourceDebugExtension")?;
        writer.write(debug_extension.into().as_bytes())?;
        writer.finish()?;
        self.finished = true;
        Ok(self)
    }

    pub fn write_synthetic(&mut self) -> Result<&mut Self, EncodeError> {
        self.attribute_writer("Synthetic")?.finish()?;
        self.finished = true;
        Ok(self)
    }
}
