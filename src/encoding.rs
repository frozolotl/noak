use crate::error::*;
use std::fmt;
use std::iter::FusedIterator;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Decoder<'a> {
    buf: &'a [u8],
    file_position: usize,
    ctx: Context,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8], ctx: Context) -> Decoder<'a> {
        Decoder {
            buf,
            file_position: 0,
            ctx,
        }
    }

    /// The position inside the file, *not* this decoder.
    pub fn file_position(&self) -> usize {
        self.file_position
    }

    pub fn bytes_remaining(&self) -> usize {
        self.buf.len()
    }

    pub fn buf(&self) -> &'a [u8] {
        self.buf
    }

    pub fn context(&self) -> Context {
        self.ctx
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.ctx = ctx;
    }

    /// Creates a new decoder which is limited to the current location and has the length of `count`.
    /// It will have its own context.
    pub fn limit(&self, count: usize, ctx: Context) -> Result<Decoder<'a>, DecodeError> {
        if count > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::UnexpectedEoi,
                self.file_position,
                self.ctx,
            ))
        } else {
            Ok(Decoder {
                buf: &self.buf[..count],
                file_position: self.file_position,
                ctx,
            })
        }
    }

    /// Creates a new decoder with its own context.
    pub fn with_context(&self, ctx: Context) -> Decoder<'a> {
        Decoder {
            buf: &self.buf,
            file_position: self.file_position,
            ctx,
        }
    }

    /// Advances by a specific number of bytes.
    pub fn advance(&mut self, count: usize) -> Result<(), DecodeError> {
        if count > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::UnexpectedEoi,
                self.file_position,
                self.ctx,
            ))
        } else {
            self.buf = &self.buf[count..];
            self.file_position += count;
            Ok(())
        }
    }

    /// Reads bytes into the buffer supplied and advances.
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        if buf.len() > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::UnexpectedEoi,
                self.file_position,
                self.ctx,
            ))
        } else {
            buf.copy_from_slice(&self.buf[..buf.len()]);
            self.buf = &self.buf[buf.len()..];
            self.file_position += buf.len();
            Ok(())
        }
    }

    /// Advances by `count` and returns `count` bytes.
    pub fn split_bytes_off(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        if count > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::UnexpectedEoi,
                self.file_position,
                self.ctx,
            ))
        } else {
            let v = &self.buf[..count];
            self.buf = &self.buf[count..];
            self.file_position += count;
            Ok(v)
        }
    }

    pub fn read<T: Decode<'a>>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }

    pub fn read_into<T: DecodeInto<'a>>(self) -> Result<T, DecodeError> {
        T::decode_into(self)
    }
}

impl<'a> fmt::Debug for Decoder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

pub trait Decode<'a>: Sized + 'a {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError>;
}

pub trait DecodeInto<'a>: Sized + 'a {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError>;
}

macro_rules! impl_decode {
    ($($t:ty => $len:expr,)*) => {
        $(
            impl<'a> Decode<'a> for $t {
                fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                    let mut buf = <[u8; $len]>::default();
                    decoder.read_bytes(&mut buf)?;
                    Ok(Self::from_be_bytes(buf))
                }
            }
        )*
    }
}

impl_decode! {
    u8 => 1, i8 => 1,
    u16 => 2, i16 => 2,
    u32 => 4, i32 => 4,
    u64 => 8, i64 => 8,
    // this will probably never be needed, but why not
    u128 => 16, i128 => 16,
}

impl<'a> Decode<'a> for f32 {
    fn decode(decoder: &mut Decoder) -> Result<f32, DecodeError> {
        let bits = decoder.read()?;
        Ok(f32::from_bits(bits))
    }
}

impl<'a> Decode<'a> for f64 {
    fn decode(decoder: &mut Decoder) -> Result<f64, DecodeError> {
        let bits = decoder.read()?;
        Ok(f64::from_bits(bits))
    }
}

#[derive(Clone)]
pub enum LazyDecodeRef<R> {
    NotRead,
    Read(R),
    Error(DecodeError),
}

impl<'a, R: Decode<'a>> LazyDecodeRef<R> {
    pub fn get(&mut self, decoder: &mut Decoder<'a>) -> Result<&R, DecodeError> {
        use LazyDecodeRef::*;

        match self {
            NotRead => match decoder.read() {
                Ok(v) => {
                    *self = Read(v);
                    if let Read(v) = self {
                        Ok(v)
                    } else {
                        unreachable!();
                    }
                }
                Err(err) => {
                    *self = Error(err.clone());
                    Err(err)
                }
            },
            Read(v) => Ok(v),
            Error(err) => Err(err.clone()),
        }
    }
}

#[derive(Clone)]
pub struct DecodeIter<'a, T> {
    decoder: Decoder<'a>,
    marker: PhantomData<T>,
}

impl<'a, T> DecodeIter<'a, T> {
    pub fn new(decoder: Decoder<'a>) -> DecodeIter<'a, T> {
        DecodeIter {
            decoder,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Decode<'a>> Iterator for DecodeIter<'a, T> {
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.read() {
            Err(ref err) if err.kind() == DecodeErrorKind::UnexpectedEoi => None,
            res => Some(res),
        }
    }
}

impl<'a, T: Decode<'a>> FusedIterator for DecodeIter<'a, T> {}

impl<'a, T> fmt::Debug for DecodeIter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DecodeIter").finish()
    }
}

pub struct DecodeCounted<'a, T> {
    decoder: Decoder<'a>,
    remaining: u16,
    marker: PhantomData<T>,
}

impl<'a, T> DecodeCounted<'a, T> {
    pub fn new(decoder: Decoder<'a>, count: u16) -> DecodeCounted<'a, T> {
        DecodeCounted {
            decoder,
            remaining: count,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> DecodeInto<'a> for DecodeCounted<'a, T> {
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        let remaining = decoder.read()?;
        Ok(DecodeCounted {
            decoder,
            remaining,
            marker: PhantomData,
        })
    }
}

impl<'a, T: Decode<'a>> Iterator for DecodeCounted<'a, T> {
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            Some(self.decoder.read())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining as usize))
    }

    fn count(self) -> usize {
        self.remaining as usize
    }
}

impl<'a, T: Decode<'a>> FusedIterator for DecodeCounted<'a, T> {}

impl<'a, T> Clone for DecodeCounted<'a, T> {
    fn clone(&self) -> DecodeCounted<'a, T> {
        DecodeCounted {
            decoder: self.decoder.clone(),
            remaining: self.remaining,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Decode<'a>> fmt::Debug for DecodeCounted<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DecodeCounted")
            .field("remaining", &self.remaining)
            .finish()
    }
}

#[derive(Clone)]
pub struct DecodeCountedCopy<'a, T> {
    iter: DecodeCounted<'a, T>,
}

impl<'a, T: 'a> DecodeInto<'a> for DecodeCountedCopy<'a, T> {
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(DecodeCountedCopy {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a, T> DecodeCountedCopy<'a, T> {
    pub fn iter(&self) -> DecodeCounted<'a, T> {
        self.iter.clone()
    }
}

impl<'a, T: 'a> fmt::Debug for DecodeCountedCopy<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DecodeCountedCopy").finish()
    }
}
