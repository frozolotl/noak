use crate::reader::cpool;
use crate::reader::decoding::*;

pub type InterfaceIter<'a> = DecodeCounted<'a, cpool::Index<cpool::Class>, u16>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;

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
