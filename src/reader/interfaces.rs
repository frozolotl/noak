use crate::error::*;
use crate::mutf8::MStr;
use crate::reader::cpool;
use crate::reader::decoding::*;
use std::fmt;
use std::iter::FusedIterator;

pub type InterfaceIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

/// An iterator over the interface names in a class.
#[derive(Clone)]
pub struct InterfaceNameIter<'a, 'b> {
    interfaces: InterfaceIter<'a>,
    pool: &'b cpool::ConstantPool<'a>,
}

impl<'a, 'b> InterfaceNameIter<'a, 'b> {
    pub(in crate::reader) fn new(
        pool: &'b cpool::ConstantPool<'a>,
        interfaces: InterfaceIter<'a>,
    ) -> InterfaceNameIter<'a, 'b> {
        InterfaceNameIter { interfaces, pool }
    }
}

impl<'a, 'b> Iterator for InterfaceNameIter<'a, 'b> {
    type Item = Result<&'a MStr, DecodeError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let interface_index = match self.interfaces.next() {
            Some(Ok(idx)) => idx,
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        };
        let interface = self.pool.get(interface_index).ok()?.name;
        let name = self.pool.get(interface).ok()?.content;
        Some(Ok(name))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.interfaces.size_hint()
    }
}

impl<'a, 'b> FusedIterator for InterfaceNameIter<'a, 'b> {}

impl<'a, 'b> fmt::Debug for InterfaceNameIter<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InterfaceNameIter").finish()
    }
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

        let mut interfaces = InterfaceIter::decode(&mut decoder).unwrap();
        assert_eq!(
            interfaces.next(),
            Some(Ok(cpool::Index::new(0x0023).unwrap()))
        );
        assert_eq!(
            interfaces.next(),
            Some(Ok(cpool::Index::new(0x0555).unwrap()))
        );
        assert_eq!(
            interfaces.next(),
            Some(Ok(cpool::Index::new(0x0001).unwrap()))
        );
        assert_eq!(interfaces.next(), None);
    }

    #[test]
    fn invalid_length() {
        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00, 0x03,
            0x12,
        ], Context::Interfaces);

        assert!(InterfaceIter::decode(&mut decoder).is_err());

        #[rustfmt::skip]
        let mut decoder = Decoder::new(&[
            0x00,
        ], Context::Interfaces);

        assert!(InterfaceIter::decode(&mut decoder).is_err());
    }
}
