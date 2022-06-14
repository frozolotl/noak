//! Modified UTF-8 string handling.

use crate::error::*;
use std::{
    borrow::{Borrow, BorrowMut, Cow, ToOwned},
    char,
    cmp::Ordering,
    fmt::{self, Write},
    iter::{DoubleEndedIterator, FromIterator},
    mem::size_of,
    ops::{self, Deref, DerefMut},
    str,
};

fn is_mutf8_valid(v: &[u8]) -> bool {
    /// False on all common systems, but who knows, maybe someone runs noak on a 128 bit system.
    const CONSTANTS_LARGE_ENOUGH: bool = size_of::<u64>() >= size_of::<usize>();

    macro_rules! is_block_non_ascii {
        ($block:expr) => {{
            let block = $block;
            let is_not_ascii = block & (0x80808080_80808080u64 as usize) != 0;
            // see https://jameshfisher.com/2017/01/24/bitwise-check-for-zero-byte/ for information on how this works
            let contains_zero =
                ((block.wrapping_sub(0x01010101_01010101u64 as usize)) & (!block) & (0x80808080_80808080u64 as usize))
                    != 0;
            is_not_ascii || contains_zero
        }};
    }

    let block_size = 2 * size_of::<usize>();
    let align_offset = v.as_ptr().align_offset(size_of::<usize>());

    let mut i = 0;
    while i < v.len() {
        let b1 = v[i];

        if b1 >= 0x80 {
            let width = if b1 & 0b1111_0000 == 0b1110_0000 {
                3
            } else if b1 & 0b1110_0000 == 0b1100_0000 {
                2
            } else {
                return false;
            };
            if v.len() < i + width {
                return false;
            }
            match width {
                2 => {
                    // two byte case: U+0000 and U+0080 to U+07FF
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000 {
                        return false;
                    }
                    // overlong encodings which do not encode `0` are not allowed
                    if b1 & 0b0001_1110 == 0 && (b1 != 0b1100_0000 && v[i + 1] != 0b1000_0000) {
                        return false;
                    }
                    i += 2;
                }
                3 => {
                    // three byte case: U+0800 and above
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000 || v[i + 2] & 0b1100_0000 != 0b1000_0000 {
                        return false;
                    }
                    // overlong encodings are not allowed
                    if b1 & 0b0000_1111 == 0 && v[i + 1] & 0b0010_0000 == 0 {
                        return false;
                    }
                    i += 3;
                }
                _ => return false,
            }
        } else {
            // ASCII case: U+0001 to 0+007F
            if !CONSTANTS_LARGE_ENOUGH || align_offset == usize::MAX || align_offset.wrapping_sub(i) % block_size != 0 {
                // probably unaligned
                if b1 == 0 {
                    return false;
                }
                i += 1;
            } else {
                // aligned
                while i + block_size < v.len() {
                    // SAFETY:
                    // - v.as_ptr().add(i) was verified to be aligned at this point
                    // - the block is confirmed to not exceed the input slice
                    unsafe {
                        let ptr = v.as_ptr().add(i).cast::<usize>();
                        if is_block_non_ascii!(*ptr) || is_block_non_ascii!(*ptr.offset(1)) {
                            break;
                        }
                    }
                    i += block_size;
                }

                // skip the remaining ascii characters after the last block
                while i < v.len() && v[i] < 0x80 {
                    if v[i] == 0 {
                        return false;
                    }
                    i += 1;
                }
            }
        }
    }

    true
}

/// A Modified UTF-8 string slice, like [prim@str].
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MStr {
    inner: [u8],
}

impl MStr {
    /// Creates a new string from a modified UTF-8 byte slice.
    pub fn from_mutf8(v: &[u8]) -> Result<&MStr, DecodeError> {
        if is_mutf8_valid(v) {
            // SAFETY: This is safe because the byte slice is guaranteed to be valid.
            Ok(unsafe { MStr::from_mutf8_unchecked(v) })
        } else {
            Err(DecodeError::new(DecodeErrorKind::InvalidMutf8))
        }
    }

