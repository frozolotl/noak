//! Modified UTF-8 string handling

use crate::error::*;
use std::{
    borrow::{Borrow, ToOwned},
    char,
    fmt::{self, Write},
    iter::FromIterator,
    ops::{self, Deref},
    str,
};

/// A Modified UTF-8 string slice, like [str].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
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
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    #[inline]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.len() {
            true
        } else {
            match self.as_bytes().get(index) {
                None => false,
                Some(&b) => b & 0b1100_0000 != 0b1000_0000 && b != 0b1110_1101,
            }
        }
    }

    #[inline]
    pub fn chars<'a>(&'a self) -> Chars<'a> {
        Chars {
            inner: &self.inner,
        }
    }
}

pub struct Chars<'a> {
    inner: &'a [u8],
}

impl<'a> Chars<'a> {
    pub fn as_mstr(&self) -> &'a MStr {
        // safe because the underlying buffer is guaranteed to be valid
        unsafe {
            MStr::from_mutf8_unchecked(&self.inner)
        }
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.inner.is_empty() {
            None
        } else {
            let (size, ch) = unsafe { decode_mutf8_char(&self.inner) };
            self.inner = &self.inner[size..];
            Some(ch)
        }
    }
}

impl ops::Index<ops::RangeFull> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, _: ops::RangeFull) -> &MStr {
        self
    }
}

impl ops::Index<ops::Range<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::Range<usize>) -> &MStr {
        if index.start <= index.end
            && self.is_char_boundary(index.start)
            && self.is_char_boundary(index.end)
        {
            unsafe { MStr::from_mutf8_unchecked(&self.inner.get_unchecked(index)) }
        } else {
            panic!("MUtf8 index out of bounds");
        }
    }
}

impl ops::Index<ops::RangeInclusive<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeInclusive<usize>) -> &MStr {
        if *index.end() == usize::max_value() {
            panic!("cannot index mutf8 to maximum integer")
        } else {
            &self[*index.start()..*index.end() + 1]
        }
    }
}

impl ops::Index<ops::RangeTo<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &MStr {
        if self.is_char_boundary(index.end) {
            unsafe { MStr::from_mutf8_unchecked(&self.inner.get_unchecked(index)) }
        } else {
            panic!("MUtf8 index out of bounds");
        }
    }
}

impl ops::Index<ops::RangeToInclusive<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeToInclusive<usize>) -> &MStr {
        if index.end == usize::max_value() {
            panic!("cannot index mutf8 to maximum integer")
        } else {
            &self[..index.end + 1]
        }
    }
}

impl ops::Index<ops::RangeFrom<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &MStr {
        if self.is_char_boundary(index.start) {
            unsafe { MStr::from_mutf8_unchecked(&self.inner.get_unchecked(index)) }
        } else {
            panic!("MUtf8 index out of bounds");
        }
    }
}

/// A Modified UTF-8 string, but owned, like [String].
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[inline]
    pub fn with_capacity(cap: usize) -> MString {
        MString {
            buf: Vec::with_capacity(cap),
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

    /// Pushes a character to the string.
    /// It might cause a reallocation.
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 6];
        let size = encode_char(ch, &mut buf);
        self.buf.extend_from_slice(&buf[..size]);
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
            if self.inner[i] < 0x80 {
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

        if i != start {
            // safe since everything from start to i is non-zero ascii
            f.write_str(unsafe { str::from_utf8_unchecked(&self.inner[start..i]) })?;
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
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
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
                    // width = 6
                    if b1 == 0b1110_1101 && v[i + 1] & 0b1111_0000 != 0b1001_0000 {
                        if v.len() - i < 6
                            || v[i + 1] & 0b1111_0000 != 0b1010_0000
                            || v[i + 2] & 0b1100_0000 != 0b1000_0000
                            || v[i + 3] != 0b1110_1101
                            || v[i + 4] & 0b1111_0000 != 0b1011_0000
                            || v[i + 5] & 0b1100_0000 != 0b1000_0000
                        {
                            return false;
                        }
                        // overlong encodings are not allowed
                        if v[i + 1].trailing_zeros() >= 4 {
                            return false;
                        }
                    } else {
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

    if v[0] == 0b1110_1101
        && v[1] & 0b1111_0000 == 0b1010_0000
        && v.len() >= 6
        && v[3] == 0b1110_1101
        && v[4] & 0b1111_0000 == 0b1011_0000
    {
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

impl From<&str> for MString {
    fn from(s: &str) -> MString {
        let mut buf = MString::with_capacity(s.len());
        buf.extend(s.chars());
        buf
    }
}

impl FromIterator<char> for MString {
    fn from_iter<I>(iter: I) -> MString
    where
        I: IntoIterator<Item = char>,
    {
        let mut buf = MString::new();
        buf.extend(iter);
        buf
    }
}

impl Extend<char> for MString {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.buf.reserve(lower_bound);
        for ch in iter {
            self.push(ch);
        }
    }
}

/// Encodes a char to a modified UTF-8 buffer and returns its size.
/// The buffer must at least be of the length which the char will take up.
fn encode_char(ch: char, buf: &mut [u8]) -> usize {
    let ch = ch as u32;
    match ch {
        0x01..=0x7F => {
            buf[0] = ch as u8;
            1
        }
        0 | 0x80..=0x7FF => {
            buf[0] = (0b1110_0000 | (ch >> 6)) as u8;
            buf[1] = (0b1100_0000 | (ch & 0b0011_1111)) as u8;
            2
        }
        0x800..=0xFFFF => {
            buf[0] = (0b1111_0000 | (ch >> 12)) as u8;
            buf[1] = (0b1100_0000 | ((ch >> 6) & 0b0011_1111)) as u8;
            buf[2] = (0b1100_0000 | (ch & 0b0011_1111)) as u8;
            3
        }
        _ => {
            buf[0] = 0b1110_1101;
            buf[1] = (0b1010_0000 | ((ch >> 16) & 0b0000_1111)) as u8;
            buf[2] = (0b1100_0000 | ((ch >> 10) & 0b0011_1111)) as u8;
            buf[3] = 0b1110_1101;
            buf[4] = (0b1011_0000 | ((ch >> 6) & 0b0000_1111)) as u8;
            buf[5] = (0b1100_0000 | (ch & 0b0011_1111)) as u8;
            6
        }
    }
}
