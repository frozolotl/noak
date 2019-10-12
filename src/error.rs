use crate::reader::decoding::Decoder;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    UnexpectedEoi,
    InvalidPrefix,
    InvalidMutf8,
    InvalidIndex,
    InvalidLength,
    TagMismatch,
    TagReserved,
    InvalidTag,
    InvalidDescriptor,
    UnknownAttributeName,
    InvalidInstruction,
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
            TagReserved => write!(f, "tag reserved"),
            InvalidTag => write!(f, "invalid tag"),
            InvalidDescriptor => write!(f, "invalid descriptor"),
            UnknownAttributeName => write!(f, "unknown attribute name"),
            InvalidInstruction => write!(f, "invalid instruction"),
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
    pub(crate) fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context: Context::None,
        }
    }

    pub(crate) fn with_context(kind: DecodeErrorKind, context: Context) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context,
        }
    }

    pub(crate) fn with_info(
        kind: DecodeErrorKind,
        position: usize,
        context: Context,
    ) -> DecodeError {
        DecodeError {
            kind,
            position: Some(position),
            context,
        }
    }

    pub(crate) fn from_decoder(kind: DecodeErrorKind, decoder: &Decoder) -> DecodeError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeErrorKind {
    TooManyItems,
    TooManyBytes,
    StringTooLong,
    ValuesMissing,
    CantChangeAnymore,
    ErroredBefore,
}

impl fmt::Display for EncodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EncodeErrorKind::*;

        match *self {
            TooManyItems => write!(f, "too many items"),
            TooManyBytes => write!(f, "too many bytes"),
            StringTooLong => write!(f, "string is too long"),
            ValuesMissing => write!(f, "some values are missing"),
            CantChangeAnymore => write!(f, "can't change some values anymore"),
            ErroredBefore => write!(f, "some error occured in this data structure before"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodeError {
    kind: EncodeErrorKind,
    context: Context,
}

impl EncodeError {
    pub(crate) fn with_context(kind: EncodeErrorKind, context: Context) -> EncodeError {
        EncodeError { kind, context }
    }

    #[inline]
    pub(crate) fn result_from_state<S: Ord>(
        prev: S,
        now: &S,
        context: Context,
    ) -> Result<(), EncodeError> {
        match prev.cmp(now) {
            Ordering::Less => Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                context,
            )),
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(EncodeError::with_context(
                EncodeErrorKind::CantChangeAnymore,
                context,
            )),
        }
    }

    #[inline]
    pub(crate) fn can_write<S: Ord>(
        prev: S,
        now: &S,
        context: Context,
    ) -> Result<bool, EncodeError> {
        match prev.cmp(now) {
            Ordering::Less => Err(EncodeError::with_context(
                EncodeErrorKind::ValuesMissing,
                context,
            )),
            Ordering::Equal => Ok(true),
            Ordering::Greater => Ok(false),
        }
    }

    pub fn kind(&self) -> EncodeErrorKind {
        self.kind
    }

    pub fn context(&self) -> Context {
        self.context
    }
}

impl std::error::Error for EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} in {}", self.kind(), self.context())
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
    ClassInfo,
    Interfaces,
    Fields,
    Methods,
    Attributes,
    AttributeContent,
    Code,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Context::*;

        match *self {
            None => write!(f, "none"),
            Start => write!(f, "start"),
            ConstantPool => write!(f, "constant pool"),
            ClassInfo => write!(f, "class information"),
            Interfaces => write!(f, "interfaces"),
            Fields => write!(f, "fields"),
            Methods => write!(f, "methods"),
            Attributes => write!(f, "attributes"),
            AttributeContent => write!(f, "attribute content"),
            Code => write!(f, "code"),
        }
    }
}