    /// Creates a new string from a modified UTF-8 byte slice.
    pub fn from_mutf8_mut(v: &mut [u8]) -> Result<&mut MStr, DecodeError> {
        if is_mutf8_valid(v) {
            // SAFETY: This is safe because the byte slice is guaranteed to be valid.
            Ok(unsafe { MStr::from_mutf8_unchecked_mut(v) })
        } else {
            Err(DecodeError::new(DecodeErrorKind::InvalidMutf8))
        }
    }

    /// Creates a string from a modified UTF-8 byte slice without checking its contents.
    ///
    /// # Safety
    /// This slice may not contain bytes that do not make up a modified UTF-8 string.
    #[must_use]
    pub const unsafe fn from_mutf8_unchecked(v: &[u8]) -> &MStr {
        // SAFETY: Relies on &MStr and &[u8] having the same layout
        std::mem::transmute(v)
    }

    /// Creates a string from a modified UTF-8 byte slice without checking its contents.
    ///
    /// # Safety
    /// This slice may not contain bytes that do not make up a modified UTF-8 string.
    #[must_use]
    pub unsafe fn from_mutf8_unchecked_mut(v: &mut [u8]) -> &mut MStr {
        // SAFETY: Relies on &MStr and &[u8] having the same layout
        let v: *mut [u8] = v;
        &mut *(v as *mut MStr)
    }

    /// Returns the length of the string in bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the string contains any characters.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the inner byte slice.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Checks whether the modified UTF-8 string is valid UTF-8 and returns a [`&str`] if it is.
    #[inline]
    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        str::from_utf8(&self.inner).ok()
    }

    /// Returns whether the `index`-th byte is the first byte of a modified UTF-8 code point sequence or the end of the string.
    #[inline]
    #[must_use]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.len() {
            true
        } else {
            match self.as_bytes().get(index) {
                None => false,
                Some(&b) => b & 0b1100_0000 != 0b1000_0000,
            }
        }
    }

    /// Returns an iterator over the chars in this string.
    ///
    /// For valid unicode characters, `Ok` is yielded.
    /// If a character is invalid, then its code will be returned in the `Err` case.
    /// If you don't care about invalid characters, use [`chars_lossy`].
    ///
    /// [`chars_lossy`]: MStr::chars_lossy
    #[inline]
    #[must_use]
    pub fn chars(&self) -> Chars<'_> {
        Chars { inner: &self.inner }
    }

    /// Returns an iterator over the chars in this string.
    ///
    /// Invalid characters are replaced with U+FFFD ([`char::REPLACEMENT_CHARACTER`]).
    #[inline]
    #[must_use]
    pub fn chars_lossy(&self) -> CharsLossy<'_> {
        CharsLossy { inner: &self.inner }
    }

    /// Provides a value of a type that implements `Display`.
    ///
    /// Invalid characters are displayed as U+FFFD ([`char::REPLACEMENT_CHARACTER`]).
    #[inline]
    #[must_use]
    pub fn display(&self) -> Display<'_> {
        Display { inner: &self.inner }
    }
}

impl Default for &'static MStr {
    fn default() -> &'static MStr {
        // SAFETY: This is safe because an empty slice is always valid.
        unsafe { MStr::from_mutf8_unchecked(&[]) }
    }
}

impl Default for &'static mut MStr {
    fn default() -> &'static mut MStr {
        // SAFETY: This is safe because an empty slice is always valid.
        unsafe { MStr::from_mutf8_unchecked_mut(&mut []) }
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
        if index.start <= index.end && self.is_char_boundary(index.start) && self.is_char_boundary(index.end) {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            unsafe { MStr::from_mutf8_unchecked(self.inner.get_unchecked(index)) }
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
            #[allow(clippy::range_plus_one)]
            &self[*index.start()..*index.end() + 1]
        }
    }
}

