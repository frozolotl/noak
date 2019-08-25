use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;

/// An iterator over the instructions and their indices into the code table
#[derive(Debug, Clone)]
pub struct RawInstructions<'a> {
    pub(in crate::reader::attributes::code) start_position: usize,
    pub(in crate::reader::attributes::code) decoder: Decoder<'a>,
}

impl<'a> Iterator for RawInstructions<'a> {
    type Item = Result<(code::Index, RawInstruction<'a>), DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.decoder.bytes_remaining() == 0 {
            return None;
        }

        let position = self.decoder.file_position() - self.start_position;
        match RawInstruction::decode(&mut self.decoder, self.start_position) {
            Ok(insn) => Some(Ok((code::Index::new(position as u32), insn))),
            Err(err) => Some(Err(err)),
        }
    }
}

#[derive(Debug)]
pub enum RawInstruction<'a> {
    AALoad,
    AAStore,
    AConstNull,
    ALoad {
        index: u8,
    },
    ALoadW {
        index: u16,
    },
    ALoad0,
    ALoad1,
    ALoad2,
    ALoad3,
    ANewArray {
        index: cpool::Index<cpool::Item<'a>>,
    },
    AReturn,
    ArrayLength,
    AStore {
        index: u8,
    },
    AStoreW {
        index: u16,
    },
    AStore0,
    AStore1,
    AStore2,
    AStore3,
    AThrow,
    BALoad,
    BAStore,
    BIPush {
        value: i8,
    },
    CALoad,
    CAStore,
    CheckCast {
        index: cpool::Index<cpool::Class>,
    },
    D2F,
    D2I,
    D2L,
    DAdd,
    DALoad,
    DAStore,
    DCmpG,
    DCmpL,
    DConst0,
    DConst1,
    DDiv,
    DLoad {
        index: u8,
    },
    DLoadW {
        index: u16,
    },
    DLoad0,
    DLoad1,
    DLoad2,
    DLoad3,
    DMul,
    DNeg,
    DRem,
    DReturn,
    DStore {
        index: u8,
    },
    DStoreW {
        index: u16,
    },
    DStore0,
    DStore1,
    DStore2,
    DStore3,
    DSub,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    F2D,
    F2I,
    F2L,
    FAdd,
    FALoad,
    FAStore,
    FCmpG,
    FCmpL,
    FConst0,
    FConst1,
    FConst2,
    FDiv,
    FLoad {
        index: u8,
    },
    FLoadW {
        index: u16,
    },
    FLoad0,
    FLoad1,
    FLoad2,
    FLoad3,
    FMul,
    FNeg,
    FRem,
    FReturn,
    FStore {
        index: u8,
    },
    FStoreW {
        index: u16,
    },
    FStore0,
    FStore1,
    FStore2,
    FStore3,
    FSub,
    GetField {
        index: cpool::Index<cpool::FieldRef>,
    },
    GetStatic {
        index: cpool::Index<cpool::FieldRef>,
    },
    Goto {
        offset: i16,
    },
    GotoW {
        offset: i32,
    },
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    IAdd,
    IALoad,
    IAnd,
    IAStore,
    IConstM1,
    IConst0,
    IConst1,
    IConst2,
    IConst3,
    IConst4,
    IConst5,
    IDiv,
    IfACmpEq {
        offset: i16,
    },
    IfACmpNe {
        offset: i16,
    },
    IfICmpEq {
        offset: i16,
    },
    IfICmpNe {
        offset: i16,
    },
    IfICmpLt {
        offset: i16,
    },
    IfICmpGe {
        offset: i16,
    },
    IFICmpGt {
        offset: i16,
    },
    IfICmpLe {
        offset: i16,
    },
    IfEq {
        offset: i16,
    },
    IfNe {
        offset: i16,
    },
    IfLt {
        offset: i16,
    },
    IfGe {
        offset: i16,
    },
    IfGt {
        offset: i16,
    },
    IfLe {
        offset: i16,
    },
    IfNonNull {
        offset: i16,
    },
    IfNull {
        offset: i16,
    },
    IInc {
        index: u8,
        value: i8,
    },
    IIncW {
        index: u16,
        value: i16,
    },
    ILoad {
        index: u8,
    },
    ILoadW {
        index: u16,
    },
    ILoad0,
    ILoad1,
    ILoad2,
    ILoad3,
    IMul,
    INeg,
    InstanceOf {
        index: cpool::Index<cpool::Class>,
    },
    InvokeDynamic {
        index: cpool::Index<cpool::InvokeDynamic>,
    },
    InvokeInterface {
        index: cpool::Index<cpool::InterfaceMethodRef>,
        count: u8,
    },
    InvokeSpecial {
        index: cpool::Index<cpool::Item<'a>>,
    },
    InvokeStatic {
        index: cpool::Index<cpool::Item<'a>>,
    },
    InvokeVirtual {
        index: cpool::Index<cpool::MethodRef>,
    },
    IOr,
    IRem,
    IReturn,
    IShL,
    IShR,
    IStore {
        index: u8,
    },
    IStoreW {
        index: u16,
    },
    IStore0,
    IStore1,
    IStore2,
    IStore3,
    ISub,
    IUShR,
    IXor,
    JSr {
        offset: i16,
    },
    JSrW {
        offset: i32,
    },
    L2D,
    L2F,
    L2I,
    LAdd,
    LALoad,
    LAnd,
    LAStore,
    LCmp,
    LConst0,
    LConst1,
    LdC {
        index: cpool::Index<cpool::Item<'a>>,
    },
    LdCW {
        index: cpool::Index<cpool::Item<'a>>,
    },
    LdC2W {
        index: cpool::Index<cpool::Item<'a>>,
    },
    LDiv,
    LLoad {
        index: u8,
    },
    LLoadW {
        index: u16,
    },
    LLoad0,
    LLoad1,
    LLoad2,
    LLoad3,
    LMul,
    LNeg,
    LookUpSwitch(LookUpSwitch<'a>),
    LOr,
    LRem,
    LReturn,
    LShL,
    LShR,
    LStore {
        index: u8,
    },
    LStoreW {
        index: u16,
    },
    LStore0,
    LStore1,
    LStore2,
    LStore3,
    LSub,
    LUShR,
    LXor,
    MonitorEnter,
    MonitorExit,
    MultiANewArray {
        index: cpool::Index<cpool::Class>,
        dimensions: u8,
    },
    New {
        index: cpool::Index<cpool::Class>,
    },
    NewArray {
        atype: ArrayType,
    },
    Nop,
    Pop,
    Pop2,
    PutField {
        index: cpool::Index<cpool::FieldRef>,
    },
    PutStatic {
        index: cpool::Index<cpool::FieldRef>,
    },
    Ret {
        index: u8,
    },
    RetW {
        index: u16,
    },
    Return,
    SALoad,
    SAStore,
    SIPush {
        value: i16,
    },
    Swap,
    TableSwitch(TableSwitch<'a>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArrayType {
    Boolean,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
}

#[derive(Debug, Clone)]
pub struct LookUpSwitch<'a> {
    default_offset: i32,
    pairs: LookUpPairs<'a>,
}

