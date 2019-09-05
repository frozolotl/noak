use crate::error::*;

pub trait Encoder: Sized {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;

    fn write<T: Encode>(&mut self, value: T) -> Result<(), EncodeError> {
        value.encode(self)
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
                    encoder.write(self.to_be_bytes().as_ref())
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
        encoder.write(self.to_bits().to_be_bytes().as_ref())
    }
}

impl Encode for f64 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write(self.to_bits().to_be_bytes().as_ref())
    }
}

#[derive(Copy, Clone)]
pub struct Position(usize);

impl Position {
    pub fn offset(self, bytes: usize) -> Position {
        Position(self.0 + bytes)
    }
}

#[derive(Clone)]
pub struct VecEncoder {
    buf: Vec<u8>,
}

impl VecEncoder {
    pub fn new() -> VecEncoder {
        VecEncoder { buf: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> VecEncoder {
        VecEncoder {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn position(&self) -> Position {
        Position(self.buf.len())
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn inserting(&mut self, at: Position) -> InsertingEncoder {
        InsertingEncoder {
            buf: &mut self.buf,
            cursor: at.0,
        }
    }

    pub fn replacing(&mut self, at: Position) -> ReplacingEncoder {
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
        assert!(bytes.len() < self.buf.len(), "cannot replace bytes which do not exist");
        let (a, b) = std::mem::replace(&mut self.buf, &mut []).split_at_mut(bytes.len());
        a.copy_from_slice(&bytes[..bytes.len()]);
        self.buf = b;
        Ok(())
    }
}

pub struct InsertingEncoder<'a> {
    buf: &'a mut Vec<u8>,
    cursor: usize,
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
