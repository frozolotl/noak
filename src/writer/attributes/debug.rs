use std::marker::PhantomData;

use crate::error::*;
use crate::mutf8::MString;
use crate::writer::{
    attributes::{AttributeWriter, AttributeWriterState},
    cpool,
    encoding::*,
};

impl<Ctx: EncoderContext> AttributeWriter<Ctx, AttributeWriterState::Start> {
    pub fn source_file<I>(
        mut self,
        file_name: I,
    ) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let length_writer = self.attribute_writer("SourceFile")?;
        let file_name_index = file_name.insert(&mut self.context)?;
        self.context.encoder().write(file_name_index)?;
        length_writer.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn source_debug_extension<I>(
        mut self,
        debug_extension: I,
    ) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: Into<MString>,
    {
        let length_writer = self.attribute_writer("SourceDebugExtension")?;
        self.context.encoder().write(debug_extension.into().as_bytes())?;
        length_writer.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn synthetic(mut self) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError> {
        self.attribute_writer("Synthetic")?.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn deprecated(mut self) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError> {
        self.attribute_writer("Deprecated")?.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }

    pub fn signature<I>(mut self, signature: I) -> Result<AttributeWriter<Ctx, AttributeWriterState::End>, EncodeError>
    where
        I: cpool::Insertable<cpool::Utf8>,
    {
        let length_writer = self.attribute_writer("Signature")?;
        let signature_index = signature.insert(&mut self.context)?;
        self.context.encoder().write(signature_index)?;
        length_writer.finish(&mut self.context)?;
        Ok(AttributeWriter {
            context: self.context,
            _marker: PhantomData,
        })
    }
}
