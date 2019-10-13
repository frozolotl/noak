use crate::error::*;
use crate::mutf8::MString;
use crate::writer::{
    cpool::{self, Insertable},
    encoding::*,
    AttributeWriter, ClassWriter,
};

impl<'a> AttributeWriter<'a> {
    pub fn write_constant_value<I: Into<ConstantValue>>(
        &mut self,
        value: I,
    ) -> Result<&mut Self, EncodeError> {
        let length_writer = self.attribute_writer("ConstantValue")?;
        let value_index = value.into().insert(self.class_writer)?;
        self.class_writer.encoder.write(value_index)?;
        length_writer.finish(self.class_writer)?;
        self.finished = true;
        Ok(self)
    }
}

#[derive(Debug)]
pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(MString),
    Index(cpool::Index<cpool::Item>),
}

impl From<i32> for ConstantValue {
    fn from(value: i32) -> ConstantValue {
        ConstantValue::Integer(value)
    }
}

impl From<i64> for ConstantValue {
    fn from(value: i64) -> ConstantValue {
        ConstantValue::Long(value)
    }
}

impl From<f32> for ConstantValue {
    fn from(value: f32) -> ConstantValue {
        ConstantValue::Float(value)
    }
}

impl From<f64> for ConstantValue {
    fn from(value: f64) -> ConstantValue {
        ConstantValue::Double(value)
    }
}

impl<I: Into<MString>> From<I> for ConstantValue {
    fn from(string: I) -> ConstantValue {
        ConstantValue::String(string.into())
    }
}

impl From<cpool::Index<cpool::Item>> for ConstantValue {
    fn from(value: cpool::Index<cpool::Item>) -> ConstantValue {
        ConstantValue::Index(value)
    }
}

impl Insertable<cpool::Item> for ConstantValue {
    fn insert(
        self,
        class_writer: &mut ClassWriter,
    ) -> Result<cpool::Index<cpool::Item>, EncodeError> {
        match self {
            ConstantValue::Integer(value) => {
                class_writer.insert_constant(cpool::Item::Integer(cpool::Integer { value }))
            }
            ConstantValue::Long(value) => {
                class_writer.insert_constant(cpool::Item::Long(cpool::Long { value }))
            }
            ConstantValue::Float(value) => {
                class_writer.insert_constant(cpool::Item::Float(cpool::Float { value }))
            }
            ConstantValue::Double(value) => {
                class_writer.insert_constant(cpool::Item::Double(cpool::Double { value }))
            }
            ConstantValue::String(content) => {
                let string = class_writer.insert_constant(cpool::Utf8 { content })?;
                class_writer.insert_constant(cpool::Item::String(cpool::String { string }))
            }
            ConstantValue::Index(index) => Ok(index),
        }
    }
}
