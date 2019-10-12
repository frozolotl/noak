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

    pub fn skip<T: Decode<'a>>(&mut self) -> Result<(), DecodeError> {
        T::skip(self)
    }
}

impl<'a> fmt::Debug for Decoder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

pub trait Decode<'a>: Sized + 'a {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError>;

    fn skip(decoder: &mut Decoder<'a>) -> Result<(), DecodeError> {
        Self::decode(decoder)?;
        Ok(())
    }
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

                fn skip(decoder: &mut Decoder) -> Result<(), DecodeError> {
                    decoder.advance($len)
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

    fn skip(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.advance(4)
    }
}

impl<'a> Decode<'a> for f64 {
    fn decode(decoder: &mut Decoder) -> Result<f64, DecodeError> {
        let bits = decoder.read()?;
        Ok(f64::from_bits(bits))
    }

    fn skip(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.advance(8)
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

pub struct DecodeCounted<'a, T, R = u16> {
    decoder: Decoder<'a>,
    remaining: R,
    marker: PhantomData<T>,
}

impl<'a, T, R: 'a> DecodeCounted<'a, T, R> {
    pub fn new(decoder: Decoder<'a>, count: R) -> DecodeCounted<'a, T, R> {
        DecodeCounted {
            decoder,
            remaining: count,
            marker: PhantomData,
        }
    }
}

impl<'a, T, R> Decode<'a> for DecodeCounted<'a, T, R>
where
    T: Decode<'a>,
    R: Decode<'a> + Countdown,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        let count: R = decoder.read()?;
        let old_decoder = decoder.clone();

        let mut remaining = count;
        while let CountState::Continue = remaining.decrement() {
            decoder.skip::<T>()?;
        }

        Ok(DecodeCounted {
            decoder: old_decoder,
            remaining: count,
            marker: PhantomData,
        })
    }
}

impl<'a, T, R> DecodeInto<'a> for DecodeCounted<'a, T, R>
where
    T: Decode<'a>,
    R: Decode<'a>,
{
    fn decode_into(mut decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        let remaining = decoder.read()?;
        Ok(DecodeCounted {
            decoder,
            remaining,
            marker: PhantomData,
        })
    }
}

impl<'a, T, R> Iterator for DecodeCounted<'a, T, R>
where
    T: Decode<'a>,
    R: Decode<'a> + Countdown,
{
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining.decrement() {
            CountState::Continue => Some(self.decoder.read()),
            CountState::Break => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining.into()))
    }
}

impl<'a, T: Decode<'a>> FusedIterator for DecodeCounted<'a, T> {}

impl<'a, T, R: Countdown> Clone for DecodeCounted<'a, T, R> {
    fn clone(&self) -> Self {
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

pub struct DecodeCountedCopy<'a, T, R = u16> {
    iter: DecodeCounted<'a, T, R>,
}

impl<'a, T, R: Countdown> DecodeCountedCopy<'a, T, R> {
    pub fn iter(&self) -> DecodeCounted<'a, T, R> {
        self.iter.clone()
    }
}

impl<'a, T, R> Decode<'a> for DecodeCountedCopy<'a, T, R>
where
    T: Decode<'a>,
    R: Decode<'a> + Countdown,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(DecodeCountedCopy {
            iter: decoder.read()?,
        })
    }
}

impl<'a, T, R> DecodeInto<'a> for DecodeCountedCopy<'a, T, R>
where
    T: Decode<'a>,
    R: Decode<'a>,
{
    fn decode_into(decoder: Decoder<'a>) -> Result<Self, DecodeError> {
        Ok(DecodeCountedCopy {
            iter: decoder.read_into()?,
        })
    }
}

impl<'a, T, R: Countdown> Clone for DecodeCountedCopy<'a, T, R> {
    fn clone(&self) -> Self {
        DecodeCountedCopy {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T, R> fmt::Debug for DecodeCountedCopy<'a, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DecodeCountedCopy").finish()
    }
}

pub trait Countdown: Copy + Into<usize> {
    /// Decrements the counter and returns whether it can continue.
    fn decrement(&mut self) -> CountState;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CountState {
    Continue,
    Break,
}

impl Countdown for u8 {
    fn decrement(&mut self) -> CountState {
        match self.checked_sub(1) {
            Some(i) => {
                *self = i;
                CountState::Continue
            }
            None => CountState::Break,
        }
    }
}

impl Countdown for u16 {
    fn decrement(&mut self) -> CountState {
        match self.checked_sub(1) {
            Some(i) => {
                *self = i;
                CountState::Continue
            }
            None => CountState::Break,
        }
    }
}
