mod lookupswitch;
mod tableswitch;

pub use lookupswitch::{LookupSwitchWriter, LookupSwitchWriterState};
pub use tableswitch::{TableSwitchWriter, TableSwitchWriterState};

use crate::error::*;
use crate::reader::{attributes::RawInstruction, decoding::*};
use crate::writer::{attributes::code::*, cpool, encoding::*};

pub struct InstructionWriter<Ctx> {
    code_writer: CodeWriter<Ctx, CodeWriterState::Instructions>,
    start_offset: Offset,
}

impl<Ctx: EncoderContext> InstructionWriter<Ctx> {
    /// The current offset starting from the code table start.
    pub(crate) fn current_offset(&mut self) -> Offset {
        self.code_writer.encoder().position().sub(self.start_offset)
    }

    pub fn new_label(&mut self) -> Result<(Label, LabelRef), EncodeError> {
        self.code_writer.new_label()
    }

    pub fn label(&mut self, label: Label) -> Result<&mut Self, EncodeError> {
        let offset = self
            .current_offset()
            .get()
            .checked_add(1)
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::Code))?;
        let offset = u32::try_from(offset)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::Code))?;
        self.code_writer.label_positions[label.0 as usize] = NonZeroU32::new(offset);
        Ok(self)
    }

    pub fn aaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x32u8)?;
        Ok(self)
    }

    pub fn aastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x53u8)?;
        Ok(self)
    }

    pub fn aconstnull(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x01u8)?;
        Ok(self)
    }

    pub fn aload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x19u8)?.write(index)?;
        Ok(self)
    }

    pub fn aload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x19u8)?.write(index)?;
        Ok(self)
    }

    pub fn aload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2au8)?;
        Ok(self)
    }

    pub fn aload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2bu8)?;
        Ok(self)
    }

    pub fn aload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2cu8)?;
        Ok(self)
    }

    pub fn aload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2du8)?;
        Ok(self)
    }

    pub fn anewarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.code_writer.encoder().write(0xbdu8)?;
        let index = array_type.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(index)?;
        Ok(self)
    }

    pub fn areturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xb0u8)?;
        Ok(self)
    }

    pub fn arraylength(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xbeu8)?;
        Ok(self)
    }

    pub fn astore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3au8)?.write(index)?;
        Ok(self)
    }

    pub fn astore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x3au8)?.write(index)?;
        Ok(self)
    }

    pub fn astore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4bu8)?;
        Ok(self)
    }

    pub fn astore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4cu8)?;
        Ok(self)
    }

    pub fn astore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4du8)?;
        Ok(self)
    }

    pub fn astore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4eu8)?;
        Ok(self)
    }

    pub fn athrow(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xbfu8)?;
        Ok(self)
    }

    pub fn baload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x33u8)?;
        Ok(self)
    }

    pub fn bastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x54u8)?;
        Ok(self)
    }

    pub fn bipush(&mut self, value: i8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x10u8)?.write(value)?;
        Ok(self)
    }

    pub fn caload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x34u8)?;
        Ok(self)
    }

    pub fn castore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x55u8)?;
        Ok(self)
    }

    pub fn checkcast<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xbdu8)?.write(index)?;
        Ok(self)
    }

    pub fn d2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x90u8)?;
        Ok(self)
    }

    pub fn d2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8eu8)?;
        Ok(self)
    }

    pub fn d2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8fu8)?;
        Ok(self)
    }

    pub fn dadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x63u8)?;
        Ok(self)
    }

    pub fn daload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x31u8)?;
        Ok(self)
    }

    pub fn dastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x52u8)?;
        Ok(self)
    }

    pub fn dcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x98u8)?;
        Ok(self)
    }

    pub fn dcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x97u8)?;
        Ok(self)
    }

    pub fn dconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0eu8)?;
        Ok(self)
    }

    pub fn dconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0fu8)?;
        Ok(self)
    }

    pub fn ddiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6fu8)?;
        Ok(self)
    }

    pub fn dload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x18u8)?.write(index)?;
        Ok(self)
    }

    pub fn dload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x18u8)?.write(index)?;
        Ok(self)
    }

    pub fn dload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x26u8)?;
        Ok(self)
    }

    pub fn dload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x27u8)?;
        Ok(self)
    }

    pub fn dload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x28u8)?;
        Ok(self)
    }

    pub fn dload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x29u8)?;
        Ok(self)
    }

    pub fn dmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6bu8)?;
        Ok(self)
    }

    pub fn dneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x77u8)?;
        Ok(self)
    }

    pub fn drem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x73u8)?;
        Ok(self)
    }

    pub fn dreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xafu8)?;
        Ok(self)
    }

    pub fn dstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x39u8)?.write(index)?;
        Ok(self)
    }

    pub fn dstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x39u8)?.write(index)?;
        Ok(self)
    }

    pub fn dstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x47u8)?;
        Ok(self)
    }

    pub fn dstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x48u8)?;
        Ok(self)
    }

    pub fn dstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x49u8)?;
        Ok(self)
    }

    pub fn dstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4au8)?;
        Ok(self)
    }

    pub fn dsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x67u8)?;
        Ok(self)
    }

    pub fn dup(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x59u8)?;
        Ok(self)
    }

    pub fn dupx1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x5au8)?;
        Ok(self)
    }

    pub fn dupx2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x5bu8)?;
        Ok(self)
    }

    pub fn dup2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x5cu8)?;
        Ok(self)
    }

    pub fn dup2x1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x5du8)?;
        Ok(self)
    }

    pub fn dup2x2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x5eu8)?;
        Ok(self)
    }

    pub fn f2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8du8)?;
        Ok(self)
    }

    pub fn f2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8bu8)?;
        Ok(self)
    }

    pub fn f2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8cu8)?;
        Ok(self)
    }

    pub fn fadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x62u8)?;
        Ok(self)
    }

    pub fn faload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x30u8)?;
        Ok(self)
    }

    pub fn fastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x51u8)?;
        Ok(self)
    }

    pub fn fcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x96u8)?;
        Ok(self)
    }

    pub fn fcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x95u8)?;
        Ok(self)
    }

    pub fn fconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0bu8)?;
        Ok(self)
    }

    pub fn fconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0cu8)?;
        Ok(self)
    }

    pub fn fconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0du8)?;
        Ok(self)
    }

    pub fn fdiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6eu8)?;
        Ok(self)
    }

    pub fn fload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x17u8)?.write(index)?;
        Ok(self)
    }

    pub fn fload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x17u8)?.write(index)?;
        Ok(self)
    }

    pub fn fload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x22u8)?;
        Ok(self)
    }

    pub fn fload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x23u8)?;
        Ok(self)
    }

    pub fn fload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x24u8)?;
        Ok(self)
    }

    pub fn fload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x25u8)?;
        Ok(self)
    }

    pub fn fmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6au8)?;
        Ok(self)
    }

    pub fn fneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x76u8)?;
        Ok(self)
    }

    pub fn frem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x72u8)?;
        Ok(self)
    }

    pub fn freturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xaeu8)?;
        Ok(self)
    }

    pub fn fstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x38u8)?.write(index)?;
        Ok(self)
    }

    pub fn fstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x38u8)?.write(index)?;
        Ok(self)
    }

    pub fn fstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x43u8)?;
        Ok(self)
    }

    pub fn fstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x44u8)?;
        Ok(self)
    }

    pub fn fstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x45u8)?;
        Ok(self)
    }

    pub fn fstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x46u8)?;
        Ok(self)
    }

    pub fn fsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x66u8)?;
        Ok(self)
    }

    pub fn getfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb4u8)?.write(index)?;
        Ok(self)
    }

    pub fn getstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb2u8)?.write(index)?;
        Ok(self)
    }

    pub fn goto(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa7u8)?.write(i)?;
            Ok(self)
        } else {
            self.gotow(label)
        }
    }

    pub fn gotow(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc8u8)?.write(label.0)?;
        Ok(self)
    }

    pub fn i2b(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x91u8)?;
        Ok(self)
    }

    pub fn i2c(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x92u8)?;
        Ok(self)
    }

    pub fn i2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x87u8)?;
        Ok(self)
    }

    pub fn i2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x86u8)?;
        Ok(self)
    }

    pub fn i2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x85u8)?;
        Ok(self)
    }

    pub fn i2s(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x93u8)?;
        Ok(self)
    }

    pub fn iadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x60u8)?;
        Ok(self)
    }

    pub fn iaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2eu8)?;
        Ok(self)
    }

    pub fn iand(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7eu8)?;
        Ok(self)
    }

    pub fn iastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x4fu8)?;
        Ok(self)
    }

    pub fn iconstm1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x02u8)?;
        Ok(self)
    }

    pub fn iconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x03u8)?;
        Ok(self)
    }

    pub fn iconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x04u8)?;
        Ok(self)
    }

    pub fn iconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x05u8)?;
        Ok(self)
    }

    pub fn iconst3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x06u8)?;
        Ok(self)
    }

    pub fn iconst4(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x07u8)?;
        Ok(self)
    }

    pub fn iconst5(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x08u8)?;
        Ok(self)
    }

    pub fn idiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6cu8)?;
        Ok(self)
    }

    pub fn ifacmpeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa5u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifacmpne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa6u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmpeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9fu8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmpne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa0u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmplt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa1u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmpge(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa2u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmpgt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa3u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ificmple(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa4u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x99u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9au8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn iflt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9bu8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifge(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9cu8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifgt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9du8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifle(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0x9eu8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifnonnull(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xc7u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn ifnull(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xc6u8)?.write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(EncodeErrorKind::LabelTooFar, Context::Code))
        }
    }

    pub fn iinc(&mut self, index: u8, value: i8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x84u8)?.write(index)?.write(value)?;
        Ok(self)
    }

    pub fn iinc_wide(&mut self, index: u16, value: i64) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .encoder()
            .write(0x4fu8)?
            .write(0x84u8)?
            .write(index)?
            .write(value)?;
        Ok(self)
    }

    pub fn iload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x15u8)?.write(index)?;
        Ok(self)
    }

    pub fn iload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x15u8)?.write(index)?;
        Ok(self)
    }

    pub fn iload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1au8)?;
        Ok(self)
    }

    pub fn iload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1bu8)?;
        Ok(self)
    }

    pub fn iload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1cu8)?;
        Ok(self)
    }

    pub fn iload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1du8)?;
        Ok(self)
    }

    pub fn imul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x68u8)?;
        Ok(self)
    }

    pub fn ineg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x74u8)?;
        Ok(self)
    }

    pub fn instanceof<I>(&mut self, type_: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = type_.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xc1u8)?.write(index)?;
        Ok(self)
    }

    pub fn invokedynamic<I>(&mut self, invoke_dynamic: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::InvokeDynamic>,
    {
        let index = invoke_dynamic.insert(&mut self.code_writer)?;
        self.code_writer
            .encoder()
            .write(0xbau8)?
            .write(index)?
            .write(0u8)?
            .write(0u8)?;
        Ok(self)
    }

    pub fn invokeinterface<I>(&mut self, method: I, count: u8) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::InterfaceMethodRef>,
    {
        let index = method.insert(&mut self.code_writer)?;
        self.code_writer
            .encoder()
            .write(0xb9u8)?
            .write(index)?
            .write(count)?
            .write(0u8)?;
        Ok(self)
    }

    pub fn invokespecial<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = method.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb7u8)?.write(index)?;
        Ok(self)
    }

    pub fn invokestatic<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = method.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb8u8)?.write(index)?;
        Ok(self)
    }

    pub fn invokevirtual<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::MethodRef>,
    {
        let index = method.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb6u8)?.write(index)?;
        Ok(self)
    }

    pub fn ior(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x80u8)?;
        Ok(self)
    }

    pub fn irem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x70u8)?;
        Ok(self)
    }

    pub fn ireturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xacu8)?;
        Ok(self)
    }

    pub fn ishl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x78u8)?;
        Ok(self)
    }

    pub fn ishr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7au8)?;
        Ok(self)
    }

    pub fn istore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x36u8)?.write(index)?;
        Ok(self)
    }

    pub fn istore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x36u8)?.write(index)?;
        Ok(self)
    }

    pub fn istore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3bu8)?;
        Ok(self)
    }

    pub fn istore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3cu8)?;
        Ok(self)
    }

    pub fn istore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3du8)?;
        Ok(self)
    }

    pub fn istore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3eu8)?;
        Ok(self)
    }

    pub fn isub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x64u8)?;
        Ok(self)
    }

    pub fn iushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7cu8)?;
        Ok(self)
    }

    pub fn ixor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x82u8)?;
        Ok(self)
    }

    pub fn jsr(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer.encoder().write(0xa8u8)?.write(i)?;
            Ok(self)
        } else {
            self.jsrw(label)
        }
    }

    pub fn jsrw(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc9u8)?.write(label.0)?;
        Ok(self)
    }

    pub fn l2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x8au8)?;
        Ok(self)
    }

    pub fn l2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x89u8)?;
        Ok(self)
    }

    pub fn l2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x88u8)?;
        Ok(self)
    }

    pub fn ladd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x61u8)?;
        Ok(self)
    }

    pub fn laload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x2fu8)?;
        Ok(self)
    }

    pub fn land(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7fu8)?;
        Ok(self)
    }

    pub fn lastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x50u8)?;
        Ok(self)
    }

    pub fn lcmp(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x94u8)?;
        Ok(self)
    }

    pub fn lconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x09u8)?;
        Ok(self)
    }

    pub fn lconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x0au8)?;
        Ok(self)
    }

    pub fn ldc<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(&mut self.code_writer)?.as_u16();
        let index = u8::try_from(index)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::IndexNotFitting, Context::Code))?;
        self.code_writer.encoder().write(0x12u8)?.write(index)?;
        Ok(self)
    }

    pub fn ldcw<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0x13u8)?.write(index)?;
        Ok(self)
    }

    pub fn ldc2w<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0x14u8)?.write(index)?;
        Ok(self)
    }

    pub fn ldiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x6du8)?;
        Ok(self)
    }

    pub fn lload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x16u8)?.write(index)?;
        Ok(self)
    }

    pub fn lload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x16u8)?.write(index)?;
        Ok(self)
    }

    pub fn lload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1eu8)?;
        Ok(self)
    }

    pub fn lload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x1fu8)?;
        Ok(self)
    }

    pub fn lload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x20u8)?;
        Ok(self)
    }

    pub fn lload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x21u8)?;
        Ok(self)
    }

    pub fn lmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x69u8)?;
        Ok(self)
    }

    pub fn lneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x75u8)?;
        Ok(self)
    }

    pub fn lookupswitch<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            LookupSwitchWriter<'f, Ctx, LookupSwitchWriterState::Default>,
        ) -> Result<LookupSwitchWriter<'f, Ctx, LookupSwitchWriterState::Jumps>, EncodeError>,
    {
        f(LookupSwitchWriter::new(self)?)?.finish()?;

        Ok(self)
    }

    pub fn lor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x81u8)?;
        Ok(self)
    }

    pub fn lrem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x71u8)?;
        Ok(self)
    }

    pub fn lreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xadu8)?;
        Ok(self)
    }

    pub fn lshl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x79u8)?;
        Ok(self)
    }

    pub fn lshr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7bu8)?;
        Ok(self)
    }

    pub fn lstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x37u8)?.write(index)?;
        Ok(self)
    }

    pub fn lstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0x37u8)?.write(index)?;
        Ok(self)
    }

    pub fn lstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x3fu8)?;
        Ok(self)
    }

    pub fn lstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x40u8)?;
        Ok(self)
    }

    pub fn lstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x41u8)?;
        Ok(self)
    }

    pub fn lstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x42u8)?;
        Ok(self)
    }

    pub fn lsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x65u8)?;
        Ok(self)
    }

    pub fn lushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x7du8)?;
        Ok(self)
    }

    pub fn lxor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x83u8)?;
        Ok(self)
    }

    pub fn monitorenter(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc2u8)?;
        Ok(self)
    }

    pub fn monitorexit(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc3u8)?;
        Ok(self)
    }

    pub fn multianewarray<I>(&mut self, array_type: I, dimensions: u8) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(&mut self.code_writer)?;
        self.code_writer
            .encoder()
            .write(0xc5u8)?
            .write(index)?
            .write(dimensions)?;
        Ok(self)
    }

    pub fn new<I>(&mut self, class: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = class.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xbbu8)?.write(index)?;
        Ok(self)
    }

    pub fn newarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xbcu8)?.write(index)?;
        Ok(self)
    }

    pub fn nop(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x00u8)?;
        Ok(self)
    }

    pub fn pop(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x57u8)?;
        Ok(self)
    }

    pub fn pop2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x58u8)?;
        Ok(self)
    }

    pub fn putfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb5u8)?.write(index)?;
        Ok(self)
    }

    pub fn putstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(&mut self.code_writer)?;
        self.code_writer.encoder().write(0xb3u8)?.write(index)?;
        Ok(self)
    }

    pub fn ret<I>(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xa9u8)?.write(index)?;
        Ok(self)
    }

    pub fn ret_wide<I>(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xc4u8)?.write(0xa9u8)?.write(index)?;
        Ok(self)
    }

    pub fn return_(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0xb1u8)?;
        Ok(self)
    }

    pub fn saload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x35u8)?;
        Ok(self)
    }

    pub fn sastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x56u8)?;
        Ok(self)
    }

    pub fn sipush(&mut self, value: i16) -> Result<&mut Self, EncodeError> {
        self.code_writer.encoder().write(0x11u8)?.write(value)?;
        Ok(self)
    }

    pub fn tableswitch<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(
            TableSwitchWriter<'f, Ctx, TableSwitchWriterState::Default>,
        ) -> Result<TableSwitchWriter<'f, Ctx, TableSwitchWriterState::Jumps>, EncodeError>,
    {
        f(TableSwitchWriter::new(self)?)?.finish()?;

        Ok(self)
    }
}

