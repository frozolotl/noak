use crate::encoding::Decoder;
use crate::error::*;
use crate::reader::cpool;
use std::iter::FusedIterator;

/// An iterator over the interfaces in a class.
#[derive(Clone)]
pub struct Interfaces<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Interfaces<'a> {
    pub(in crate::reader) fn new(decoder: &mut Decoder<'a>) -> Result<Interfaces<'a>, DecodeError> {
        let count: u16 = decoder.read()?;
        let limit = count as usize * 2;
        let interface_decoder = decoder.limit(limit, Context::Interfaces)?;
        decoder.advance(limit)?;

        Ok(Interfaces {
            decoder: interface_decoder,
        })
    }
}

impl<'a> Iterator for Interfaces<'a> {
    type Item = cpool::Index<cpool::Class>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.read().ok()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.decoder.advance(n * 2).ok()?;
        self.decoder.read().ok()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.decoder
            .advance(self.decoder.bytes_remaining().saturating_sub(2))
            .ok()?;
        self.decoder.read().ok()
    }
}

impl<'a> ExactSizeIterator for Interfaces<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.decoder.bytes_remaining() / 2
    }
}

impl<'a> FusedIterator for Interfaces<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_interfaces() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00, 0x03,
            0x00, 0x23,
            0x05, 0x55,
            0x00, 0x01,
        ], Context::Interfaces);

        let mut interfaces = Interfaces::new(&mut decoder).unwrap();
        assert_eq!(interfaces.len(), 3);
        assert_eq!(interfaces.next(), Some(cpool::Index::new(0x0023).unwrap()));
        assert_eq!(interfaces.next(), Some(cpool::Index::new(0x0555).unwrap()));
        assert_eq!(interfaces.next(), Some(cpool::Index::new(0x0001).unwrap()));
        assert_eq!(interfaces.next(), None);
    }

    #[test]
    fn invalid_length() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00, 0x03,
            0x12,
        ], Context::Interfaces);

        assert!(Interfaces::new(&mut decoder).is_err());

        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00,
        ], Context::Interfaces);

        assert!(Interfaces::new(&mut decoder).is_err());
    }
}
