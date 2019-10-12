use crate::error::*;
use crate::writer::{cpool, encoding::*, AttributeWriter};

impl<'a> AttributeWriter<'a> {
    pub fn write_source_file<I>(&mut self, file_name: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let mut writer = self.attribute_writer("SourceFile")?;
        let class_writer = writer.class_writer();
        let file_name_index = file_name.insert(class_writer)?;
        writer.write(file_name_index)?;
        writer.finish()?;

        Ok(self)
    }
}
