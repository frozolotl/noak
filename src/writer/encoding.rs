use crate::error::*;

pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    pub fn write<T: Encode>(&mut self, value: T) -> Result<(), EncodeError> {
        value.encode(self)
    }
}

pub trait Encode {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError>;
}

impl<T> Encode for &T {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write(self)
    }
}

impl Encode for &[u8] {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.buf.extend_from_slice(self);
        Ok(())
    }
}

macro_rules! impl_encode {
    ($($t:ty,)*) => {
        $(
            impl Encode for $t {
                fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
                    encoder.write(&self.to_be_bytes())
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
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write(&self.to_bits().to_be_bytes())
    }
}

impl Encode for f64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write(&self.to_bits().to_be_bytes())
    }
}
