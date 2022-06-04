use crate::error::*;
use std::fmt;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::ControlFlow;

#[derive(Clone)]
pub struct Decoder<'input> {
    buf: &'input [u8],
    file_position: usize,
    ctx: Context,
}

impl<'input> Decoder<'input> {
    pub fn new(buf: &'input [u8], ctx: Context) -> Decoder<'input> {
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

    pub fn buf(&self) -> &'input [u8] {
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
    pub fn limit(&self, count: usize, ctx: Context) -> Result<Decoder<'input>, DecodeError> {
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
    pub fn with_context(&self, ctx: Context) -> Decoder<'input> {
        Decoder {
            buf: self.buf,
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
    pub fn split_bytes_off(&mut self, count: usize) -> Result<&'input [u8], DecodeError> {
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

    pub fn read<T: Decode<'input>>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }

    pub fn read_into<T: DecodeInto<'input>>(self) -> Result<T, DecodeError> {
        T::decode_into(self)
    }
}

impl<'input> fmt::Debug for Decoder<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

pub trait Decode<'input>: Sized + 'input {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError>;
}

pub trait DecodeInto<'input>: Sized + 'input {
    fn decode_into(decoder: Decoder<'input>) -> Result<Self, DecodeError>;
}

macro_rules! impl_decode {
    ($($t:ty => $len:expr,)*) => {
        $(
            impl<'input> Decode<'input> for $t {
                fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
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

impl<'input> Decode<'input> for f32 {
    fn decode(decoder: &mut Decoder<'input>) -> Result<f32, DecodeError> {
        let bits = decoder.read()?;
        Ok(f32::from_bits(bits))
    }
}

impl<'input> Decode<'input> for f64 {
    fn decode(decoder: &mut Decoder<'input>) -> Result<f64, DecodeError> {
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

impl<'input, R: Decode<'input>> LazyDecodeRef<R> {
    pub fn get(&mut self, decoder: &mut Decoder<'input>) -> Result<&R, DecodeError> {
        match self {
            LazyDecodeRef::NotRead => match decoder.read() {
                Ok(v) => {
                    *self = LazyDecodeRef::Read(v);
                    if let LazyDecodeRef::Read(v) = self {
                        Ok(v)
                    } else {
                        unreachable!();
                    }
                }
                Err(err) => {
                    *self = LazyDecodeRef::Error(err.clone());
                    Err(err)
                }
            },
            LazyDecodeRef::Read(v) => Ok(v),
            LazyDecodeRef::Error(err) => Err(err.clone()),
        }
    }
}

pub struct DecodeManyIter<'input, T, Count> {
    decoder: Decoder<'input>,
    remaining: Count,
    marker: PhantomData<T>,
}

impl<'input, T, Count: 'input> DecodeManyIter<'input, T, Count> {
    pub fn new(decoder: Decoder<'input>, count: Count) -> DecodeManyIter<'input, T, Count> {
        DecodeManyIter {
            decoder,
            remaining: count,
            marker: PhantomData,
        }
    }
}

impl<'input, T, Count> Decode<'input> for DecodeManyIter<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input> + Countdown,
{
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
        let count: Count = decoder.read()?;
        let old_decoder = decoder.clone();

        let mut remaining = count;
        while remaining.decrement().is_continue() {
            decoder.read::<T>()?;
        }

        Ok(DecodeManyIter {
            decoder: old_decoder,
            remaining: count,
            marker: PhantomData,
        })
    }
}

impl<'input, T, Count> DecodeInto<'input> for DecodeManyIter<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input>,
{
    fn decode_into(mut decoder: Decoder<'input>) -> Result<Self, DecodeError> {
        let remaining = decoder.read()?;
        Ok(DecodeManyIter {
            decoder,
            remaining,
            marker: PhantomData,
        })
    }
}

impl<'input, T, Count> Iterator for DecodeManyIter<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input> + Countdown,
{
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining.decrement() {
            ControlFlow::Continue(()) => Some(self.decoder.read()),
            ControlFlow::Break(()) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining.into()))
    }
}

impl<'input, T, Count> FusedIterator for DecodeManyIter<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input> + Countdown,
{
}

impl<'input, T, Count: Countdown> Clone for DecodeManyIter<'input, T, Count> {
    fn clone(&self) -> Self {
        DecodeManyIter {
            decoder: self.decoder.clone(),
            remaining: self.remaining,
            marker: PhantomData,
        }
    }
}

impl<'input, T, Count> fmt::Debug for DecodeManyIter<'input, T, Count>
where
    T: Decode<'input>,
    Count: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecodeManyIter")
            .field("remaining", &self.remaining)
            .finish()
    }
}

pub struct DecodeMany<'input, T, Count> {
    iter: DecodeManyIter<'input, T, Count>,
}

impl<'input, T, Count> DecodeMany<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input> + Countdown,
{
    #[must_use]
    pub fn iter(&self) -> DecodeManyIter<'input, T, Count> {
        self.iter.clone()
    }
}

impl<'input, T, Count> Decode<'input> for DecodeMany<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input> + Countdown,
{
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
        Ok(DecodeMany { iter: decoder.read()? })
    }
}

