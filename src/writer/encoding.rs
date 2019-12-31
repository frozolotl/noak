use crate::error::*;
use crate::writer::ClassWriter;
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

#[derive(Copy, Clone)]
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
pub struct LengthWriter {
    /// The offset of the byte counter starting at the pool end.
    length_offset: Offset,
}

impl LengthWriter {
    pub fn new(class_writer: &mut ClassWriter) -> Result<Self, EncodeError> {
        let length_offset = class_writer.encoder.position().sub(class_writer.pool_end);
        class_writer.encoder.write(0u32)?;
        Ok(LengthWriter { length_offset })
    }

    pub fn finish(self, class_writer: &mut ClassWriter) -> Result<(), EncodeError> {
        let length = class_writer
            .encoder
            .position()
            .sub(class_writer.pool_end)
            .sub(self.length_offset)
            .sub(Offset(4));
        let length = u32::try_from(length.0)
            .map_err(|_| EncodeError::with_context(EncodeErrorKind::TooManyBytes, Context::None))?;
        class_writer
            .encoder
            .replacing(self.length_offset.add(class_writer.pool_end))
            .write(length)?;
        Ok(())
    }
}

impl<E: Encoder> Encoder for &mut E {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (*self).write_bytes(bytes)
    }
}

pub trait WriteBuilder<'a>: Sized {
    fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError>;
    fn finish(self) -> Result<&'a mut ClassWriter, EncodeError>;
}

pub trait WriteSimple<'a, I>: WriteBuilder<'a> {
    fn write_simple(
        class_writer: &'a mut ClassWriter,
        to_write: I,
    ) -> Result<&'a mut ClassWriter, EncodeError>;
}

pub struct CountedWriter<'a, W, R = u16> {
    /// The offset of the counter starting at the pool end.
    count_offset: Offset,
    class_writer: Option<&'a mut ClassWriter>,
    count: R,
    marker: PhantomData<W>,
}

impl<'a, W, R> CountedWriter<'a, W, R>
where
    R: Encode + Counter,
    W: WriteBuilder<'a>,
{
    pub(crate) fn new(class_writer: &'a mut ClassWriter) -> Result<Self, EncodeError> {
        let count_offset = class_writer.encoder.position().sub(class_writer.pool_end);
        let count = R::zero();
        class_writer.encoder.write(&count)?;
        Ok(CountedWriter {
            class_writer: Some(class_writer),
            count_offset,
            count,
            marker: PhantomData,
        })
    }

    pub fn write<F>(&mut self, f: F) -> Result<&mut Self, EncodeError>
    where
        F: for<'f> FnOnce(&'f mut W) -> Result<(), EncodeError>,
    {
        let class_writer = self.class_writer.take().ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None)
        })?;
        self.count.check()?;

        let mut builder = W::new(class_writer)?;
        f(&mut builder)?;
        let class_writer = builder.finish()?;

        self.count.increment()?;
        class_writer
            .encoder
            .replacing(self.count_offset.add(class_writer.pool_end))
            .write(&self.count)?;
        self.class_writer = Some(class_writer);
        Ok(self)
    }

    pub fn write_simple<I>(&mut self, item: I) -> Result<&mut Self, EncodeError>
    where
        W: WriteSimple<'a, I>,
    {
        let class_writer = self.class_writer.take().ok_or_else(|| {
            EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None)
        })?;

        let class_writer = W::write_simple(class_writer, item)?;

        self.count.increment()?;
        class_writer
            .encoder
            .replacing(self.count_offset.add(class_writer.pool_end))
            .write(&self.count)?;
        self.class_writer = Some(class_writer);

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
