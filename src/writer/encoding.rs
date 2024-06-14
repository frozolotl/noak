use crate::error::*;
use std::fmt;
use std::marker::PhantomData;

use super::cpool;

pub trait Encoder: Sized {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;

    fn write<T: Encode>(&mut self, value: T) -> Result<&mut Self, EncodeError> {
        value.encode(self)?;
        Ok(self)
    }
}

pub trait Encode {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError>;
}

impl<T: Encode> Encode for &T {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self).encode(encoder)
    }
}

impl Encode for &[u8] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_bytes(self)
    }
}

macro_rules! impl_encode {
    ($($t:ty,)*) => {
        $(
            impl Encode for $t {
                fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                    encoder.write(self.to_be_bytes().as_ref())?;
                    Ok(())
                }
            }
        )*
    }
}

impl_encode! {
    u8, i8,
    u16, i16,
    u32, i32,
    u64, i64,
    // this will probably never be needed, but why not
    u128, i128,
}

impl Encode for f32 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write(self.to_bits().to_be_bytes().as_ref())?;
        Ok(())
    }
}

impl Encode for f64 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write(self.to_bits().to_be_bytes().as_ref())?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(usize);

impl Offset {
    pub const fn new(position: usize) -> Offset {
        Offset(position)
    }

    pub const fn get(self) -> usize {
        self.0
    }

    pub const fn offset(self, by: usize) -> Offset {
        Offset(self.0 + by)
    }

    pub const fn sub(self, by: Offset) -> Offset {
        Offset(self.0 - by.0)
    }
}

#[derive(Clone, Debug)]
pub struct VecEncoder {
    buf: Vec<u8>,
}

impl VecEncoder {
    pub fn new(buf: Vec<u8>) -> VecEncoder {
        VecEncoder { buf }
    }

    pub fn position(&self) -> Offset {
        Offset::new(self.buf.len())
    }

    pub fn inner(&self) -> &[u8] {
        &self.buf
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn buf(&self) -> &[u8] {
        &self.buf
    }

    pub fn replacing(&mut self, at: Offset) -> ReplacingEncoder<'_> {
        ReplacingEncoder {
            buf: &mut self.buf[at.0..],
        }
    }
}

impl Encoder for VecEncoder {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.buf.extend_from_slice(bytes);
        Ok(())
    }
}

#[derive(Debug)]
pub struct ReplacingEncoder<'a> {
    buf: &'a mut [u8],
}

impl<'a> Encoder for ReplacingEncoder<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        assert!(bytes.len() <= self.buf.len(), "cannot replace bytes which do not exist");
        let (a, b) = std::mem::take(&mut self.buf).split_at_mut(bytes.len());
        a.copy_from_slice(bytes);
        self.buf = b;
        Ok(())
    }
}

/// An encoder writing the amount of bytes written since its creation to the front.
pub(crate) struct LengthWriter<Ctx> {
    /// The offset of the byte counter.
    length_offset: Offset,
    _marker: PhantomData<Ctx>,
}

impl<Ctx: EncoderContext> LengthWriter<Ctx> {
    pub(crate) fn new(context: &mut Ctx) -> Result<Self, EncodeError> {
        let length_offset = context.encoder().position();
        context.encoder().write(0u32)?;
        Ok(LengthWriter {
            length_offset,
            _marker: PhantomData,
        })
    }

    pub(crate) fn finish(self, context: &mut Ctx) -> Result<(), EncodeError> {
        let length = context.encoder().position().sub(self.length_offset).sub(Offset(4)); // subtract the amount of bytes the length takes up
        let length = u32::try_from(length.0)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::None))?;
        context.encoder().replacing(self.length_offset).write(length)?;
        Ok(())
    }
}

impl<E: Encoder> Encoder for &mut E {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (*self).write_bytes(bytes)
    }
}

pub trait InternalEncoderContext {
    fn encoder(&mut self) -> &mut VecEncoder;
}