impl ops::Index<ops::RangeTo<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &MStr {
        if self.is_char_boundary(index.end) {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            unsafe { MStr::from_mutf8_unchecked(self.inner.get_unchecked(index)) }
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
            #[allow(clippy::range_plus_one)]
            &self[..index.end + 1]
        }
    }
}

impl ops::Index<ops::RangeFrom<usize>> for MStr {
    type Output = MStr;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &MStr {
        if self.is_char_boundary(index.start) {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            unsafe { MStr::from_mutf8_unchecked(self.inner.get_unchecked(index)) }
        } else {
            panic!("MUtf8 index out of bounds");
        }
    }
}

impl PartialEq<MString> for MStr {
    #[inline]
    fn eq(&self, other: &MString) -> bool {
        *self == **other
    }
}

impl PartialEq<MStr> for MString {
    #[inline]
    fn eq(&self, other: &MStr) -> bool {
        **self == *other
    }
}

impl PartialEq<str> for MStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        let mut left = self.chars();
        let mut right = other.chars();
        loop {
            match (left.next(), right.next()) {
                (Some(Ok(l)), Some(r)) if l == r => {}
                (None, None) => return true,
                (_, _) => return false,
            }
        }
    }
}

impl PartialEq<MStr> for str {
    #[inline]
    fn eq(&self, other: &MStr) -> bool {
        *other == *self
    }
}

impl PartialEq<&'_ str> for MStr {
    #[inline]
    fn eq(&self, other: &&'_ str) -> bool {
        *self == **other
    }
}

impl PartialEq<MStr> for &'_ str {
    #[inline]
    fn eq(&self, other: &MStr) -> bool {
        *other == **self
    }
}

impl PartialEq<Cow<'_, MStr>> for MStr {
    #[inline]
    fn eq(&self, other: &Cow<'_, MStr>) -> bool {
        *self == **other
    }
}

impl PartialEq<MStr> for Cow<'_, MStr> {
    #[inline]
    fn eq(&self, other: &MStr) -> bool {
        **self == *other
    }
}

impl PartialOrd<str> for MStr {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        let mut left = self.chars();
        let mut right = other.chars();
        loop {
            match (left.next(), right.next()) {
                (Some(Ok(l)), Some(r)) => match l.cmp(&r) {
                    Ordering::Equal => {}
                    ord => return Some(ord),
                },
                (None, None) => return Some(Ordering::Equal),
                (None, Some(_)) => return Some(Ordering::Less),
                (Some(_), None) => return Some(Ordering::Greater),
                (Some(Err(l)), Some(r)) => return Some(l.cmp(&(r as u32))),
            }
        }
    }
}

impl PartialOrd<MStr> for str {
    #[inline]
    fn partial_cmp(&self, other: &MStr) -> Option<Ordering> {
        other.partial_cmp(self)
    }
}

/// A Modified UTF-8 string, but owned, like [`String`].
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MString {
    buf: Vec<u8>,
}

impl MString {
    /// Creates an empty string.
    #[inline]
    #[must_use]
    pub fn new() -> MString {
        MString { buf: Vec::new() }
    }

    /// Creates an empty string with capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(cap: usize) -> MString {
        MString {
            buf: Vec::with_capacity(cap),
        }
    }

    /// Creates a new string from a modified UTF-8 byte vector.
    pub fn from_mutf8(buf: Vec<u8>) -> Result<MString, DecodeError> {
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
        let size = encode_mutf8_char(ch, &mut buf);
        self.buf.extend_from_slice(&buf[..size]);
    }
}

impl Deref for MString {
    type Target = MStr;

    #[inline]
    fn deref(&self) -> &MStr {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        unsafe { MStr::from_mutf8_unchecked(&self.buf) }
    }
}

impl DerefMut for MString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        unsafe { MStr::from_mutf8_unchecked_mut(&mut self.buf) }
    }
}

