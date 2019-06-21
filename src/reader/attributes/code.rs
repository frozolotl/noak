use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::reader::{cpool, attributes, Attributes};
use std::fmt;
use std::ops::Range;

pub struct Code<'a> {
    max_stack: u16,
    max_locals: u16,
    instructions: Instructions<'a>,
    exception_handlers: ExceptionHandlers<'a>,
    attributes: Attributes<'a>,
}

impl<'a> Code<'a> {
    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }
}

impl<'a> Decode<'a> for Code<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let max_stack = decoder.read()?;
        let max_locals = decoder.read()?;

        let code_length = decoder.read::<u32>()? as usize;
        let instructions = Instructions {
            decoder: decoder.limit(code_length, Context::Code)?,
        };
        decoder.advance(code_length)?;

        let exception_count: u16 = decoder.read()?;
        let exception_table_bytes = exception_count as usize * 8;
        let exception_handlers = ExceptionHandlers {
            decoder: decoder.limit(exception_table_bytes, Context::Code)?,
        };
        decoder.advance(exception_table_bytes)?;

        let attributes = decoder.read()?;
        Ok(Code {
            max_stack,
            max_locals,
            instructions,
            exception_handlers,
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct Instructions<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for Instructions<'a> {
    type Item = Instruction<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

#[derive(Debug)]
pub enum Instruction<'a> {
    __IgnoreMePlease(std::marker::PhantomData<&'a ()>),
}

impl<'a> Decode<'a> for Instruction<'a> {
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

#[derive(Debug)]
pub struct ExceptionHandlers<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ExceptionHandlers<'a> {
    type Item = ExceptionHandler;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

#[derive(Debug)]
pub struct ExceptionHandler {
    start: Index,
    end: Index,
    handler: Index,
    catch_type: cpool::Index<cpool::Class>,
}

impl ExceptionHandler {
    pub fn range(&self) -> Range<Index> {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    pub fn handler(&self) -> Index {
        self.handler
    }

    pub fn catch_type(&self) -> cpool::Index<cpool::Class> {
        self.catch_type
    }
}

impl<'a> Decode<'a> for ExceptionHandler {
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

/// A 0-based index into the code table.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Index {
    index: u32,
}

impl Index {
    fn new(index: u32) -> Index {
        Index { index }
    }

    fn as_u32(self) -> u32 {
        self.index
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "={}", self.index)
    }
}

impl<'a> Decode<'a> for Index {
    fn decode(decoder: &mut Decoder) -> Result<Index, DecodeError> {
        Ok(Index::new(decoder.read()?))
    }
}
