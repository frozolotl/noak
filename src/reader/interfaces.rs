use crate::encoding::{Decode, Decoder};
use crate::error::*;
use crate::mutf8::MStr;
use crate::reader::cpool;
use std::fmt;
use std::iter::FusedIterator;

/// An iterator over the interface indices in a class.
#[derive(Clone)]
pub struct Interfaces<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Decode<'a> for Interfaces<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, DecodeError> {
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

impl<'a> fmt::Debug for Interfaces<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Interfaces").finish()
    }
}

/// An iterator over the interface names in a class.
#[derive(Clone)]
pub struct InterfaceNames<'a, 'b> {
    interfaces: Interfaces<'a>,
    pool: &'b cpool::ConstantPool<'a>,
}

impl<'a, 'b> InterfaceNames<'a, 'b> {
    pub(in crate::reader) fn new(
        pool: &'b cpool::ConstantPool<'a>,
        interfaces: Interfaces<'a>,
    ) -> InterfaceNames<'a, 'b> {
        InterfaceNames { interfaces, pool }
    }
}

impl<'a, 'b> Iterator for InterfaceNames<'a, 'b> {
    type Item = &'a MStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.interfaces.next()?;
        get_name(self.pool, v)
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
        let v = self.interfaces.nth(n)?;
        get_name(self.pool, v)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let v = self.interfaces.last()?;
        get_name(self.pool, v)
    }
}

impl<'a, 'b> ExactSizeIterator for InterfaceNames<'a, 'b> {
    #[inline]
    fn len(&self) -> usize {
        self.interfaces.len()
    }
}

impl<'a, 'b> FusedIterator for InterfaceNames<'a, 'b> {}

impl<'a, 'b> fmt::Debug for InterfaceNames<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InterfaceNames").finish()
    }
}

fn get_name<'a>(
    pool: &cpool::ConstantPool<'a>,
    interface_index: cpool::Index<cpool::Class>,
) -> Option<&'a MStr> {
    let interface = pool.get(interface_index).ok()?.name;
    let name = pool.get(interface).ok()?.content;
    Some(name)
}

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

        let mut interfaces = Interfaces::decode(&mut decoder).unwrap();
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

        assert!(Interfaces::decode(&mut decoder).is_err());

        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00,
        ], Context::Interfaces);

        assert!(Interfaces::decode(&mut decoder).is_err());
    }
}
