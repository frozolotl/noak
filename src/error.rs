use crate::reader::decoding::Decoder;
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
    AttributeNotFound,
}

impl fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            AttributeNotFound => write!(f, "attribute was not found"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    position: Option<usize>,
    context: Context,
}

impl DecodeError {
    #[must_use]
    pub(crate) const fn new(kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context: Context::None,
        }
    }

    #[must_use]
    pub(crate) const fn with_context(kind: DecodeErrorKind, context: Context) -> DecodeError {
        DecodeError {
            kind,
            position: None,
            context,
        }
    }

    #[must_use]
    pub(crate) const fn with_info(kind: DecodeErrorKind, position: usize, context: Context) -> DecodeError {
        DecodeError {
            kind,
            position: Some(position),
            context,
        }
    }

    #[must_use]
    pub(crate) fn from_decoder(kind: DecodeErrorKind, decoder: &Decoder<'_>) -> DecodeError {
        DecodeError {
            kind,
            position: Some(decoder.file_position()),
            context: decoder.context(),
        }
    }

    #[must_use]
    pub fn kind(&self) -> DecodeErrorKind {
        self.kind
    }

    /// The absolute byte position at which the error occurred.
    #[must_use]
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    #[must_use]
    pub fn context(&self) -> Context {
        self.context
    }
}

impl Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = self.position() {
            write!(f, "{} at {} in {}", self.kind(), pos, self.context())
        } else {
            write!(f, "{}", self.kind())
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeErrorKind {
    TooManyItems,
    TooManyBytes,
    StringTooLong,
    ValuesMissing,
    CantChangeAnymore,
    ErroredBefore,
    IndexNotFitting,
    LabelNotFound,
    LabelTooFar,
    NegativeOffset,
    IncorrectBounds,
    InvalidKeyOrder,
    Other(Box<dyn Error + 'static>),
}

impl fmt::Display for EncodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EncodeErrorKind::*;

        match self {
            TooManyItems => write!(f, "too many items"),
            TooManyBytes => write!(f, "too many bytes"),
            StringTooLong => write!(f, "string is too long"),
            ValuesMissing => write!(f, "some values are missing"),
            CantChangeAnymore => write!(f, "can't change some values anymore"),
            ErroredBefore => write!(f, "some error occured in this data structure before"),
            IndexNotFitting => write!(f, "the index does not fit into this instruction"),
            LabelNotFound => write!(f, "label was not found"),
            LabelTooFar => write!(f, "label is not close enough to jump point, code start or -- in the case of the stack map table -- the previous label"),
            NegativeOffset => write!(f, "the previous label points to a greater offset than the current label"),
            IncorrectBounds => write!(f, "incorrect bounds (low > high)"),
            InvalidKeyOrder => write!(
                f,
                "the keys in the lookupswitch instruction must be in an increasing numerical order"
            ),
            Other(err) => write!(f, "other: {}", err),
        }
    }
}

#[derive(Debug)]
pub struct EncodeError {
    kind: EncodeErrorKind,
    context: Context,
}

impl EncodeError {
    pub fn from_err<E: Error + 'static>(err: E, context: Context) -> EncodeError {
        EncodeError {
            kind: EncodeErrorKind::Other(Box::new(err)),
            context,
        }
    }

    pub(crate) fn with_context(kind: EncodeErrorKind, context: Context) -> EncodeError {
        EncodeError { kind, context }
    }

    #[must_use]
    pub fn kind(&self) -> &EncodeErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn context(&self) -> Context {
        self.context
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let EncodeErrorKind::Other(err) = self.kind() {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.kind(), self.context())
    }
}

/// The context in which a error occurred in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
