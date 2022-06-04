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
pub struct Code<'input> {
    max_stack: u16,
    max_locals: u16,
    raw_instructions: RawInstructions<'input>,
    exception_handlers: ExceptionHandlers<'input>,
    attributes: AttributeIter<'input>,
}

impl<'input> Code<'input> {
    #[must_use]
    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    #[must_use]
    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }

    #[must_use]
    pub fn raw_instructions(&self) -> RawInstructions<'input> {
        self.raw_instructions.clone()
    }

    pub fn raw_instructions_from(&self, index: Index) -> Result<RawInstructions<'input>, DecodeError> {
        let mut instructions = self.raw_instructions();
        instructions.decoder.advance(index.as_u32() as usize)?;
        Ok(instructions)
    }

    #[must_use]
    pub fn exception_handlers(&self) -> ExceptionHandlers<'input> {
        self.exception_handlers.clone()
    }

    #[must_use]
    pub fn attributes(&self) -> AttributeIter<'input> {
        self.attributes.clone()
    }
}

impl<'input> DecodeInto<'input> for Code<'input> {
    fn decode_into(mut decoder: Decoder<'input>) -> Result<Self, DecodeError> {
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

impl<'input> fmt::Debug for Code<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Code").finish()
    }
}

#[derive(Clone)]
pub struct ExceptionHandlers<'input> {
    decoder: Decoder<'input>,
}

impl<'input> Iterator for ExceptionHandlers<'input> {
    type Item = ExceptionHandler<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }
}

impl<'input> fmt::Debug for ExceptionHandlers<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExceptionHandlers").finish()
    }
}

pub struct ExceptionHandler<'input> {
    start: Index,
    end: Index,
    handler: Index,
    catch_type: Option<cpool::Index<cpool::Class<'input>>>,
}

impl<'input> ExceptionHandler<'input> {
    #[must_use]
    pub fn start(&self) -> Index {
        self.start
    }

    #[must_use]
    pub fn end(&self) -> Index {
        self.end
    }

    #[must_use]
    pub fn handler(&self) -> Index {
        self.handler
    }

    #[must_use]
    pub fn catch_type(&self) -> Option<cpool::Index<cpool::Class<'input>>> {
        self.catch_type
    }
}

impl<'input> Decode<'input> for ExceptionHandler<'input> {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
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

impl<'input> fmt::Debug for ExceptionHandler<'input> {
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
    #[must_use]
    pub(crate) fn new(index: u32) -> Index {
        Index { index }
    }

    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.index
    }
}

impl<'input> Decode<'input> for Index {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Index, DecodeError> {
        Ok(Index::new(decoder.read()?))
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("code::Index").field(&self.index).finish()
    }
}

pub type LineNumberTable<'input> = DecodeMany<'input, Line<'input>, u16>;
pub type LineNumberIter<'input> = DecodeManyIter<'input, Line<'input>, u16>;

dec_structure! {
    pub struct Line<'input> {
        start: Index,
        line_number: u16,
    }
}
