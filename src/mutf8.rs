//! Modified UTF-8 string handling

use crate::error::*;
use std::{
    borrow::{Borrow, ToOwned},
    char,
    fmt::{self, Write},
    ops::Deref,
    str,
};

/// A Modified UTF-8 string slice, like [str].
#[derive(Eq, PartialEq, Hash)]
pub struct MStr {
    inner: [u8],
}

impl MStr {
    /// Creates a new string from a modified UTF-8 byte slice.
    pub fn from_bytes(v: &[u8]) -> Result<&MStr, DecodeError> {
        if is_mutf8_valid(v) {
            Ok(unsafe { MStr::from_mutf8_unchecked(v) })
        } else {
            Err(DecodeError::new(DecodeErrorKind::InvalidMutf8))
        }
    }

    unsafe fn from_mutf8_unchecked(v: &[u8]) -> &MStr {
        &*(v as *const [u8] as *const MStr)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    #[inline]
    pub fn chars<'a>(&'a self) -> impl Iterator<Item = char> + 'a {
        struct Chars<'a> {
            inner: &'a [u8],
            cursor: usize,
        }

        impl<'a> Iterator for Chars<'a> {
            type Item = char;

            fn next(&mut self) -> Option<char> {
                let (size, ch) = unsafe { decode_mutf8_char(&self.inner[self.cursor..]) };
                self.cursor += size;
                Some(ch)
            }
        }

        Chars {
            inner: &self.inner,
            cursor: 0,
        }
    }
}

/// A Modified UTF-8 string, but owned, like [String].
pub struct MString {
    buf: Vec<u8>,
}

impl MString {
    /// Creates an empty string.
    #[inline]
    pub fn new() -> MString {
        MString { buf: Vec::new() }
    }

    /// Creates an empty string with capacity.
    /// The capacity is of type [u16] since a string in bytecode can't be larger than that.
    #[inline]
    pub fn with_capacity(cap: u16) -> MString {
        MString {
            buf: Vec::with_capacity(cap as usize),
        }
    }

    /// Creates a new string from a modified UTF-8 byte vector.
    pub fn from_vec(buf: Vec<u8>) -> Result<MString, DecodeError> {
        if is_mutf8_valid(&buf) {
            Ok(MString { buf })
        } else {
            Err(DecodeError::new(DecodeErrorKind::InvalidMutf8))
        }
    }
}

impl Deref for MString {
    type Target = MStr;

    #[inline]
    fn deref(&self) -> &MStr {
        unsafe { MStr::from_mutf8_unchecked(&self.buf) }
    }
}

impl Borrow<MStr> for MString {
    #[inline]
    fn borrow(&self) -> &MStr {
        self.deref()
    }
}

impl ToOwned for MStr {
    type Owned = MString;

    #[inline]
    fn to_owned(&self) -> MString {
        MString {
            buf: self.inner.to_owned(),
        }
    }
}

impl fmt::Display for MStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut start = 0;
        let mut i = 0;
        while i < self.len() {
            if self.inner[0] < 0x80 {
                i += 1;
            } else {
                if i != start {
                    // safe since everything from start to i is non-zero ascii
                    f.write_str(unsafe { str::from_utf8_unchecked(&self.inner[start..i]) })?;
                }

                let (size, ch) = unsafe { decode_mutf8_char(&self.inner[i..]) };
                i += size;

                start = i;
                f.write_char(ch)?;
            }
        }

        Ok(())
    }
}

fn is_mutf8_valid(v: &[u8]) -> bool {
    /// The amount of bytes a character starting with a specific byte takes.
    #[rustfmt::skip]
    static MUTF8_CHAR_WIDTH: [u8; 256] = [
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 6, 3, 3,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let mut i = 0;
    while i < v.len() {
        let b1 = v[i];
        if b1 == 0 {
            return false;
        }

        if b1 < 0x80 {
            i += 1;
        } else {
            let width = MUTF8_CHAR_WIDTH[b1 as usize];
            if v.len() < i + width as usize {
                return false;
            }
            match width {
                2 => {
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000 {
                        return false;
                    }
                    // overlong encodings which do not encode `0` are not allowed
                    if b1 & 0b0001_1110 == 0 && (b1 != 0b1100_0000 && v[i + 1] != 0b1000_0000) {
                        return false;
                    }
                }
                3 => {
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000
                        || v[i + 2] & 0b1100_0000 != 0b1000_0000
                    {
                        return false;
                    }
                    // overlong encodings are not allowed
                    if b1.trailing_zeros() >= 4 && v[i + 1] & 0b0010_0000 == 0 {
                        return false;
                    }
                }
                6 => {
                    if v[i + 1] & 0b1111_0000 != 0b1010_0000
                        || v[i + 2] & 0b1100_0000 != 0b1000_0000
                        || v[i + 3] != 0b1110_1101
                        || v[i + 4] & 0b1111_0000 != 0b1011_0000
                        || v[i + 5] & 0b1100_0000 != 0b1000_0000
                    {
                        return false;
                    }
                    // overlong encoding...
                    if v[i + 1].trailing_zeros() >= 4 {
                        return false;
                    }
                }
                _ => return false,
            }
            i += width as usize;
        }
    }

    true
}

/// Decodes a character and returns its size.
/// The input bytes **must** be valid modified utf-8
unsafe fn decode_mutf8_char(v: &[u8]) -> (usize, char) {
    if v[0] & 0b1000_0000 == 0b0000_0000 {
        // single byte case
        return (1, v[0] as char);
    }

    if v[0] & 0b1110_0000 == 0b1100_0000 {
        // two byte case
        let c1 = u32::from(v[0] & 0b0001_1111) << 6;
        let c2 = u32::from(v[1] & 0b0011_1111);
        return (2, char::from_u32_unchecked(c1 | c2));
    }

    if v[0] == 0b1110_1101 {
        // six byte case
        let c2 = u32::from(v[1] & 0b0000_1111) << 16;
        let c3 = u32::from(v[2] & 0b0011_1111) << 10;
        let c5 = u32::from(v[4] & 0b0000_1111) << 6;
        let c6 = u32::from(v[5] & 0b0011_1111);
        return (6, char::from_u32_unchecked(0x10000 | c2 | c3 | c5 | c6));
    }

    // three byte case
    let c1 = u32::from(v[0] & 0b0000_1111) << 12;
    let c2 = u32::from(v[1] & 0b0011_1111) << 6;
    let c3 = u32::from(v[2] & 0b0011_1111);
    (3, char::from_u32_unchecked(c1 | c2 | c3))
}