impl<'input, T, Count> DecodeInto<'input> for DecodeMany<'input, T, Count>
where
    T: Decode<'input>,
    Count: Decode<'input>,
{
    fn decode_into(decoder: Decoder<'input>) -> Result<Self, DecodeError> {
        Ok(DecodeMany {
            iter: decoder.read_into()?,
        })
    }
}

impl<'input, T, Count: Countdown> Clone for DecodeMany<'input, T, Count> {
    fn clone(&self) -> Self {
        DecodeMany {
            iter: self.iter.clone(),
        }
    }
}

impl<'input, T, Count> fmt::Debug for DecodeMany<'input, T, Count> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecodeMany").finish()
    }
}

pub trait Countdown: Copy + Into<usize> {
    /// Decrements the counter and returns whether it can continue.
    fn decrement(&mut self) -> ControlFlow<()>;
}

impl Countdown for u8 {
    fn decrement(&mut self) -> ControlFlow<()> {
        match self.checked_sub(1) {
            Some(i) => {
                *self = i;
                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(()),
        }
    }
}

impl Countdown for u16 {
    fn decrement(&mut self) -> ControlFlow<()> {
        match self.checked_sub(1) {
            Some(i) => {
                *self = i;
                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(()),
        }
    }
}

macro_rules! dec_structure {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident<'input> $($into:ident)? {
            $(
                $(#[doc = $doc_comment:literal])*
                $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        $vis struct $struct_name<'input> {
            $(
                $(#[doc = $doc_comment])*
                $field_name : $field_type,
            )*
            _marker: std::marker::PhantomData<&'input ()>,
        }

        impl<'input> $struct_name<'input> {
            $(
                $(#[doc = $doc_comment])*
                #[must_use]
                $vis fn $field_name(&self) -> $field_type {
                    Clone::clone(&self.$field_name)
                }
            )*
        }

        $crate::reader::decoding::dec_structure!(@decode $($into)? => $struct_name; $($field_name),*);

        impl<'input> std::fmt::Debug for $struct_name<'input> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(std::stringify!($struct_name)).finish()
            }
        }
    };
    (@decode => $struct_name:ident; $($field_name:ident),*) => {
        impl<'input> $crate::reader::decoding::Decode<'input> for $struct_name<'input> {
            fn decode(decoder: &mut $crate::reader::decoding::Decoder<'input>) -> Result<Self, $crate::error::DecodeError> {
                Ok(Self {
                    $($field_name: decoder.read()?,)*
                    _marker: std::marker::PhantomData,
                })
            }
        }
    };
    (@decode into => $struct_name:ident; $($field_name:ident),*) => {
        impl<'input> $crate::reader::decoding::DecodeInto<'input> for $struct_name<'input> {
            fn decode_into(mut decoder: $crate::reader::decoding::Decoder<'input>) -> Result<Self, $crate::error::DecodeError> {
                Ok(Self {
                    $($field_name: decoder.read()?,)*
                    _marker: std::marker::PhantomData,
                })
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use dec_structure;
