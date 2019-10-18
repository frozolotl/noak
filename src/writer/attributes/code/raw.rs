use crate::error::*;
use crate::writer::{cpool, encoding::*, ClassWriter};
use std::convert::TryFrom;

pub struct InstructionWriter<'a> {
    class_writer: &'a mut ClassWriter,
}

impl<'a> InstructionWriter<'a> {
    pub fn write_aaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x32u8)?;
        Ok(self)
    }

    pub fn write_aastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x53u8)?;
        Ok(self)
    }

    pub fn write_aconstnull(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x01u8)?;
        Ok(self)
    }

    pub fn write_aload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x19u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_aload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x19u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_aload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2au8)?;
        Ok(self)
    }

    pub fn write_aload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2bu8)?;
        Ok(self)
    }

    pub fn write_aload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2cu8)?;
        Ok(self)
    }

    pub fn write_aload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2du8)?;
        Ok(self)
    }

    pub fn write_anewarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.class_writer.encoder.write(0xbdu8)?;
        let index = array_type.insert(self.class_writer)?;
        self.class_writer.encoder.write(index)?;
        Ok(self)
    }

    pub fn write_areturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xb0u8)?;
        Ok(self)
    }

    pub fn write_arraylength(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xbeu8)?;
        Ok(self)
    }

    pub fn write_astore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3au8)?.write(index)?;
        Ok(self)
    }

    pub fn write_astore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x3au8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_astore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4bu8)?;
        Ok(self)
    }

    pub fn write_astore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4cu8)?;
        Ok(self)
    }

    pub fn write_astore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4du8)?;
        Ok(self)
    }

    pub fn write_astore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4eu8)?;
        Ok(self)
    }

    pub fn write_athrow(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xbfu8)?;
        Ok(self)
    }

    pub fn write_baload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x33u8)?;
        Ok(self)
    }

    pub fn write_bastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x54u8)?;
        Ok(self)
    }

    pub fn write_bipush(&mut self, value: i8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x10u8)?.write(value)?;
        Ok(self)
    }

    pub fn write_caload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x34u8)?;
        Ok(self)
    }

    pub fn write_castore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x55u8)?;
        Ok(self)
    }

    pub fn write_checkcast<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xbdu8)?.write(index)?;
        Ok(self)
    }

    pub fn write_d2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x90u8)?;
        Ok(self)
    }

    pub fn write_d2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8eu8)?;
        Ok(self)
    }

    pub fn write_d2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8fu8)?;
        Ok(self)
    }

    pub fn write_dadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x63u8)?;
        Ok(self)
    }

    pub fn write_daload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x31u8)?;
        Ok(self)
    }

    pub fn write_dastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x52u8)?;
        Ok(self)
    }

    pub fn write_dcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x98u8)?;
        Ok(self)
    }

    pub fn write_dcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x97u8)?;
        Ok(self)
    }

    pub fn write_dconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0eu8)?;
        Ok(self)
    }

    pub fn write_dconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0fu8)?;
        Ok(self)
    }

    pub fn write_ddiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6fu8)?;
        Ok(self)
    }

    pub fn write_dload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x18u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_dload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x18u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x26u8)?;
        Ok(self)
    }

    pub fn write_dload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x27u8)?;
        Ok(self)
    }

    pub fn write_dload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x28u8)?;
        Ok(self)
    }

    pub fn write_dload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x29u8)?;
        Ok(self)
    }

    pub fn write_dmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6bu8)?;
        Ok(self)
    }

    pub fn write_dneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x77u8)?;
        Ok(self)
    }

    pub fn write_drem(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x73u8)?;
        Ok(self)
    }

    pub fn write_dreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xafu8)?;
        Ok(self)
    }

    pub fn write_dstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x39u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_dstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x39u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x47u8)?;
        Ok(self)
    }

    pub fn write_dstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x48u8)?;
        Ok(self)
    }

    pub fn write_dstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x49u8)?;
        Ok(self)
    }

    pub fn write_dstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4au8)?;
        Ok(self)
    }

    pub fn write_dsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x67u8)?;
        Ok(self)
    }

    pub fn write_dup(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x59u8)?;
        Ok(self)
    }

    pub fn write_dupx1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x5au8)?;
        Ok(self)
    }

    pub fn write_dupx2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x5bu8)?;
        Ok(self)
    }

    pub fn write_dup2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x5cu8)?;
        Ok(self)
    }

    pub fn write_dup2x1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x5du8)?;
        Ok(self)
    }

    pub fn write_dup2x2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x5eu8)?;
        Ok(self)
    }

    pub fn write_f2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8du8)?;
        Ok(self)
    }

    pub fn write_f2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8bu8)?;
        Ok(self)
    }

    pub fn write_f2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8cu8)?;
        Ok(self)
    }

    pub fn write_fadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x62u8)?;
        Ok(self)
    }

    pub fn write_faload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x30u8)?;
        Ok(self)
    }

    pub fn write_fastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x51u8)?;
        Ok(self)
    }

    pub fn write_fcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x96u8)?;
        Ok(self)
    }

    pub fn write_fcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x95u8)?;
        Ok(self)
    }

    pub fn write_fconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0bu8)?;
        Ok(self)
    }

    pub fn write_fconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0cu8)?;
        Ok(self)
    }

    pub fn write_fconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0du8)?;
        Ok(self)
    }

    pub fn write_fdiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6eu8)?;
        Ok(self)
    }

    pub fn write_fload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x17u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_fload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x17u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x22u8)?;
        Ok(self)
    }

    pub fn write_fload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x23u8)?;
        Ok(self)
    }

    pub fn write_fload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x24u8)?;
        Ok(self)
    }

    pub fn write_fload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x25u8)?;
        Ok(self)
    }

    pub fn write_fmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6au8)?;
        Ok(self)
    }

    pub fn write_fneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x76u8)?;
        Ok(self)
    }

    pub fn write_frem(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x72u8)?;
        Ok(self)
    }

    pub fn write_freturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xaeu8)?;
        Ok(self)
    }

    pub fn write_fstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x38u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_fstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x38u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x43u8)?;
        Ok(self)
    }

    pub fn write_fstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x44u8)?;
        Ok(self)
    }

    pub fn write_fstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x45u8)?;
        Ok(self)
    }

    pub fn write_fstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x46u8)?;
        Ok(self)
    }

    pub fn write_fsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x66u8)?;
        Ok(self)
    }

    pub fn write_getfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb4u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_getstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb2u8)?.write(index)?;
        Ok(self)
    }

    // TODO Goto, GotoW

    pub fn write_i2b(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x91u8)?;
        Ok(self)
    }

    pub fn write_i2c(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x92u8)?;
        Ok(self)
    }

    pub fn write_i2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x87u8)?;
        Ok(self)
    }

    pub fn write_i2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x86u8)?;
        Ok(self)
    }

    pub fn write_i2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x85u8)?;
        Ok(self)
    }

    pub fn write_i2s(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x93u8)?;
        Ok(self)
    }

    pub fn write_iadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x60u8)?;
        Ok(self)
    }

    pub fn write_iaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2eu8)?;
        Ok(self)
    }

    pub fn write_iand(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7eu8)?;
        Ok(self)
    }

    pub fn write_iastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x4fu8)?;
        Ok(self)
    }

    pub fn write_iconstm1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x02u8)?;
        Ok(self)
    }

    pub fn write_iconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x03u8)?;
        Ok(self)
    }

    pub fn write_iconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x04u8)?;
        Ok(self)
    }

    pub fn write_iconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x05u8)?;
        Ok(self)
    }

    pub fn write_iconst3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x06u8)?;
        Ok(self)
    }

    pub fn write_iconst4(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x07u8)?;
        Ok(self)
    }

    pub fn write_iconst5(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x08u8)?;
        Ok(self)
    }

    pub fn write_idiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6cu8)?;
        Ok(self)
    }

    // TODO IfAcmpEq, IfACmpNe, IfICmpEq, IfICmpNe, IfICmpLt, IfICmpGe, IfICmpGt, IfICmpLe
    // TODO IfEq, IfNe, IfLt, IfGe, IfGt, IfLe, IfNonNull, IfNull

    pub fn write_iinc(&mut self, index: u8, value: i8) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0x84u8)?
            .write(index)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_iinc_wide(&mut self, index: u16, value: i64) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0x4fu8)?
            .write(0x84u8)?
            .write(index)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_iload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x15u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_iload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x15u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_iload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1au8)?;
        Ok(self)
    }

    pub fn write_iload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1bu8)?;
        Ok(self)
    }

    pub fn write_iload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1cu8)?;
        Ok(self)
    }

    pub fn write_iload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1du8)?;
        Ok(self)
    }

    pub fn write_imul(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x68u8)?;
        Ok(self)
    }

    pub fn write_ineg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x74u8)?;
        Ok(self)
    }

    pub fn write_instanceof<I>(&mut self, r#type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = r#type.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xc1u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_invokedynamic<I>(&mut self, invoke_dynamic: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::InvokeDynamic>,
    {
        let index = invoke_dynamic.insert(self.class_writer)?;
        self.class_writer
            .encoder
            .write(0xbau8)?
            .write(index)?
            .write(0u8)?
            .write(0u8)?;
        Ok(self)
    }

    pub fn write_invokeinterface<I>(
        &mut self,
        method: I,
        count: u8,
    ) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::InterfaceMethodRef>,
    {
        let index = method.insert(self.class_writer)?;
        self.class_writer
            .encoder
            .write(0xb9u8)?
            .write(index)?
            .write(count)?
            .write(0u8)?;
        Ok(self)
    }

    pub fn write_invokespecial<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = method.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb7u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_invokestatic<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = method.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb8u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_invokevirtual<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::MethodRef>,
    {
        let index = method.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb6u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ior(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x80u8)?;
        Ok(self)
    }

    pub fn write_irem(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x70u8)?;
        Ok(self)
    }

    pub fn write_ireturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xacu8)?;
        Ok(self)
    }

    pub fn write_ishl(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x78u8)?;
        Ok(self)
    }

    pub fn write_ishr(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7au8)?;
        Ok(self)
    }

    pub fn write_istore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x36u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_istore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x36u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_istore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3bu8)?;
        Ok(self)
    }

    pub fn write_istore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3cu8)?;
        Ok(self)
    }

    pub fn write_istore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3du8)?;
        Ok(self)
    }

    pub fn write_istore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3eu8)?;
        Ok(self)
    }

    pub fn write_isub(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x64u8)?;
        Ok(self)
    }

    pub fn write_iushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7cu8)?;
        Ok(self)
    }

    pub fn write_ixor(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x82u8)?;
        Ok(self)
    }

    // TODO Jsr, JSrW

    pub fn write_l2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x8au8)?;
        Ok(self)
    }

    pub fn write_l2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x89u8)?;
        Ok(self)
    }

    pub fn write_l2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x88u8)?;
        Ok(self)
    }

    pub fn write_ladd(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x61u8)?;
        Ok(self)
    }

    pub fn write_laload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x2fu8)?;
        Ok(self)
    }

    pub fn write_land(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7fu8)?;
        Ok(self)
    }

    pub fn write_lastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x50u8)?;
        Ok(self)
    }

    pub fn write_lcmp(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x94u8)?;
        Ok(self)
    }

    pub fn write_lconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x09u8)?;
        Ok(self)
    }

    pub fn write_lconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x0au8)?;
        Ok(self)
    }

    pub fn write_ldc<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = u8::try_from(constant.insert(self.class_writer)?.as_u16()).map_err(|_| {
            EncodeError::with_context(EncodeErrorKind::IndexNotFitting, Context::Code)
        })?;
        self.class_writer.encoder.write(0x12u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ldcw<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(self.class_writer)?;
        self.class_writer.encoder.write(0x13u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ldc2w<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(self.class_writer)?;
        self.class_writer.encoder.write(0x14u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ldiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x6du8)?;
        Ok(self)
    }

    pub fn write_lload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x16u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_lload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x16u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1eu8)?;
        Ok(self)
    }

    pub fn write_lload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x1fu8)?;
        Ok(self)
    }

    pub fn write_lload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x20u8)?;
        Ok(self)
    }

    pub fn write_lload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x21u8)?;
        Ok(self)
    }

    pub fn write_lmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x69u8)?;
        Ok(self)
    }

    pub fn write_lneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x75u8)?;
        Ok(self)
    }

    // TODO LookUpSwitch

    pub fn write_lor(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x81u8)?;
        Ok(self)
    }

    pub fn write_lrem(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x71u8)?;
        Ok(self)
    }

    pub fn write_lreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xadu8)?;
        Ok(self)
    }

    pub fn write_lshl(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x79u8)?;
        Ok(self)
    }

    pub fn write_lshr(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7bu8)?;
        Ok(self)
    }

    pub fn write_lstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x37u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_lstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x37u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x3fu8)?;
        Ok(self)
    }

    pub fn write_lstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x40u8)?;
        Ok(self)
    }

    pub fn write_lstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x41u8)?;
        Ok(self)
    }

    pub fn write_lstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x42u8)?;
        Ok(self)
    }

    pub fn write_lsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x65u8)?;
        Ok(self)
    }

    pub fn write_lushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x7du8)?;
        Ok(self)
    }

    pub fn write_lxor(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x83u8)?;
        Ok(self)
    }

    pub fn write_monitorenter(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xc2u8)?;
        Ok(self)
    }

    pub fn write_monitorexit(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xc3u8)?;
        Ok(self)
    }

    pub fn write_multianewarray<I>(
        &mut self,
        array_type: I,
        dimensions: u8,
    ) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(self.class_writer)?;
        self.class_writer
            .encoder
            .write(0xc5u8)?
            .write(index)?
            .write(dimensions)?;
        Ok(self)
    }

    pub fn write_new<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xbbu8)?.write(index)?;
        Ok(self)
    }

    pub fn write_newarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xbcu8)?.write(index)?;
        Ok(self)
    }

    pub fn write_nop(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x00u8)?;
        Ok(self)
    }

    pub fn write_pop(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x57u8)?;
        Ok(self)
    }

    pub fn write_pop2(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x58u8)?;
        Ok(self)
    }

    pub fn write_putfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb5u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_putstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.class_writer)?;
        self.class_writer.encoder.write(0xb3u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ret<I>(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xa9u8)?.write(index)?;
        Ok(self)
    }

    pub fn write_ret_wide<I>(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.class_writer
            .encoder
            .write(0xc4u8)?
            .write(0xa9u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_return(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0xb1u8)?;
        Ok(self)
    }

    pub fn write_saload(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x35u8)?;
        Ok(self)
    }

    pub fn write_sastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x56u8)?;
        Ok(self)
    }

    pub fn write_sipush(&mut self, value: i16) -> Result<&mut Self, EncodeError> {
        self.class_writer.encoder.write(0x11u8)?.write(value)?;
        Ok(self)
    }

    // TODO TableSwitch
}

impl<'a> WriteBuilder<'a> for InstructionWriter<'a> {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        Ok(InstructionWriter { class_writer })
    }

    fn finish(self) -> Result<&'a mut ClassWriter, EncodeError> {
        Ok(self.class_writer)
    }
}
