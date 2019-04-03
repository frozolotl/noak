use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    UnexpectedEoi,
}

impl fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeErrorKind::*;

        match *self {
            UnexpectedEoi => write!(f, "unexpected end of input"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    position: usize,
    context: Context,
}

impl DecodeError {
    pub fn kind(&self) -> DecodeErrorKind {
        self.kind
    }

    /// The absolute byte position at which the error occurred.
    pub fn position(&self) -> usize {
        self.position
    }

    pub fn context(&self) -> Context {
        self.context
    }
}

impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {} in {}", self.kind(), self.position(), self.context())
    }
}

/// The context in which a error occurred in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// Either the `0xCAFEBABE` prefix or the major and minor versions.
    Start,
    /// The constant pool along with the index into it.
    /// The index starts at 0.
    ConstantPool(u16),
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Context::*;

        match *self {
            Start => write!(f, "start"),
            ConstantPool(index) => write!(f, "constant pool at {}", index),
        }
    }
}