impl Borrow<MStr> for MString {
    #[inline]
    fn borrow(&self) -> &MStr {
        &**self
    }
}

impl BorrowMut<MStr> for MString {
    fn borrow_mut(&mut self) -> &mut MStr {
        &mut *self
    }
}

impl AsRef<[u8]> for MString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<MStr> for MString {
    fn as_ref(&self) -> &MStr {
        &**self
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

impl fmt::Debug for MStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for c in self.chars() {
            match c {
                Ok(c) => {
                    write!(f, "{}", c.escape_debug())?;
                }
                Err(n) => {
                    // Unpaired surrogates are written as `\s{..}`.
                    write!(f, "\\s{{{n:x}}}")?;
                }
            }
        }
        f.write_char('"')
    }
}

impl fmt::Debug for MString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&**self).fmt(f)
    }
}

pub struct Display<'a> {
    inner: &'a [u8],
}

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut start = 0;
        let mut i = 0;
        while i < self.inner.len() {
            if self.inner[i] != 0b1110_1101 {
                // Three byte long
                i += 1 + i.leading_ones() as usize;
            } else {
                if i != start {
                    // SAFETY: This is safe because everything from start to i are non-zero ascii bytes.
                    f.write_str(unsafe { str::from_utf8_unchecked(&self.inner[start..i]) })?;
                }

                // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
                let (size, ch) = unsafe { decode_mutf8_char(&self.inner[i..]) };
                i += size;

                start = i;
                f.write_char(ch.unwrap_or(char::REPLACEMENT_CHARACTER))?;
            }
        }

        if i != start {
            // SAFETY: This is safe because everything from start to i are non-zero ascii bytes.
            f.write_str(unsafe { str::from_utf8_unchecked(&self.inner[start..i]) })?;
        }

        Ok(())
    }
}

impl<'a> fmt::Debug for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        unsafe { MStr::from_mutf8_unchecked(self.inner) }.fmt(f)
    }
}

pub struct Chars<'a> {
    inner: &'a [u8],
}

impl<'a> Chars<'a> {
    #[must_use]
    pub fn as_mstr(&self) -> &'a MStr {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        unsafe { MStr::from_mutf8_unchecked(self.inner) }
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = Result<char, u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            let (size, ch) = unsafe { decode_mutf8_char(self.inner) };
            self.inner = &self.inner[size..];
            Some(ch)
        }
    }
}

impl<'a> DoubleEndedIterator for Chars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            let (size, ch) = unsafe { decode_mutf8_char_reversed(self.inner) };
            self.inner = &self.inner[..self.inner.len() - size];
            Some(ch)
        }
    }
}

impl<'a> fmt::Debug for Chars<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        let s = unsafe { MStr::from_mutf8_unchecked(self.inner) };
        f.debug_struct("Chars").field("remaining", &s).finish()
    }
}

pub struct CharsLossy<'a> {
    inner: &'a [u8],
}

impl<'a> CharsLossy<'a> {
    #[must_use]
    pub fn as_mstr(&self) -> &'a MStr {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        unsafe { MStr::from_mutf8_unchecked(self.inner) }
    }
}

impl<'a> Iterator for CharsLossy<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            let (size, ch) = unsafe { decode_mutf8_char(self.inner) };
            self.inner = &self.inner[size..];
            Some(ch.unwrap_or(char::REPLACEMENT_CHARACTER))
        }
    }
}

impl<'a> DoubleEndedIterator for CharsLossy<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
            let (size, ch) = unsafe { decode_mutf8_char_reversed(self.inner) };
            self.inner = &self.inner[..self.inner.len() - size];
            Some(ch.unwrap_or(char::REPLACEMENT_CHARACTER))
        }
    }
}

impl<'a> fmt::Debug for CharsLossy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: This is safe because the underlying buffer is guaranteed to be valid.
        let s = unsafe { MStr::from_mutf8_unchecked(self.inner) };
        f.debug_struct("CharsLossy").field("remaining", &s).finish()
    }
}

