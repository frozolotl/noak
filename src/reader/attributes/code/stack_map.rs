use crate::error::*;
use crate::reader::attributes::code;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;
use std::iter::FusedIterator;

#[derive(Clone)]
pub struct StackMapTable<'a> {
    iter: StackMapIter<'a>,
}

impl<'a> StackMapTable<'a> {
    pub fn iter(&self) -> StackMapIter<'a> {
        self.iter.clone()
    }
}

impl<'a> DecodeInto<'a> for StackMapTable<'a> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
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

#[derive(Clone)]
pub struct StackMapIter<'a> {
    decoder: Decoder<'a>,
    remaining: u16,
    current_offset: u32,
}

impl<'a> Iterator for StackMapIter<'a> {
    type Item = Result<(code::Index, StackMapFrame<'a>), DecodeError>;

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

impl<'a> FusedIterator for StackMapIter<'a> {}

impl<'a> fmt::Debug for StackMapIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StackMapIter").finish()
    }
}

#[derive(Debug, Clone)]
pub enum StackMapFrame<'a> {
    Same,
    SameExtended,
    Same1 {
        stack: VerificationType,
    },
    Same1Extended {
        stack: VerificationType,
    },
    Chop {
        to_chop: u8,
    },
    Append {
        locals: VerificationTypeIter<'a>,
    },
    Full {
        locals: VerificationTypeIter<'a>,
        stack: VerificationTypeIter<'a>,
    },
}

fn decode_stack_map_frame<'a>(
    decoder: &mut Decoder<'a>,
    current_offset: u32,
) -> Result<(code::Index, StackMapFrame<'a>), DecodeError> {
    let frame_type: u8 = decoder.read()?;
    if frame_type < 64 {
        let index = code::Index::new(frame_type.into());
        Ok((index, StackMapFrame::Same))
    } else if frame_type == 251 {
        let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
        Ok((index, StackMapFrame::SameExtended))
    } else if frame_type >= 64 && frame_type < 128 {
        let index = code::Index::new(u32::from(frame_type - 64) + current_offset);
        let stack = decode_verification_type(decoder, current_offset)?;
        Ok((index, StackMapFrame::Same1 { stack }))
    } else if frame_type == 247 {
        let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
        let stack = decode_verification_type(decoder, current_offset)?;
        Ok((index, StackMapFrame::Same1 { stack }))
    } else if frame_type >= 248 && frame_type <= 250 {
        let to_chop = (251 - frame_type).into();
        let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
        Ok((index, StackMapFrame::Chop { to_chop }))
    } else if frame_type >= 252 && frame_type <= 254 {
        let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);
        let locals = VerificationTypeIter::new(decoder, (frame_type - 251).into(), current_offset)?;
        Ok((index, StackMapFrame::Append { locals }))
    } else if frame_type == 255 {
        let index = code::Index::new(u32::from(decoder.read::<u16>()?) + current_offset);

        let local_count = decoder.read()?;
        let locals = VerificationTypeIter::new(decoder, local_count, current_offset)?;

        let stack_count = decoder.read()?;
        let stack = VerificationTypeIter::new(decoder, stack_count, current_offset)?;

        Ok((index, StackMapFrame::Full { locals, stack }))
    } else {
        Err(DecodeError::from_decoder(
            DecodeErrorKind::TagReserved,
            decoder,
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VerificationType {
    Top,
    Null,
    UninitializedThis,
    Object(cpool::Index<cpool::Class>),
    UninitializedVariable(code::Index),
    Integer,
    Long,
    Float,
    Double,
}

fn decode_verification_type(
    decoder: &mut Decoder,
    current_offset: u32,
) -> Result<VerificationType, DecodeError> {
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
        _ => Err(DecodeError::from_decoder(
            DecodeErrorKind::InvalidTag,
            decoder,
        )),
    }
}

fn skip_verification_type(decoder: &mut Decoder) -> Result<(), DecodeError> {
    let tag: u8 = decoder.read()?;
    match tag {
        0x07 => decoder.skip::<cpool::Index<cpool::Class>>(),
        0x08 => decoder.skip::<u16>(),
        _ if tag < 0x07 => Ok(()),
        _ => Err(DecodeError::from_decoder(
            DecodeErrorKind::InvalidTag,
            decoder,
        )),
    }
}

#[derive(Clone)]
pub struct VerificationTypeIter<'a> {
    decoder: Decoder<'a>,
    remaining: u16,
    current_offset: u32,
}

impl<'a> VerificationTypeIter<'a> {
    fn new(
        decoder: &mut Decoder<'a>,
        count: u16,
        current_offset: u32,
    ) -> Result<VerificationTypeIter<'a>, DecodeError> {
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

impl<'a> Iterator for VerificationTypeIter<'a> {
    type Item = Result<VerificationType, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            let bytes_remaining = self.decoder.bytes_remaining() as u32;
            let verification_type =
                decode_verification_type(&mut self.decoder, self.current_offset);
            self.current_offset += bytes_remaining - self.decoder.bytes_remaining() as u32;
            Some(verification_type)
        }
    }
}

impl<'a> fmt::Debug for VerificationTypeIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VerificationTypeIter").finish()
    }
}
