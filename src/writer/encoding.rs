use crate::error::*;
use crate::writer::class::ClassWriter;
use std::convert::TryFrom;
use std::marker::PhantomData;

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

    pub const fn add(self, by: Offset) -> Offset {
        Offset(self.0 + by.0)
    }

    pub const fn sub(self, by: Offset) -> Offset {
        Offset(self.0 - by.0)
    }
}

#[derive(Clone)]
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

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn buf(&self) -> &[u8] {
        &self.buf
    }

    pub fn inserting(&mut self, at: Offset) -> InsertingEncoder {
        InsertingEncoder {
            buf: &mut self.buf,
            cursor: at.0,
        }
    }

    pub fn replacing(&mut self, at: Offset) -> ReplacingEncoder {
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

pub struct ReplacingEncoder<'a> {
    buf: &'a mut [u8],
}

impl<'a> Encoder for ReplacingEncoder<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        assert!(
            bytes.len() <= self.buf.len(),
            "cannot replace bytes which do not exist"
        );
        let (a, b) = std::mem::replace(&mut self.buf, &mut []).split_at_mut(bytes.len());
        a.copy_from_slice(&bytes);
        self.buf = b;
        Ok(())
    }
}

pub struct InsertingEncoder<'a> {
    buf: &'a mut Vec<u8>,
    cursor: usize,
}

impl<'a> InsertingEncoder<'a> {
    pub fn position(&self) -> Offset {
        Offset::new(self.cursor)
    }
}

impl<'a> Encoder for InsertingEncoder<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let mut v = self.buf.split_off(self.cursor);
        self.buf.extend_from_slice(bytes);
        self.buf.append(&mut v);

        self.cursor += bytes.len();
        Ok(())
    }
}

/// An encoder writing the count of bytes to the front.
pub struct LengthWriter<Ctx> {
    /// The offset of the byte counter starting at the pool end.
    length_offset: Offset,
    marker: PhantomData<Ctx>,
}

impl<Ctx: EncoderContext> LengthWriter<Ctx> {
    pub fn new(context: &mut Ctx) -> Result<Self, EncodeError> {
        let pool_end = context.class_writer().pool_end;
        let encoder = &mut context.class_writer_mut().encoder;
        let length_offset = encoder.position().sub(pool_end);
        encoder.write(0u32)?;
        Ok(LengthWriter {
            length_offset,
            marker: PhantomData,
        })
    }

    pub fn finish(self, context: &mut Ctx) -> Result<(), EncodeError> {
        let length = context
            .class_writer_mut()
            .encoder
            .position()
            .sub(context.class_writer().pool_end)
            .sub(self.length_offset)
            .sub(Offset(4));
        let length = u32::try_from(length.0)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::None))?;
        let position = self.length_offset.add(context.class_writer().pool_end);
        context
            .class_writer_mut()
            .encoder
            .replacing(position)
            .write(length)?;
        Ok(())
    }
}

impl<E: Encoder> Encoder for &mut E {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (*self).write_bytes(bytes)
    }
}

pub trait EncoderContext {
    fn class_writer(&self) -> &ClassWriter;
    fn class_writer_mut(&mut self) -> &mut ClassWriter;
}

impl<'a, Ctx: EncoderContext> EncoderContext for &'a mut Ctx {
    fn class_writer(&self) -> &ClassWriter {
        Ctx::class_writer(self)
    }

    fn class_writer_mut(&mut self) -> &mut ClassWriter {
        Ctx::class_writer_mut(self)
    }
}

pub trait WriteBuilder<'a>: Sized {
    type Context: EncoderContext;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError>;
    fn finish(self) -> Result<&'a mut Self::Context, EncodeError>;
}

pub trait WriteSimple<'a, I>: WriteBuilder<'a> {
    fn write_simple(
        context: &'a mut Self::Context,
        to_write: I,
    ) -> Result<&'a mut Self::Context, EncodeError>;
}

pub struct CountedWriter<'a, W: WriteBuilder<'a>, Count> {
    /// The offset of the counter starting at the pool end.
    count_offset: Offset,
    context: Option<&'a mut W::Context>,
    count: Count,
    marker: PhantomData<W>,
}

impl<'a, W, Count> WriteBuilder<'a> for CountedWriter<'a, W, Count>
where
    W: WriteBuilder<'a> + 'a,
    Count: Encode + Counter,
{
    type Context = W::Context;

    fn new(context: &'a mut Self::Context) -> Result<Self, EncodeError> {
        let count_offset = context
            .class_writer_mut()
            .encoder
            .position()
            .sub(context.class_writer().pool_end);
        let count = Count::zero();
        context.class_writer_mut().encoder.write(&count)?;
        Ok(CountedWriter {
            context: Some(context),
            count_offset,
            count,
            marker: PhantomData,
        })
    }

    fn finish(mut self) -> Result<&'a mut Self::Context, EncodeError> {
        self.context
            .take()
            .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None))
    }
}

impl<'a, W, Count> CountedWriter<'a, W, Count>
where
    W: WriteBuilder<'a> + 'a,
    Count: Encode + Counter,
{
    pub fn write<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&'f mut W) -> Result<(), EncodeError>,
    {
        let context = self.context.take().ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None)
        })?;
        self.count.check()?;

        let mut builder = W::new(context)?;
        f(&mut builder)?;
        let context = builder.finish()?;

        self.count.increment()?;
        let position = self.count_offset.add(context.class_writer().pool_end);
        context
            .class_writer_mut()
            .encoder
            .replacing(position)
            .write(&self.count)?;
        self.context = Some(context);
        Ok(self)
    }

    pub fn write_simple<I>(&mut self, item: I) -> Result<&mut Self, EncodeError>
    where
        W: WriteSimple<'a, I>,
    {
        let context = self.context.take().ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None)
        })?;

        let context = W::write_simple(context, item)?;

        self.count.increment()?;
        let position = self.count_offset.add(context.class_writer().pool_end);
        context
            .class_writer_mut()
            .encoder
            .replacing(position)
            .write(&self.count)?;
        self.context = Some(context);

        Ok(self)
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
                if self == $v::max_value() {
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
