use crate::error::*;
use crate::reader::{attributes::RawInstruction, decoding::*};
use crate::writer::{attributes::code::*, cpool, encoding::*};
use std::convert::TryFrom;

pub struct InstructionWriter<'a, 'b> {
    code_writer: &'b mut CodeWriter<'a>,
    start_offset: Offset,
}

impl<'a, 'b> InstructionWriter<'a, 'b> {
    pub(crate) fn new(code_writer: &'b mut CodeWriter<'a>) -> Result<Self, EncodeError> {
        let start_offset = code_writer
            .class_writer
            .encoder
            .position()
            .sub(code_writer.class_writer.pool_end);
        Ok(InstructionWriter {
            code_writer,
            start_offset,
        })
    }

    pub(crate) fn finish(self) -> Result<(), EncodeError> {
        let pool_end = self.code_writer.class_writer.pool_end;
        let len = self.current_offset().get();
        let mut offset = 0;
        while offset < len {
            use RawInstruction::*;

            let instruction_start = self.start_offset.add(pool_end).offset(offset);
            let bytes = &self.code_writer.class_writer.encoder.buf()[instruction_start.get()..];
            let mut decoder = Decoder::new(bytes, Context::Code);
            let prev_rem = decoder.bytes_remaining();
            let instruction = RawInstruction::decode(&mut decoder, 0)
                .expect("decoder failed to read encoded instruction");
            let diff = prev_rem - decoder.bytes_remaining();

            macro_rules! jmp_i16 {
                ($jump_offset:expr) => {{
                    let label_position = self
                        .code_writer
                        .get_label_position(LabelRef($jump_offset as u16 as u32))?;
                    let label_offset = label_position as i64 - offset as i64;

                    if let Ok(i) = i16::try_from(label_offset) {
                        let mut encoder = self
                            .code_writer
                            .class_writer
                            .encoder
                            .replacing(instruction_start.offset(1));
                        encoder.write(i)?;
                    } else {
                        unimplemented!("noak does not support changing jump offset sizes yet");
                    }
                }};
            }

            macro_rules! jmp_i32 {
                ($read_offset:expr) => {{
                    let bytes = &self.code_writer.class_writer.encoder.buf()[$read_offset.get()..];
                    let mut decoder = Decoder::new(bytes, Context::Code);
                    let label_index = LabelRef(decoder.read::<u32>().unwrap());
                    let label_position = self.code_writer.get_label_position(label_index)?;
                    let label_offset = label_position as i64 - offset as i64;

                    let mut encoder = self
                        .code_writer
                        .class_writer
                        .encoder
                        .replacing($read_offset);

                    encoder.write(label_offset as i32)?;
                }};
            }

            match instruction {
                Goto {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                GotoW { .. } => {
                    jmp_i32!(instruction_start.offset(1));
                }
                IfACmpEq {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfACmpNe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpEq {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpNe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpLt {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpGe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpGt {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfICmpLe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfEq {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfNe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfLt {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfGe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfGt {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfLe {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfNonNull {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                IfNull {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                JSr {
                    offset: jump_offset,
                } => {
                    jmp_i16!(jump_offset);
                }
                JSrW { .. } => {
                    jmp_i32!(instruction_start.offset(1));
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
        Ok(())
    }

    /// The current offset starting from the code table start.
    fn current_offset(&self) -> Offset {
        self.code_writer
            .class_writer
            .encoder
            .position()
            .sub(self.start_offset)
            .sub(self.code_writer.class_writer.pool_end)
    }

    pub fn new_label(&mut self) -> Result<(Label, LabelRef), EncodeError> {
        self.code_writer.new_label()
    }

    pub fn write_label(&mut self, label: Label) -> Result<&mut Self, EncodeError> {
        let offset = self.current_offset().get().checked_add(1).ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::Code)
        })?;
        let offset = u32::try_from(offset)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::Code))?;
        self.code_writer.label_positions[label.0 as usize] = NonZeroU32::new(offset);
        Ok(self)
    }

    pub fn write_aaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x32u8)?;
        Ok(self)
    }

    pub fn write_aastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x53u8)?;
        Ok(self)
    }

    pub fn write_aconstnull(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x01u8)?;
        Ok(self)
    }

    pub fn write_aload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x19u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_aload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x19u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_aload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2au8)?;
        Ok(self)
    }

    pub fn write_aload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2bu8)?;
        Ok(self)
    }