/// Decodes a character and returns its size.
///
/// # Safety
/// The input bytes **must** be valid modified utf-8
unsafe fn decode_mutf8_char(v: &[u8]) -> (usize, Result<char, u32>) {
    if v[0] & 0b1000_0000 == 0b0000_0000 {
        // single byte case
        return (1, Ok(v[0] as char));
    }

    if v[0] & 0b1110_0000 == 0b1100_0000 {
        // two byte case
        let c1 = u32::from(v[0] & 0b0001_1111) << 6;
        let c2 = u32::from(v[1] & 0b0011_1111);
        return (2, Ok(char::from_u32_unchecked(c1 | c2)));
    }

    if v[0] == 0b1110_1101 {
        if v.len() >= 6 && v[1] & 0b1111_0000 == 0b1010_0000 && v[3] == 0b1110_1101 && v[4] & 0b1111_0000 == 0b1011_0000
        {
            // six byte case (paired surrogate)
            let c2 = u32::from(v[1] & 0b0000_1111) << 16;
            let c3 = u32::from(v[2] & 0b0011_1111) << 10;
            let c5 = u32::from(v[4] & 0b0000_1111) << 6;
            let c6 = u32::from(v[5] & 0b0011_1111);
            return (6, Ok(char::from_u32_unchecked(0x10000 + (c2 | c3 | c5 | c6))));
        }

        // unpaired surrogates
        if v[1] & 0b1110_0000 == 0b1010_0000 {
            let c2 = u32::from(v[1] & 0b0011_1111) << 6;
            let c3 = u32::from(v[2] & 0b0011_1111);
            return (3, Err(0b1101_0000_0000_0000 | c2 | c3));
        }
    }

    // three byte case
    let c1 = u32::from(v[0] & 0b0000_1111) << 12;
    let c2 = u32::from(v[1] & 0b0011_1111) << 6;
    let c3 = u32::from(v[2] & 0b0011_1111);
    (3, Ok(char::from_u32_unchecked(c1 | c2 | c3)))
}

/// Decodes a character from back to front and returns its size.
///
/// # Safety
/// The input bytes **must** be valid modified utf-8
unsafe fn decode_mutf8_char_reversed(v: &[u8]) -> (usize, Result<char, u32>) {
    let b1 = v[v.len() - 1];
    if b1 & 0b1000_0000 == 0b0000_0000 {
        // single byte case
        return (1, Ok(b1 as char));
    }

    let b2 = v[v.len() - 2];
    if b2 & 0b1110_0000 == 0b1100_0000 {
        // two byte case
        let c1 = u32::from(b2 & 0b0001_1111) << 6;
        let c2 = u32::from(b1 & 0b0011_1111);
        return (2, Ok(char::from_u32_unchecked(c1 | c2)));
    }

    let b3 = v[v.len() - 3];
    if b3 == 0b1110_1101 {
        if v.len() >= 6 {
            let b4 = v[v.len() - 4];
            let b5 = v[v.len() - 5];
            let b6 = v[v.len() - 6];
            if b2 & 0b1111_0000 == 0b1011_0000 && b5 & 0b1111_0000 == 0b1010_0000 && b6 == 0b1110_1101 {
                // six byte case
                let c2 = u32::from(b5 & 0b0000_1111) << 16;
                let c3 = u32::from(b4 & 0b0011_1111) << 10;
                let c5 = u32::from(b2 & 0b0000_1111) << 6;
                let c6 = u32::from(b1 & 0b0011_1111);
                return (6, Ok(char::from_u32_unchecked(0x10000 + (c2 | c3 | c5 | c6))));
            }
        }
        // unpaired surrogates
        if b2 & 0b1110_0000 == 0b1010_0000 {
            let c2 = u32::from(b2 & 0b0011_1111) << 6;
            let c3 = u32::from(b1 & 0b0011_1111);
            return (3, Err(0b1101_0000_0000_0000 | c2 | c3));
        }
    }

    // three byte case
    let c1 = u32::from(b3 & 0b0000_1111) << 12;
    let c2 = u32::from(b2 & 0b0011_1111) << 6;
    let c3 = u32::from(b1 & 0b0011_1111);
    (3, Ok(char::from_u32_unchecked(c1 | c2 | c3)))
}

