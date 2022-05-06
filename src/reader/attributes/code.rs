mod instructions;
mod locals;
mod stack_map;

pub use instructions::*;
pub use locals::*;
pub use stack_map::*;

use crate::error::*;
use crate::reader::decoding::*;
use crate::reader::{cpool, AttributeIter};
use std::fmt;

#[derive(Clone)]
pub struct Code<'a> {
    max_stack: u16,
    max_locals: u16,
    raw_instructions: RawInstructions<'a>,
    exception_handlers: ExceptionHandlers<'a>,
    attributes: AttributeIter<'a>,
}

impl<'a> Code<'a> {
    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }

    pub fn raw_instructions(&self) -> RawInstructions<'a> {
        self.raw_instructions.clone()
    }

    pub fn raw_instructions_from(&self, index: Index) -> Result<RawInstructions<'a>, DecodeError> {
        let mut instructions = self.raw_instructions();
        instructions.decoder.advance(index.as_u32() as usize)?;
        Ok(instructions)
    }

    pub fn exception_handlers(&self) -> ExceptionHandlers<'a> {
        self.exception_handlers.clone()
    }

    pub fn attributes(&self) -> AttributeIter<'a> {
        self.attributes.clone()
    }
}

impl<'a> DecodeInto<'a> for Code<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        let max_stack = decoder.read()?;
        let max_locals = decoder.read()?;

        let code_length = decoder.read::<u32>()? as usize;
        let raw_instructions = RawInstructions {
            start_position: decoder.file_position(),
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
            raw_instructions,
            exception_handlers,
            attributes,
        })
    }
}

impl<'a> fmt::Debug for Code<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Code").finish()
    }
}

#[derive(Clone)]
pub struct ExceptionHandlers<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Iterator for ExceptionHandlers<'a> {
    type Item = ExceptionHandler;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'a> fmt::Debug for ExceptionHandlers<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExceptionHandlers").finish()
    }
}

pub struct ExceptionHandler {
    start: Index,
    end: Index,
    handler: Index,
    catch_type: Option<cpool::Index<cpool::Class>>,
}

impl ExceptionHandler {
    pub fn start(&self) -> Index {
        self.start
    }

    pub fn end(&self) -> Index {
        self.end
    }

    pub fn handler(&self) -> Index {
        self.handler
    }

    pub fn catch_type(&self) -> Option<cpool::Index<cpool::Class>> {
        self.catch_type
    }
}

impl<'a> Decode<'a> for ExceptionHandler {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let start: u16 = decoder.read()?;
        let end: u16 = decoder.read()?;
        let handler: u16 = decoder.read()?;
        let catch_type = decoder.read()?;

        Ok(ExceptionHandler {
            start: Index::new(start.into()),
            end: Index::new(end.into()),
            handler: Index::new(handler.into()),
            catch_type,
        })
    }
}

impl fmt::Debug for ExceptionHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExceptionHandler").finish()
    }
}

/// A 0-based index into the code table.
#[derive(Copy, Clone, PartialEq)]
pub struct Index {
    index: u32,
}

impl Index {
    pub(crate) fn new(index: u32) -> Index {
        Index { index }
    }

    pub fn as_u32(self) -> u32 {
        self.index
    }
}

impl<'a> Decode<'a> for Index {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Index, DecodeError> {
        Ok(Index::new(decoder.read()?))
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code::Index({})", self.index)
    }
}

pub type LineNumberTable<'a> = DecodeCountedCopy<'a, Line<'a>, u16>;
pub type LineNumberIter<'a> = DecodeCounted<'a, Line<'a>, u16>;

dec_structure! {
    pub struct Line<'a> {
        start: Index,
        line_number: u16,
    }
}
