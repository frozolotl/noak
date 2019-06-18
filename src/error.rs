use crate::encoding::Decoder;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    UnexpectedEoi,
    InvalidPrefix,
    InvalidMutf8,
    InvalidIndex,
    InvalidLength,
    TagMismatch,
    InvalidTag,
    InvalidDescriptor,
}

impl fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeErrorKind::*;

        match *self {
            UnexpectedEoi => write!(f, "unexpected end of input"),
            InvalidPrefix => write!(f, "invalid file prefix"),
            InvalidMutf8 => write!(f, "invalid modified utf8"),
            InvalidIndex => write!(f, "invalid index into a table or the constant pool"),
            InvalidLength => write!(f, "invalid length of a table or the constant pool"),
            TagMismatch => write!(f, "tag mismatch"),
            InvalidTag => write!(f, "invalid tag"),
            InvalidDescriptor => write!(f, "invalid descriptor"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    position: Option<usize>,
    context: Context,
}

impl DecodeError {
    pub fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context: Context::None,
        }
    }

    pub fn with_context(kind: DecodeErrorKind, context: Context) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context,
        }
    }

    pub fn with_info(kind: DecodeErrorKind, position: usize, context: Context) -> DecodeError {
        DecodeError {
            kind,
            position: Some(position),
            context,
        }
    }

    pub fn from_decoder(kind: DecodeErrorKind, decoder: &Decoder) -> DecodeError {
        DecodeError {
            kind,
            position: Some(decoder.file_position()),
            context: decoder.context(),
        }
    }

    pub fn kind(&self) -> DecodeErrorKind {
        self.kind
    }

    /// The absolute byte position at which the error occurred.
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    pub fn context(&self) -> Context {
        self.context
    }
}

impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(pos) = self.position() {
            write!(f, "{} at {} in {}", self.kind(), pos, self.context())
        } else {
            write!(f, "{}", self.kind())
        }
    }
}

/// The context in which a error occurred in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// No context.
    None,
    /// Either the `0xCAFEBABE` prefix or the major and minor versions.
    Start,
    ConstantPool,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Context::*;

        match *self {
            None => write!(f, "none"),
            Start => write!(f, "start"),
            ConstantPool => write!(f, "constant pool"),
        }
    }
}