impl From<&MStr> for MString {
    fn from(s: &MStr) -> MString {
        s.to_owned()
    }
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

impl<'a> Extend<&'a str> for MString {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        self.extend(iter.into_iter().flat_map(str::chars));
    }
}

/// Encodes a char to a modified UTF-8 buffer and returns its size.
/// The buffer must at least be of the length which the char will take up.
fn encode_mutf8_char(ch: char, buf: &mut [u8]) -> usize {
    let ch = ch as u32;
    match ch {
        0x01..=0x7F => {
            buf[0] = ch as u8;
            1
        }
        0 | 0x80..=0x7FF => {
            buf[0] = (0b1100_0000 | (ch >> 6)) as u8;
            buf[1] = (0b1000_0000 | (ch & 0b0011_1111)) as u8;
            2
        }
        0x800..=0xFFFF => {
            buf[0] = (0b1110_0000 | (ch >> 12)) as u8;
            buf[1] = (0b1000_0000 | ((ch >> 6) & 0b0011_1111)) as u8;
            buf[2] = (0b1000_0000 | (ch & 0b0011_1111)) as u8;
            3
        }
        _ => {
            let ch = ch - 0x10000;
            buf[0] = 0b1110_1101;
            buf[1] = (0b1010_0000 | ((ch >> 16) & 0b0000_1111)) as u8;
            buf[2] = (0b1000_0000 | ((ch >> 10) & 0b0011_1111)) as u8;
            buf[3] = 0b1110_1101;
            buf[4] = (0b1011_0000 | ((ch >> 6) & 0b0000_1111)) as u8;
            buf[5] = (0b1000_0000 | (ch & 0b0011_1111)) as u8;
            6
        }
    }
}

/// Dispatch conversion to a byte slice.
/// Used from within the [`mutf8`] macro.
#[doc(hidden)]
#[allow(non_camel_case_types, missing_debug_implementations)]
pub struct __private_MUtf8Literal<T>(pub T);

impl __private_MUtf8Literal<&'static str> {
    pub const fn is_str(self) -> bool {
        true
    }

    pub const fn as_slice(self) -> &'static [u8] {
        self.0.as_bytes()
    }
}

impl<const N: usize> __private_MUtf8Literal<&'static [u8; N]> {
    pub const fn is_str(self) -> bool {
        false
    }

    pub const fn as_slice(self) -> &'static [u8] {
        self.0
    }
}

/// Reimplementation of [`is_mutf8_valid`] but const.
/// Used from within the [`mutf8`] macro.
/// TODO: Remove this when const functions are advanced enough.
#[doc(hidden)]
pub const fn __private_is_mutf8_valid(v: &[u8]) -> bool {
    let mut i = 0;
    while i < v.len() {
        let b1 = v[i];

        if b1 == 0 {
            return false;
        } else if b1 < 0x80 {
            i += 1;
        } else {
            let width = if b1 & 0b1111_0000 == 0b1110_0000 {
                3
            } else if b1 & 0b1110_0000 == 0b1100_0000 {
                2
            } else {
                return false;
            };
            if v.len() < i + width {
                return false;
            }
            match width {
                2 => {
                    // two byte case: U+0000 and U+0080 to U+07FF
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000 {
                        return false;
                    }
                    // overlong encodings which do not encode `0` are not allowed
                    if b1 & 0b0001_1110 == 0 && (b1 != 0b1100_0000 && v[i + 1] != 0b1000_0000) {
                        return false;
                    }
                    i += 2;
                }
                3 => {
                    // three byte case: U+0800 and above
                    if v[i + 1] & 0b1100_0000 != 0b1000_0000 || v[i + 2] & 0b1100_0000 != 0b1000_0000 {
                        return false;
                    }
                    // overlong encodings are not allowed
                    if b1 & 0b0000_1111 == 0 && v[i + 1] & 0b0010_0000 == 0 {
                        return false;
                    }
                    i += 3;
                }
                _ => return false,
            }
        }
    }

    true
}

