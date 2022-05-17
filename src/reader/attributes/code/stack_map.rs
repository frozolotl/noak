use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct StackMapTable<'input> {
    iter: StackMapIter<'input>,
}

impl<'input> StackMapTable<'input> {
    pub fn iter(&self) -> StackMapIter<'input> {
        self.iter.clone()
    }
}

impl<'input> DecodeInto<'input> for StackMapTable<'input> {
    fn decode_into(mut decoder: Decoder<'input>) -> Result<Self, DecodeError> {
        let count = decoder.read()?;
        Ok(StackMapTable {
            iter: StackMapIter {
                decoder,
                remaining: count,
                current_offset: 0,
            },
        })
    }
}

impl<'input> fmt::Debug for StackMapTable<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackMapTable").finish()
    }
}

#[derive(Clone)]
pub struct StackMapIter<'input> {
    decoder: Decoder<'input>,
    remaining: u16,
    current_offset: u32,
}

impl<'input> Iterator for StackMapIter<'input> {
    type Item = Result<(code::Index, StackMapFrame<'input>), DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            let bytes_remaining = self.decoder.bytes_remaining() as u32;
            let stack_map_frame = decode_stack_map_frame(&mut self.decoder, self.current_offset);
            self.current_offset += bytes_remaining - self.decoder.bytes_remaining() as u32;
            Some(stack_map_frame)
        }
    }
}

impl<'input> FusedIterator for StackMapIter<'input> {}

impl<'input> fmt::Debug for StackMapIter<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackMapIter").finish()
    }
}

#[derive(Debug, Clone)]
pub enum StackMapFrame<'input> {
    Same,
    SameExtended,
    Same1 {
        stack: VerificationType<'input>,
    },
    Same1Extended {
        stack: VerificationType<'input>,
    },
    Chop {
        to_chop: u8,
    },
    Append {
        locals: VerificationTypeIter<'input>,
    },
    Full {
        locals: VerificationTypeIter<'input>,
        stack: VerificationTypeIter<'input>,
    },
}

fn decode_stack_map_frame<'input>(
    decoder: &mut Decoder<'input>,
    current_offset: u32,
) -> Result<(code::Index, StackMapFrame<'input>), DecodeError> {
    let frame_type: u8 = decoder.read()?;
    match frame_type {
        0..=63 => {
            let index = code::Index::new(frame_type.into());
            Ok((index, StackMapFrame::Same))
        }
        64..=127 => {
            let index = code::Index::new(u32::from(frame_type - 64) + current_offset);
            let stack = decode_verification_type(decoder, current_offset)?;
            Ok((index, StackMapFrame::Same1 { stack }))
        }
        247 => {
            let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
            let stack = decode_verification_type(decoder, current_offset)?;
            Ok((index, StackMapFrame::Same1 { stack }))
        }
        248..=250 => {
            let to_chop = 251 - frame_type;
            let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
            Ok((index, StackMapFrame::Chop { to_chop }))
        }
        251 => {
            let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
            Ok((index, StackMapFrame::SameExtended))
        }
        252..=254 => {
            let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
            let locals = VerificationTypeIter::new(decoder, (frame_type - 251).into(), current_offset)?;
            Ok((index, StackMapFrame::Append { locals }))
        }
        255 => {
            let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);

            let local_count = decoder.read()?;
            let locals = VerificationTypeIter::new(decoder, local_count, current_offset)?;

            let stack_count = decoder.read()?;
            let stack = VerificationTypeIter::new(decoder, stack_count, current_offset)?;

            Ok((index, StackMapFrame::Full { locals, stack }))
        }
        _ => Err(DecodeError::from_decoder(DecodeErrorKind::TagReserved, decoder)),
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VerificationType<'input> {
    Top,
    Null,
    UninitializedThis,
    Object(cpool::Index<cpool::Class<'input>>),
    UninitializedVariable(code::Index),
    Integer,
    Long,
    Float,
    Double,
}

fn decode_verification_type<'input>(
    decoder: &mut Decoder<'input>,
    current_offset: u32,
) -> Result<VerificationType<'input>, DecodeError> {
    let tag: u8 = decoder.read()?;
    match tag {
        0x00 => Ok(VerificationType::Top),
        0x01 => Ok(VerificationType::Integer),
        0x02 => Ok(VerificationType::Float),
        0x03 => Ok(VerificationType::Double),
        0x04 => Ok(VerificationType::Long),
        0x05 => Ok(VerificationType::Null),
        0x06 => Ok(VerificationType::UninitializedThis),
        0x07 => Ok(VerificationType::Object(decoder.read()?)),
        0x08 => {
            let index = code::Index::new(current_offset + u32::from(decoder.read::<u16>()?));
            Ok(VerificationType::UninitializedVariable(index))
        }
        _ => Err(DecodeError::from_decoder(DecodeErrorKind::InvalidTag, decoder)),
    }
}

fn skip_verification_type<'input>(decoder: &mut Decoder<'input>) -> Result<(), DecodeError> {
    let tag: u8 = decoder.read()?;
    match tag {
        0x07 => {
            decoder.read::<cpool::Index<cpool::Class<'input>>>()?;
            Ok(())
        }
        0x08 => {
            decoder.read::<u16>()?;
            Ok(())
        }
        _ if tag < 0x07 => Ok(()),
        _ => Err(DecodeError::from_decoder(DecodeErrorKind::InvalidTag, decoder)),
    }
}

#[derive(Clone)]
pub struct VerificationTypeIter<'input> {
    decoder: Decoder<'input>,
    remaining: u16,
    current_offset: u32,
}

impl<'input> VerificationTypeIter<'input> {
    fn new(
        decoder: &mut Decoder<'input>,
        count: u16,
        current_offset: u32,
    ) -> Result<VerificationTypeIter<'input>, DecodeError> {
        let old_decoder = decoder.clone();
        for _ in 0..count {
            skip_verification_type(decoder)?;
        }
        Ok(VerificationTypeIter {
            decoder: old_decoder,
            remaining: count,
            current_offset,
        })
    }
}

impl<'input> Iterator for VerificationTypeIter<'input> {
    type Item = Result<VerificationType<'input>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            let bytes_remaining = self.decoder.bytes_remaining() as u32;
            let verification_type = decode_verification_type(&mut self.decoder, self.current_offset);
            self.current_offset += bytes_remaining - self.decoder.bytes_remaining() as u32;
            Some(verification_type)
        }
    }
}

impl<'input> fmt::Debug for VerificationTypeIter<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationTypeIter").finish()
    }
}