impl<Ctx: EncoderContext> WriteAssembler for InstructionWriter<Ctx> {
    type Context = CodeWriter<Ctx, CodeWriterState::Instructions>;

    fn new(mut code_writer: Self::Context) -> Result<Self, EncodeError> {
        let start_offset = code_writer.encoder().position();
        Ok(InstructionWriter {
            code_writer,
            start_offset,
        })
    }
}

impl<Ctx: EncoderContext> WriteDisassembler for InstructionWriter<Ctx> {
    type Context = CodeWriter<Ctx, CodeWriterState::Instructions>;

    fn finish(mut self) -> Result<Self::Context, EncodeError> {
        let start_offset = self.start_offset;
        let len = self.current_offset().get();
        let mut offset = 0;
        while offset < len {
            use RawInstruction::*;

            let instruction_start = start_offset.offset(offset);

            // we need to create a new decoder in each iteration as the bytes are modified later on in this block
            let mut decoder = Decoder::new(self.code_writer.encoder().buf(), Context::Code);
            // slicing does not work here because the decoder uses the current offset to compute table- and lookupswitch paddings
            decoder
                .advance(instruction_start.get())
                .expect("decoder failed to read encoded instruction");
            let prev_rem = decoder.bytes_remaining();
            let instruction = RawInstruction::decode(&mut decoder, start_offset.get())
                .expect("decoder failed to read encoded instruction");
            let diff = prev_rem - decoder.bytes_remaining();

            macro_rules! jmp_i16 {
                ($jump_offset:expr) => {{
                    let label_position = self
                        .code_writer
                        .get_label_position(LabelRef($jump_offset as u16 as u32))?;
                    let label_offset = label_position as i64 - offset as i64;

                    if let Ok(i) = i16::try_from(label_offset) {
                        let mut encoder = self.code_writer.encoder().replacing(instruction_start.offset(1));
                        encoder.write(i)?;
                    } else {
                        unimplemented!("noak does not support changing jump offset sizes yet");
                    }
                }};
            }

            macro_rules! jmp_i32 {
                ($read_offset:expr) => {{
                    let bytes = &self.code_writer.encoder().buf()[$read_offset.get()..];
                    let mut decoder = Decoder::new(bytes, Context::Code);
                    let label_index = LabelRef(decoder.read::<u32>().unwrap());
                    let label_position = self.code_writer.get_label_position(label_index)?;
                    let label_offset = label_position as i64 - offset as i64;

                    let mut encoder = self.code_writer.encoder().replacing($read_offset);

                    encoder.write(label_offset as i32)?;
                }};
            }

            match instruction {
                Goto { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                GotoW { .. } => {
                    jmp_i32!(instruction_start.offset(1));
                }
                IfACmpEq { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfACmpNe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpEq { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpNe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpLt { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpGe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpGt { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpLe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfEq { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfNe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfLt { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfGe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfGt { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfLe { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfNonNull { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                IfNull { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                JSr { offset: jump_offset } => {
                    jmp_i16!(jump_offset);
                }
                JSrW { .. } => {
                    jmp_i32!(instruction_start.offset(1));
                }
                LookupSwitch(lookupswitch) => {
                    let count = lookupswitch.pairs().count();
                    // skip opcode and padding
                    let offset_default = instruction_start.offset(1 + 3 - (offset & 3));

                    // write correct default offset
                    jmp_i32!(offset_default);

                    // skip default and count
                    let offset_pair_start = offset_default.offset(4 + 4);
                    for i in 0..count {
                        jmp_i32!(offset_pair_start.offset((i * 2 + 1) * 4));
                    }
                }
                TableSwitch(tableswitch) => {
                    let low = tableswitch.low();
                    let high = tableswitch.high();

                    // skip opcode and padding
                    let offset_default = instruction_start.offset(1 + 3 - (offset & 3));

                    // write correct default offset
                    jmp_i32!(offset_default);

                    // skip default, low and high
                    let offset_pair_start = offset_default.offset(4 + 4 + 4);
                    for i in low..=high {
                        jmp_i32!(offset_pair_start.offset(i as usize * 4));
                    }
                }
                _ => {}
            }

            offset += diff;
        }
        Ok(self.code_writer)
    }
}

impl<Ctx: EncoderContext> InternalEncoderContext for InstructionWriter<Ctx> {
    fn encoder(&mut self) -> &mut VecEncoder {
        self.code_writer.encoder()
    }
}

impl<Ctx: EncoderContext> EncoderContext for InstructionWriter<Ctx> {
    fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        self.code_writer.insert_constant(item)
    }
}

impl<Ctx> fmt::Debug for InstructionWriter<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstructionWriter").finish()
    }
}