impl<'a, Ctx: InternalEncoderContext> InternalEncoderContext for &'a mut Ctx {
    fn encoder(&mut self) -> &mut VecEncoder {
        (**self).encoder()
    }
}

pub trait EncoderContext: InternalEncoderContext {
    fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError>;
}

impl<'a, Ctx: EncoderContext> EncoderContext for &'a mut Ctx {
    fn insert_constant<I: Into<cpool::Item>>(&mut self, item: I) -> Result<cpool::Index<I>, EncodeError> {
        (**self).insert_constant(item)
    }
}

pub trait WriteAssembler: Sized {
    type Context: EncoderContext;

    fn new(context: Self::Context) -> Result<Self, EncodeError>;
}

pub trait WriteDisassembler {
    type Context: EncoderContext;

    fn finish(self) -> Result<Self::Context, EncodeError>;
}

pub struct ManyWriter<W: WriteAssembler, Count> {
    /// The offset of the counter starting at the pool end.
    count_offset: Offset,
    context: Option<W::Context>,
    count: Count,
    _marker: PhantomData<W>,
}

impl<W, Count> WriteAssembler for ManyWriter<W, Count>
where
    W: WriteAssembler,
    Count: Encode + Counter,
{
    type Context = W::Context;

    fn new(mut context: Self::Context) -> Result<Self, EncodeError> {
        let count_offset = context.encoder().position();
        let count = Count::zero();
        context.encoder().write(count)?;
        Ok(ManyWriter {
            context: Some(context),
            count_offset,
            count,
            _marker: PhantomData,
        })
    }
}

impl<W, Count> WriteDisassembler for ManyWriter<W, Count>
where
    W: WriteAssembler,
    Count: Encode + Counter,
{
    type Context = W::Context;

    fn finish(mut self) -> Result<Self::Context, EncodeError> {
        self.context
            .take()
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None))
    }
}

impl<W, Count> ManyWriter<W, Count>
where
    W: WriteAssembler,
    Count: Encode + Counter,
{
    pub fn begin<D, F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        D: WriteDisassembler<Context = W::Context>,
        F: FnOnce(W) -> Result<D, EncodeError>,
    {
        let context = self
            .context
            .take()
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None))?;
        self.count.check()?;

        let mut context = f(W::new(context)?)?.finish()?;

        self.count.increment()?;
        context.encoder().replacing(self.count_offset).write(self.count)?;
        self.context = Some(context);
        Ok(self)
    }
}

impl<W: WriteAssembler, Count> fmt::Debug for ManyWriter<W, Count> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManyWriter").finish()
    }
}

pub trait Counter: Copy {
    fn zero() -> Self;
    fn check(self) -> Result<(), EncodeError>;
    fn increment(&mut self) -> Result<(), EncodeError>;
}

macro_rules! impl_counter {
    ($v:ident) => {
        impl Counter for $v {
            fn zero() -> $v {
                0
            }

            fn check(self) -> Result<(), EncodeError> {
                if self == $v::MAX {
                    Err(EncodeError::with_context(
                        EncodeErrorKind::TooManyItems,
                        Context::None,
                    ))
                } else {
                    Ok(())
                }
            }

            fn increment(&mut self) -> Result<(), EncodeError> {
                match self.checked_add(1) {
                    Some(i) => {
                        *self = i;
                        Ok(())
                    }
                    None => Err(EncodeError::with_context(
                        EncodeErrorKind::TooManyItems,
                        Context::None,
                    )),
                }
            }
        }
    };
}

impl_counter!(u8);
impl_counter!(u16);

macro_rules! enc_state {
    ($vis:vis mod $mod:ident : $($state:ident),* $(,)?) => {
        #[allow(non_snake_case)]
        #[doc(hidden)]
        $vis mod $mod {
            pub trait State: sealed::Sealed {}

            $(
                #[derive(Debug)]
                pub struct $state(std::convert::Infallible);
                impl State for $state {}
            )*

            mod sealed {
                pub trait Sealed {}
                $(
                    impl Sealed for super::$state {}
                )*
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use enc_state;