    pub fn write_aload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2cu8)?;
        Ok(self)
    }

    pub fn write_aload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2du8)?;
        Ok(self)
    }

    pub fn write_anewarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        self.code_writer.class_writer.encoder.write(0xbdu8)?;
        let index = array_type.insert(self.code_writer.class_writer)?;
        self.code_writer.class_writer.encoder.write(index)?;
        Ok(self)
    }

    pub fn write_areturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xb0u8)?;
        Ok(self)
    }

    pub fn write_arraylength(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xbeu8)?;
        Ok(self)
    }

    pub fn write_astore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x3au8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_astore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x3au8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_astore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4bu8)?;
        Ok(self)
    }

    pub fn write_astore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4cu8)?;
        Ok(self)
    }

    pub fn write_astore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4du8)?;
        Ok(self)
    }

    pub fn write_astore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4eu8)?;
        Ok(self)
    }

    pub fn write_athrow(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xbfu8)?;
        Ok(self)
    }

    pub fn write_baload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x33u8)?;
        Ok(self)
    }

    pub fn write_bastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x54u8)?;
        Ok(self)
    }

    pub fn write_bipush(&mut self, value: i8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x10u8)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_caload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x34u8)?;
        Ok(self)
    }

    pub fn write_castore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x55u8)?;
        Ok(self)
    }

    pub fn write_checkcast<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xbdu8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_d2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x90u8)?;
        Ok(self)
    }

    pub fn write_d2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8eu8)?;
        Ok(self)
    }

    pub fn write_d2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8fu8)?;
        Ok(self)
    }

    pub fn write_dadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x63u8)?;
        Ok(self)
    }

    pub fn write_daload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x31u8)?;
        Ok(self)
    }

    pub fn write_dastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x52u8)?;
        Ok(self)
    }

    pub fn write_dcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x98u8)?;
        Ok(self)
    }

    pub fn write_dcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x97u8)?;
        Ok(self)
    }

    pub fn write_dconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0eu8)?;
        Ok(self)
    }

    pub fn write_dconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0fu8)?;
        Ok(self)
    }

    pub fn write_ddiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6fu8)?;
        Ok(self)
    }

    pub fn write_dload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x18u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x18u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x26u8)?;
        Ok(self)
    }

    pub fn write_dload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x27u8)?;
        Ok(self)
    }

    pub fn write_dload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x28u8)?;
        Ok(self)
    }

    pub fn write_dload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x29u8)?;
        Ok(self)
    }

    pub fn write_dmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6bu8)?;
        Ok(self)
    }

    pub fn write_dneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x77u8)?;
        Ok(self)
    }

    pub fn write_drem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x73u8)?;
        Ok(self)
    }

    pub fn write_dreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xafu8)?;
        Ok(self)
    }

    pub fn write_dstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x39u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x39u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_dstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x47u8)?;
        Ok(self)
    }

    pub fn write_dstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x48u8)?;
        Ok(self)
    }

    pub fn write_dstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x49u8)?;
        Ok(self)
    }

    pub fn write_dstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4au8)?;
        Ok(self)
    }

    pub fn write_dsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x67u8)?;
        Ok(self)
    }

    pub fn write_dup(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x59u8)?;
        Ok(self)
    }

    pub fn write_dupx1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x5au8)?;
        Ok(self)
    }

    pub fn write_dupx2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x5bu8)?;
        Ok(self)
    }

    pub fn write_dup2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x5cu8)?;
        Ok(self)
    }

    pub fn write_dup2x1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x5du8)?;
        Ok(self)
    }

    pub fn write_dup2x2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x5eu8)?;
        Ok(self)
    }

    pub fn write_f2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8du8)?;
        Ok(self)
    }

    pub fn write_f2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8bu8)?;
        Ok(self)
    }

    pub fn write_f2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8cu8)?;
        Ok(self)
    }

    pub fn write_fadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x62u8)?;
        Ok(self)
    }

    pub fn write_faload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x30u8)?;
        Ok(self)
    }

    pub fn write_fastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x51u8)?;
        Ok(self)
    }

    pub fn write_fcmpg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x96u8)?;
        Ok(self)
    }

    pub fn write_fcmpl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x95u8)?;
        Ok(self)
    }

    pub fn write_fconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0bu8)?;
        Ok(self)
    }

    pub fn write_fconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0cu8)?;
        Ok(self)
    }

    pub fn write_fconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0du8)?;
        Ok(self)
    }

    pub fn write_fdiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6eu8)?;
        Ok(self)
    }

    pub fn write_fload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x17u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x17u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x22u8)?;
        Ok(self)
    }

    pub fn write_fload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x23u8)?;
        Ok(self)
    }

    pub fn write_fload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x24u8)?;
        Ok(self)
    }

    pub fn write_fload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x25u8)?;
        Ok(self)
    }

    pub fn write_fmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6au8)?;
        Ok(self)
    }

    pub fn write_fneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x76u8)?;
        Ok(self)
    }

    pub fn write_frem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x72u8)?;
        Ok(self)
    }

    pub fn write_freturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xaeu8)?;
        Ok(self)
    }

    pub fn write_fstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x38u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x38u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_fstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x43u8)?;
        Ok(self)
    }

    pub fn write_fstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x44u8)?;
        Ok(self)
    }

    pub fn write_fstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x45u8)?;
        Ok(self)
    }

    pub fn write_fstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x46u8)?;
        Ok(self)
    }

    pub fn write_fsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x66u8)?;
        Ok(self)
    }

    pub fn write_getfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb4u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_getstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb2u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_goto(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa7u8)?
                .write(i)?;
            Ok(self)
        } else {
            self.write_gotow(label)
        }
    }

    pub fn write_gotow(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc8u8)?
            .write(label.0)?;
        Ok(self)
    }

    pub fn write_i2b(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x91u8)?;
        Ok(self)
    }

    pub fn write_i2c(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x92u8)?;
        Ok(self)
    }

    pub fn write_i2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x87u8)?;
        Ok(self)
    }

    pub fn write_i2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x86u8)?;
        Ok(self)
    }

    pub fn write_i2l(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x85u8)?;
        Ok(self)
    }

    pub fn write_i2s(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x93u8)?;
        Ok(self)
    }

    pub fn write_iadd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x60u8)?;
        Ok(self)
    }

    pub fn write_iaload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2eu8)?;
        Ok(self)
    }

    pub fn write_iand(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7eu8)?;
        Ok(self)
    }

    pub fn write_iastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x4fu8)?;
        Ok(self)
    }

    pub fn write_iconstm1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x02u8)?;
        Ok(self)
    }

    pub fn write_iconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x03u8)?;
        Ok(self)
    }

    pub fn write_iconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x04u8)?;
        Ok(self)
    }

    pub fn write_iconst2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x05u8)?;
        Ok(self)
    }

    pub fn write_iconst3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x06u8)?;
        Ok(self)
    }

    pub fn write_iconst4(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x07u8)?;
        Ok(self)
    }

    pub fn write_iconst5(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x08u8)?;
        Ok(self)
    }

    pub fn write_idiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6cu8)?;
        Ok(self)
    }

    pub fn write_ifacmpeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa5u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifacmpne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa6u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmpeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9fu8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmpne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa0u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmplt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa1u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmpge(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa2u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmpgt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa3u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ificmple(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa4u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifeq(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x99u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifne(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9au8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_iflt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9bu8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifge(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9cu8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifgt(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9du8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifle(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0x9eu8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifnonnull(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xc7u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_ifnull(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xc6u8)?
                .write(i)?;
            Ok(self)
        } else {
            Err(EncodeError::with_context(
                EncodeErrorKind::LabelTooFar,
                Context::Code,
            ))
        }
    }

    pub fn write_iinc(&mut self, index: u8, value: i8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x84u8)?
            .write(index)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_iinc_wide(&mut self, index: u16, value: i64) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x4fu8)?
            .write(0x84u8)?
            .write(index)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_iload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x15u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_iload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x15u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_iload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1au8)?;
        Ok(self)
    }

    pub fn write_iload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1bu8)?;
        Ok(self)
    }

    pub fn write_iload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1cu8)?;
        Ok(self)
    }

    pub fn write_iload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1du8)?;
        Ok(self)
    }

    pub fn write_imul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x68u8)?;
        Ok(self)
    }

    pub fn write_ineg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x74u8)?;
        Ok(self)
    }

    pub fn write_instanceof<I>(&mut self, r#type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = r#type.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xc1u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_invokedynamic<I>(&mut self, invoke_dynamic: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::InvokeDynamic>,
    {
        let index = invoke_dynamic.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
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
        let index = method.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
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
        let index = method.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb7u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_invokestatic<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = method.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb8u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_invokevirtual<I>(&mut self, method: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::MethodRef>,
    {
        let index = method.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb6u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ior(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x80u8)?;
        Ok(self)
    }

    pub fn write_irem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x70u8)?;
        Ok(self)
    }

    pub fn write_ireturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xacu8)?;
        Ok(self)
    }

    pub fn write_ishl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x78u8)?;
        Ok(self)
    }

    pub fn write_ishr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7au8)?;
        Ok(self)
    }

    pub fn write_istore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x36u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_istore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x36u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_istore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x3bu8)?;
        Ok(self)
    }

    pub fn write_istore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x3cu8)?;
        Ok(self)
    }

    pub fn write_istore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x3du8)?;
        Ok(self)
    }

    pub fn write_istore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x3eu8)?;
        Ok(self)
    }

    pub fn write_isub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x64u8)?;
        Ok(self)
    }

    pub fn write_iushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7cu8)?;
        Ok(self)
    }

    pub fn write_ixor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x82u8)?;
        Ok(self)
    }

    pub fn write_jsr(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        if let Ok(i) = u16::try_from(label.0) {
            self.code_writer
                .class_writer
                .encoder
                .write(0xa8u8)?
                .write(i)?;
            Ok(self)
        } else {
            self.write_jsrw(label)
        }
    }

    pub fn write_jsrw(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc9u8)?
            .write(label.0)?;
        Ok(self)
    }

    pub fn write_l2d(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x8au8)?;
        Ok(self)
    }

    pub fn write_l2f(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x89u8)?;
        Ok(self)
    }

    pub fn write_l2i(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x88u8)?;
        Ok(self)
    }

    pub fn write_ladd(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x61u8)?;
        Ok(self)
    }

    pub fn write_laload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x2fu8)?;
        Ok(self)
    }

    pub fn write_land(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7fu8)?;
        Ok(self)
    }

    pub fn write_lastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x50u8)?;
        Ok(self)
    }

    pub fn write_lcmp(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x94u8)?;
        Ok(self)
    }

    pub fn write_lconst0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x09u8)?;
        Ok(self)
    }

    pub fn write_lconst1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x0au8)?;
        Ok(self)
    }

    pub fn write_ldc<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = u8::try_from(constant.insert(self.code_writer.class_writer)?.as_u16())
            .map_err(|_| {
                EncodeError::with_context(EncodeErrorKind::IndexNotFitting, Context::Code)
            })?;
        self.code_writer
            .class_writer
            .encoder
            .write(0x12u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ldcw<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0x13u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ldc2w<I>(&mut self, constant: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Item>,
    {
        let index = constant.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0x14u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ldiv(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x6du8)?;
        Ok(self)
    }

    pub fn write_lload(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x16u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lload_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x16u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lload0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1eu8)?;
        Ok(self)
    }

    pub fn write_lload1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x1fu8)?;
        Ok(self)
    }

    pub fn write_lload2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x20u8)?;
        Ok(self)
    }

    pub fn write_lload3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x21u8)?;
        Ok(self)
    }

    pub fn write_lmul(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x69u8)?;
        Ok(self)
    }

    pub fn write_lneg(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x75u8)?;
        Ok(self)
    }

    // TODO LookUpSwitch

    pub fn write_lor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x81u8)?;
        Ok(self)
    }

    pub fn write_lrem(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x71u8)?;
        Ok(self)
    }

    pub fn write_lreturn(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xadu8)?;
        Ok(self)
    }

    pub fn write_lshl(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x79u8)?;
        Ok(self)
    }

    pub fn write_lshr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7bu8)?;
        Ok(self)
    }

    pub fn write_lstore(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x37u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lstore_wide(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0x37u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_lstore0(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x3fu8)?;
        Ok(self)
    }

    pub fn write_lstore1(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x40u8)?;
        Ok(self)
    }

    pub fn write_lstore2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x41u8)?;
        Ok(self)
    }

    pub fn write_lstore3(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x42u8)?;
        Ok(self)
    }

    pub fn write_lsub(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x65u8)?;
        Ok(self)
    }

    pub fn write_lushr(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x7du8)?;
        Ok(self)
    }

    pub fn write_lxor(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x83u8)?;
        Ok(self)
    }

    pub fn write_monitorenter(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xc2u8)?;
        Ok(self)
    }

    pub fn write_monitorexit(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xc3u8)?;
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
        let index = array_type.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
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
        let index = class.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xbbu8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_newarray<I>(&mut self, array_type: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::Class>,
    {
        let index = array_type.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xbcu8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_nop(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x00u8)?;
        Ok(self)
    }

    pub fn write_pop(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x57u8)?;
        Ok(self)
    }

    pub fn write_pop2(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x58u8)?;
        Ok(self)
    }

    pub fn write_putfield<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb5u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_putstatic<I>(&mut self, field: I) -> Result<&mut Self, EncodeError>
    where
        I: cpool::Insertable<cpool::FieldRef>,
    {
        let index = field.insert(self.code_writer.class_writer)?;
        self.code_writer
            .class_writer
            .encoder
            .write(0xb3u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ret<I>(&mut self, index: u8) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xa9u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_ret_wide<I>(&mut self, index: u16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0xc4u8)?
            .write(0xa9u8)?
            .write(index)?;
        Ok(self)
    }

    pub fn write_return(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0xb1u8)?;
        Ok(self)
    }

    pub fn write_saload(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x35u8)?;
        Ok(self)
    }

    pub fn write_sastore(&mut self) -> Result<&mut Self, EncodeError> {
        self.code_writer.class_writer.encoder.write(0x56u8)?;
        Ok(self)
    }

    pub fn write_sipush(&mut self, value: i16) -> Result<&mut Self, EncodeError> {
        self.code_writer
            .class_writer
            .encoder
            .write(0x11u8)?
            .write(value)?;
        Ok(self)
    }

    pub fn write_tableswitch<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&mut TableSwitchWriter<'a, 'f>) -> Result<(), EncodeError>,
    {
        let mut builder = TableSwitchWriter::new(self.code_writer, self.current_offset())?;
        f(&mut builder)?;
        builder.finish()?;

        Ok(self)
    }
}