/// Computes the amount of bytes that a modified UTF-8 string would take up.
/// The input has to be valid UTF-8.
/// Used from within the [`mutf8`] macro.
#[doc(hidden)]
pub const fn __private_utf8_to_mutf8_length(v: &[u8]) -> usize {
    // The position within the UTF-8 string.
    let mut i = 0;
    // The length of the UTF-8 string converted to modified UTF-8.
    let mut len = 0;

    while i < v.len() {
        match v[i] {
            0b0000_0000 => {
                // U+0000
                i += 1;
                len += 2;
            }
            0b0000_0001..=0b0111_1111 => {
                // U+0001 to U+007F
                i += 1;
                len += 1;
            }
            0b1100_0000..=0b1101_1111 => {
                // U+0080 to 0+7FF
                i += 2;
                len += 2;
            }
            0b1110_0000..=0b1110_1111 => {
                // U+0800 to U+FFFF
                i += 3;
                len += 3;
            }
            0b1111_0000..=0b1111_0111 => {
                // U+10000 to U+10FFFF
                i += 4;
                len += 6;
            }
            // unreachable! is not const yet
            _ => panic!("can't have invalid utf-8 here"),
        }
    }

    len
}

#[doc(hidden)]
pub const fn __private_utf8_to_mutf8<const N: usize>(v: &[u8]) -> [u8; N] {
    let mut out = [0; N];
    // The position within the UTF-8 string.
    let mut i = 0;
    // The position within the modified UTF-8 output buffer.
    let mut m = 0;

    while i < v.len() {
        match v[i] {
            0b0000_0000 => {
                // U+0000
                out[m] = 0b1100_0000;
                out[m + 1] = 0b1000_0000;
                i += 1;
                m += 2;
            }
            0b0000_0001..=0b0111_1111 => {
                // U+0001 to U+007F
                out[m] = v[i];
                i += 1;
                m += 1;
            }
            0b1100_0000..=0b1101_1111 => {
                // U+0080 to 0+7FF
                out[m] = v[i];
                out[m + 1] = v[i + 1];
                i += 2;
                m += 2;
            }
            0b1110_0000..=0b1110_1111 => {
                // U+0800 to U+FFFF
                out[m] = v[i];
                out[m + 1] = v[i + 1];
                out[m + 2] = v[i + 2];
                i += 3;
                m += 3;
            }
            0b1111_0000..=0b1111_0111 => {
                // U+10000 to U+10FFFF
                out[m] = 0b1110_1101;
                out[m + 1] = 0b1010_0000 | ((v[i] & 0b0000_0111) << 1) | ((v[i + 1] & 0b0010_0000) >> 5);
                out[m + 2] = 0b1000_0000 | ((v[i + 1] & 0b0000_1111) << 2) | ((v[i + 2] & 0b0011_0000) >> 4);
                out[m + 3] = 0b1110_1101;
                out[m + 4] = 0b1011_0000 | (v[i + 2] & 0b0000_1111);
                out[m + 5] = v[i + 3];
                i += 4;
                m += 6;
            }
            // unreachable! is not const yet
            _ => panic!("can't have invalid utf-8 here"),
        }
    }

    out
}