impl<'a> LookUpSwitch<'a> {
    pub fn default_offset(&self) -> i32 {
        self.default_offset
    }

    pub fn pairs(&self) -> LookUpPairs<'a> {
        self.pairs.clone()
    }
}

#[derive(Debug, Clone)]
pub struct LookUpPairs<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for LookUpPairs<'a> {
    type Item = LookUpPair;

    fn next(&mut self) -> Option<Self::Item> {
        Some(LookUpPair {
            key: self.decoder.read().ok()?,
            offset: self.decoder.read().ok()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LookUpPair {
    key: i32,
    offset: i32,
}

impl LookUpPair {
    pub fn key(&self) -> i32 {
        self.key
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }
}

#[derive(Debug, Clone)]
pub struct TableSwitch<'a> {
    default_offset: i32,
    pairs: TablePairs<'a>,
}

impl<'a> TableSwitch<'a> {
    pub fn default_offset(&self) -> i32 {
        self.default_offset
    }

    pub fn pairs(&self) -> TablePairs<'a> {
        self.pairs.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TablePairs<'a> {
    decoder: Decoder<'a>,
    key: i32,
}

impl<'a> Iterator for TablePairs<'a> {
    type Item = TablePair;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.decoder.read().ok()?;
        let key = self.key;
        self.key += 1;
        Some(TablePair { key, offset })
    }
}

#[derive(Debug, Clone)]
pub struct TablePair {
    key: i32,
    offset: i32,
}

impl TablePair {
    pub fn key(&self) -> i32 {
        self.key
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }
}

impl<'a> RawInstruction<'a> {
    fn decode(decoder: &mut Decoder<'a>, method_start: usize) -> Result<Self, DecodeError> {
        use RawInstruction::*;
        let opcode: u8 = decoder.read()?;
        let instruction = match opcode {
            0x32 => AALoad,
            0x53 => AAStore,
            0x01 => AConstNull,
            0x19 => ALoad {
                index: decoder.read()?,
            },
            0x2a => ALoad0,
            0x2b => ALoad1,
            0x2c => ALoad2,
            0x2d => ALoad3,
            0xbd => ANewArray {
                index: decoder.read()?,
            },
            0xb0 => AReturn,
            0xbe => ArrayLength,
            0x3a => AStore {
                index: decoder.read()?,
            },
            0x4b => AStore0,
            0x4c => AStore1,
            0x4d => AStore2,
            0x4e => AStore3,
            0xbf => AThrow,
            0x33 => BALoad,
            0x54 => BAStore,
            0x10 => BIPush {
                value: decoder.read()?,
            },
            0x34 => CALoad,
            0x55 => CAStore,
            0xc0 => CheckCast {
                index: decoder.read()?,
            },
            0x90 => D2F,
            0x8e => D2I,
            0x8f => D2L,
            0x63 => DAdd,
            0x31 => DALoad,
            0x52 => DAStore,
            0x98 => DCmpG,
            0x97 => DCmpL,
            0x0e => DConst0,
            0x0f => DConst1,
            0x6f => DDiv,
            0x18 => DLoad {
                index: decoder.read()?,
            },
            0x26 => DLoad0,
            0x27 => DLoad1,
            0x28 => DLoad2,
            0x29 => DLoad3,
            0x6b => DMul,
            0x77 => DNeg,
            0x73 => DRem,
            0xaf => DReturn,
            0x39 => DStore {
                index: decoder.read()?,
            },
            0x47 => DStore0,
            0x48 => DStore1,
            0x49 => DStore2,
            0x4a => DStore3,
            0x67 => DSub,
            0x59 => Dup,
            0x5a => DupX1,
            0x5b => DupX2,
            0x5c => Dup2,
            0x5d => Dup2X1,
            0x5e => Dup2X2,
            0x8d => F2D,
            0x8b => F2I,
            0x8c => F2L,
            0x62 => FAdd,
            0x30 => FALoad,
            0x51 => FAStore,
            0x96 => FCmpG,
            0x95 => FCmpL,
            0x0b => FConst0,
            0x0c => FConst1,
            0x0d => FConst2,
            0x6e => FDiv,
            0x17 => FLoad {
                index: decoder.read()?,
            },
            0x22 => FLoad0,
            0x23 => FLoad1,
            0x24 => FLoad2,
            0x25 => FLoad3,
            0x6a => FMul,
            0x76 => FNeg,
            0x72 => FRem,
            0xae => FReturn,
            0x38 => FStore {
                index: decoder.read()?,
            },
            0x43 => FStore0,
            0x44 => FStore1,
            0x45 => FStore2,
            0x46 => FStore3,
            0x66 => FSub,
            0xb4 => GetField {
                index: decoder.read()?,
            },
            0xb2 => GetStatic {
                index: decoder.read()?,
            },
            0xa7 => Goto {
                offset: decoder.read()?,
            },
            0xc8 => GotoW {
                offset: decoder.read()?,
            },
            0x91 => I2B,
            0x92 => I2C,
            0x87 => I2D,
            0x86 => I2F,
            0x85 => I2L,
            0x93 => I2S,
            0x60 => IAdd,
            0x2e => IALoad,
            0x7e => IAnd,
            0x4f => IAStore,
            0x02 => IConstM1,
            0x03 => IConst0,
            0x04 => IConst1,
            0x05 => IConst2,
            0x06 => IConst3,
            0x07 => IConst4,
            0x08 => IConst5,
            0x6c => IDiv,
            0xa5 => IfACmpEq {
                offset: decoder.read()?,
            },
            0xa6 => IfACmpNe {
                offset: decoder.read()?,
            },
            0x9f => IfICmpEq {
                offset: decoder.read()?,
            },
            0xa0 => IfICmpNe {
                offset: decoder.read()?,
            },
            0xa1 => IfICmpLt {
                offset: decoder.read()?,
            },
            0xa2 => IfICmpGe {
                offset: decoder.read()?,
            },
            0xa3 => IFICmpGt {
                offset: decoder.read()?,
            },
            0xa4 => IfICmpLe {
                offset: decoder.read()?,
            },
            0x99 => IfEq {
                offset: decoder.read()?,
            },
            0x9a => IfNe {
                offset: decoder.read()?,
            },
            0x9b => IfLt {
                offset: decoder.read()?,
            },
            0x9c => IfGe {
                offset: decoder.read()?,
            },
            0x9d => IfGt {
                offset: decoder.read()?,
            },
            0x9e => IfLe {
                offset: decoder.read()?,
            },
            0xc7 => IfNonNull {
                offset: decoder.read()?,
            },
            0xc6 => IfNull {
                offset: decoder.read()?,
            },
            0x84 => IInc {
                index: decoder.read()?,
                value: decoder.read()?,
            },
            0x15 => ILoad {
                index: decoder.read()?,
            },
            0x1a => ILoad0,
            0x1b => ILoad1,
            0x1c => ILoad2,
            0x1d => ILoad3,
            0x68 => IMul,
            0x74 => INeg,
            0xc1 => InstanceOf {
                index: decoder.read()?,
            },
            0xba => {
                let index = decoder.read()?;
                if decoder.read::<u8>()? != 0 || decoder.read::<u8>()? != 0 {
                    return Err(DecodeError::from_decoder(
                        DecodeErrorKind::InvalidInstruction,
                        decoder,
                    ));
                }
                InvokeDynamic { index }
            }
            0xb9 => {
                let index = decoder.read()?;
                let count = decoder.read()?;
                if decoder.read::<u8>()? != 0 {
                    return Err(DecodeError::from_decoder(
                        DecodeErrorKind::InvalidInstruction,
                        decoder,
                    ));
                }
                InvokeInterface { index, count }
            }
            0xb7 => InvokeSpecial {
                index: decoder.read()?,
            },
            0xb8 => InvokeStatic {
                index: decoder.read()?,
            },
            0xb6 => InvokeVirtual {
                index: decoder.read()?,
            },
            0x80 => IOr,
            0x70 => IRem,
            0xac => IReturn,
            0x78 => IShL,
            0x7a => IShR,
            0x36 => IStore {
                index: decoder.read()?,
            },
            0x3b => IStore0,
            0x3c => IStore1,
            0x3d => IStore2,
            0x3e => IStore3,
            0x64 => ISub,
            0x7c => IUShR,
            0x82 => IXor,
            0xa8 => JSr {
                offset: decoder.read()?,
            },
            0xc9 => JSrW {
                offset: decoder.read()?,
            },
            0x8a => L2D,
            0x89 => L2F,
            0x88 => L2I,
            0x61 => LAdd,
            0x2f => LALoad,
            0x7f => LAnd,
            0x50 => LAStore,
            0x94 => LCmp,
            0x09 => LConst0,
            0x0a => LConst1,
            0x12 => LdC {
                index: cpool::Index::new(decoder.read::<u8>()?.into())?,
            },
            0x13 => LdCW {
                index: decoder.read()?,
            },
            0x14 => LdC2W {
                index: decoder.read()?,
            },
            0x6d => LDiv,
            0x16 => LLoad {
                index: decoder.read()?,
            },
            0x1e => LLoad0,
            0x1f => LLoad1,
            0x20 => LLoad2,
            0x21 => LLoad3,
            0x69 => LMul,
            0x75 => LNeg,
            0xab => {
                // skip padding
                let offset = decoder.file_position() - method_start - 1;
                decoder.advance(3 - (offset & 3))?;

                LookUpSwitch(self::LookUpSwitch {
                    default_offset: decoder.read()?,
                    pairs: decoder.read()?,
                })
            }
            0x81 => LOr,
            0x71 => LRem,
            0xad => LReturn,
            0x79 => LShL,
            0x7b => LShR,
            0x37 => LStore {
                index: decoder.read()?,
            },
            0x3f => LStore0,
            0x40 => LStore1,
            0x41 => LStore2,
            0x42 => LStore3,
            0x65 => LSub,
            0x7d => LUShR,
            0x83 => LXor,
            0xc2 => MonitorEnter,
            0xc3 => MonitorExit,
            0xc5 => MultiANewArray {
                index: decoder.read()?,
                dimensions: decoder.read()?,
            },
            0xbb => New {
                index: decoder.read()?,
            },
            0xbc => NewArray {
                atype: decoder.read()?,
            },
            0x00 => Nop,
            0x57 => Pop,
            0x58 => Pop2,
            0xb5 => PutField {
                index: decoder.read()?,
            },
            0xb3 => PutStatic {
                index: decoder.read()?,
            },
            0xa9 => Ret {
                index: decoder.read()?,
            },
            0xb1 => Return,
            0x35 => SALoad,
            0x56 => SAStore,
            0x11 => SIPush {
                value: decoder.read()?,
            },
            0x5f => Swap,
            0xaa => {
                // skip padding
                let offset = decoder.file_position() - method_start - 1;
                decoder.advance(3 - (offset & 3))?;

                TableSwitch(self::TableSwitch {
                    default_offset: decoder.read()?,
                    pairs: decoder.read()?,
                })
            }
            0xc4 => {
                let opcode: u8 = decoder.read()?;
                match opcode {
                    0x19 => ALoadW {
                        index: decoder.read()?,
                    },
                    0x3a => AStoreW {
                        index: decoder.read()?,
                    },
                    0x18 => DLoadW {
                        index: decoder.read()?,
                    },
                    0x39 => DStoreW {
                        index: decoder.read()?,
                    },
                    0x17 => FLoadW {
                        index: decoder.read()?,
                    },
                    0x38 => FStoreW {
                        index: decoder.read()?,
                    },
                    0x15 => ILoadW {
                        index: decoder.read()?,
                    },
                    0x36 => IStoreW {
                        index: decoder.read()?,
                    },
                    0x16 => LLoadW {
                        index: decoder.read()?,
                    },
                    0x37 => LStoreW {
                        index: decoder.read()?,
                    },
                    0xa9 => RetW {
                        index: decoder.read()?,
                    },
                    0x84 => IIncW {
                        index: decoder.read()?,
                        value: decoder.read()?,
                    },
                    _ => {
                        return Err(DecodeError::from_decoder(
                            DecodeErrorKind::InvalidInstruction,
                            decoder,
                        ))
                    }
                }
            }
            _ => {
                return Err(DecodeError::from_decoder(
                    DecodeErrorKind::InvalidInstruction,
                    decoder,
                ))
            }
        };
        Ok(instruction)
    }
}

impl<'a> Decode<'a> for ArrayType {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        use ArrayType::*;

        let tag: u8 = decoder.read()?;
        match tag {
            4 => Ok(Boolean),
            5 => Ok(Char),
            6 => Ok(Float),
            7 => Ok(Double),
            8 => Ok(Byte),
            9 => Ok(Short),
            10 => Ok(Int),
            11 => Ok(Long),
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidTag,
                decoder,
            )),
        }
    }
}

impl<'a> Decode<'a> for TablePairs<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let low: i32 = decoder.read()?;
        let high: i32 = decoder.read()?;
        let count = (i64::from(high) - i64::from(low) + 1) * 4;
        if count < 0 {
            return Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidInstruction,
                decoder,
            ));
        }
        let count = count as usize;
        let pair_decoder = decoder.limit(count, Context::Code)?;
        decoder.advance(count)?;

        Ok(TablePairs {
            decoder: pair_decoder,
            key: low,
        })
    }
}

impl<'a> Decode<'a> for LookUpPairs<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count = decoder.read::<i32>()?;
        if count < 0 {
            return Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidInstruction,
                decoder,
            ));
        }
        let count = count as usize * 8;
        let pair_decoder = decoder.limit(count, Context::Code)?;
        decoder.advance(count)?;

        Ok(LookUpPairs {
            decoder: pair_decoder,
        })
    }
}