pub struct TableSwitchWriter<'a, 'b> {
    code_writer: &'b mut CodeWriter<'a>,
    state: WriteSwitchState,
    remaining: u32,
}

impl<'a, 'b> TableSwitchWriter<'a, 'b> {
    fn new(code_writer: &'b mut CodeWriter<'a>, offset: Offset) -> Result<Self, EncodeError> {
        code_writer.class_writer.encoder.write(0xaau8)?;
        for _ in 0..3 - (offset.get() & 3) {
            code_writer.class_writer.encoder.write(0u8)?;
        }

        Ok(TableSwitchWriter {
            code_writer,
            state: WriteSwitchState::Default,
            remaining: 0,
        })
    }

    fn finish(self) -> Result<&'b mut CodeWriter<'a>, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteSwitchState::Finished, Context::Code)?;
        Ok(self.code_writer)
    }

    pub fn write_default(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteSwitchState::Default, Context::Code)?;

        self.code_writer.class_writer.encoder.write(label.0)?;
        self.state = WriteSwitchState::Low;
        Ok(self)
    }

    pub fn write_low(&mut self, low: i32) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteSwitchState::Low, Context::Code)?;

        self.code_writer.class_writer.encoder.write(low)?;
        self.remaining = low as u32;

        self.state = WriteSwitchState::High;
        Ok(self)
    }

    pub fn write_high(&mut self, high: i32) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteSwitchState::High, Context::Code)?;

        self.code_writer.class_writer.encoder.write(high)?;

        let low = self.remaining as i32;
        if low > high {
            return Err(EncodeError::with_context(
                EncodeErrorKind::IncorrectBounds,
                Context::Code,
            ));
        }

        self.remaining = (high - low + 1) as u32;

        self.state = WriteSwitchState::Jumps;
        Ok(self)
    }

    pub fn write_jump(&mut self, label: LabelRef) -> Result<&mut Self, EncodeError> {
        EncodeError::result_from_state(self.state, &WriteSwitchState::Jumps, Context::Code)?;
        self.code_writer.class_writer.encoder.write(label.0)?;
        if self.remaining == 1 {
            self.state = WriteSwitchState::Finished;
        } else if self.remaining == 0 {
            return Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                Context::Code,
            ));
        }

        self.remaining -= 1;

        Ok(self)
    }
}

/// What's written next
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WriteSwitchState {
    Default,
    Low,
    High,
    Jumps,
    Finished,
}