/// Declares a modified UTF-8 literal.
///
/// The input has to be a either a string literal or a byte string literal.
/// - A string literal is converted from UTF-8 to modified UTF-8.
/// - A byte string literal is assumed to be valid modified UTF-8.
///
/// ```no_run
/// use noak::{mutf8, MStr};
///
/// const HELLO_WORLD: &MStr = mutf8!("Hello World!");
/// ```
#[macro_export]
macro_rules! mutf8 {
    ($s:literal) => {{
        // Ensure that the code is executed in a const context.
        const MSTR: &$crate::mutf8::MStr = {
            const BYTES: &[u8] = $crate::mutf8::__private_MUtf8Literal($s).as_slice();
            if $crate::mutf8::__private_MUtf8Literal($s).is_str() {
                let s = &$crate::mutf8::__private_utf8_to_mutf8::<
                    { $crate::mutf8::__private_utf8_to_mutf8_length(BYTES) },
                >(BYTES);
                // SAFETY: The converted string is guaranteed to be valid modified UTF-8.
                unsafe { $crate::mutf8::MStr::from_mutf8_unchecked(s) }
            } else {
                if !$crate::mutf8::__private_is_mutf8_valid(BYTES) {
                    panic!("literal is not a valid modified UTF-8 string.");
                }
                // SAFETY: It was verified that the string is valued modified UTF-8.
                unsafe { $crate::mutf8::MStr::from_mutf8_unchecked(BYTES) }
            }
        };
        MSTR
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_mutf8() {
        assert!(is_mutf8_valid(b"Hello World"));
        assert!(is_mutf8_valid("Ich grüße die Welt".as_bytes()));
        assert!(is_mutf8_valid("你好，世界".as_bytes()));
        // paired surrogates
        assert!(is_mutf8_valid(&[0xED, 0xA0, 0xBD, 0xED, 0xB0, 0x96]));
        // unpaired surrogates
        assert!(is_mutf8_valid(&[0xED, 0xBB, 0x8B]));
        assert!(is_mutf8_valid(&[0xED, 0xA7, 0xAB]));
        assert!(is_mutf8_valid(&[0xED, 0xAD, 0x9C, 0x26, 0x0A, 0x0A]));
    }

    #[test]
    fn invalid_mutf8() {
        assert!(!is_mutf8_valid(&[0xFF]));
        assert!(!is_mutf8_valid(&[0x00]));
        assert!(!is_mutf8_valid(&[0xED, 0xAD, 0xBD, 0xED, 0x25]));
    }

    #[test]
    fn display() {
        assert_eq!(mutf8!("Ich grüße die Welt").display().to_string(), "Ich grüße die Welt");
        assert_eq!(mutf8!("Hello 🦀").display().to_string(), "Hello 🦀");
        assert_eq!(mutf8!(b"Test \xED\xBB\x8B.").display().to_string(), "Test \u{FFFD}.");
    }

    #[test]
    fn iterate() {
        let s = MStr::from_mutf8(&[0xED, 0xA0, 0xBD, 0xED, 0xB0, 0x96]).unwrap();
        assert_eq!(s.chars().next_back(), s.chars().next());
    }

    #[test]
    fn valid_mutf8_macro() {
        assert_eq!(mutf8!("Hello World").to_str().unwrap(), "Hello World");
        assert_eq!(mutf8!("Ich grüße die Welt").to_str().unwrap(), "Ich grüße die Welt");
        assert_eq!(mutf8!("Hello 🦀").display().to_string(), "Hello 🦀");
        assert!(is_mutf8_valid(
            mutf8!(b"\xED\xA0\xBD\xED\xB0\x96 \xED\xBB\x8B \xED\xA7\xAB \xED\xAD\x9C \x26\x0A\x0A").as_bytes()
        ));
        assert_eq!(
            mutf8!("Ich grüße die 🦀.").as_bytes(),
            MString::from("Ich grüße die 🦀.").as_bytes()
        );

        assert_eq!(
            mutf8!("这里有一些三字节的案例").as_bytes(),
            MString::from("这里有一些三字节的案例").as_bytes()
        );
    }
}
