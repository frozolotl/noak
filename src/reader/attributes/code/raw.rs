use crate::reader::attributes::code;
use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::cpool;

/// An iterator over the instructions and their indices into the code table
#[derive(Debug, Clone)]
pub struct RawInstructions<'a> {
    start_position: usize,
    decoder: Decoder<'a>,
}

impl<'a> Iterator for RawInstructions<'a> {
    type Item = (code::Index, RawInstruction<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(insn) = self.decoder.read() {
            let position = self.decoder.file_position() - self.start_position;
            Some((code::Index::new(position as u32), insn))
        } else {
            None
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
    BIPush,
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
    Return,
    SALoad,
    SAStore,
    SIPush,
    Swap,
    TableSwitch(TableSwitch<'a>),
}

impl<'a> Decode<'a> for RawInstruction<'a> {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let tag: u8 = decoder.read()?;
        match tag {
            _ => Err(DecodeError::from_decoder(
                DecodeErrorKind::InvalidInstruction,
                decoder,
            )),
        }
    }
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
        self.decoder.read().ok()
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
}

impl<'a> Iterator for TablePairs<'a> {
    type Item = TablePair;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
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
