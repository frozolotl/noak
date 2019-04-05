use crate::error::{Context, DecodeError, DecodeErrorKind};

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

    /// Advances by a specific number of bytes.
    pub fn advance(&mut self, count: usize) -> Result<(), DecodeError> {
        if count > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::UnexpectedEoi,
                self.file_position,
                self.ctx,
            ))
        } else {
            self.buf = &self.buf[..count];
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

    pub fn read<T: Decode>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }
}

pub trait Decode: Sized {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}

macro_rules! impl_decode {
    ($($t:ty => $buf:expr,)*) => {
        $(
            impl Decode for $t {
                fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                    let mut buf = $buf;
                    decoder.read_bytes(&mut buf)?;
                    Ok(Self::from_be_bytes(buf))
                }
            }
        )*
    }
}

impl_decode! {
    u8 => [0], i8 => [0],
    u16 => [0, 0], i16 => [0, 0],
    u32 => [0, 0, 0, 0], i32 => [0, 0, 0, 0],
    u64 => [0, 0, 0, 0, 0, 0, 0, 0], i64 => [0, 0, 0, 0, 0, 0, 0, 0],
    // this will probably never be needed, but why not
    u128 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], i128 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
}
