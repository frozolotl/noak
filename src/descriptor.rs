use crate::error::*;
use crate::mutf8::{Chars, MStr};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType<'a> {
    Boolean,
    Byte,
    Short,
    Integer,
    Long,
    Float,
    Double,
    Char,
    Object(&'a MStr),
}

impl<'a> fmt::Display for BaseType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BaseType::*;
        match self {
            Boolean => write!(f, "Z"),
            Byte => write!(f, "B"),
            Short => write!(f, "S"),
            Integer => write!(f, "I"),
            Long => write!(f, "J"),
            Float => write!(f, "F"),
            Double => write!(f, "D"),
            Char => write!(f, "C"),
            Object(name) => write!(f, "L{};", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDescriptor<'a> {
    dimensions: u8,
    base: BaseType<'a>,
}

impl<'a> TypeDescriptor<'a> {
    pub fn new(base: BaseType<'a>, dimensions: u8) -> TypeDescriptor {
        TypeDescriptor { dimensions, base }
    }

    pub fn parse(s: &'a MStr) -> Result<TypeDescriptor<'a>, DecodeError> {
        let mut chars = s.chars().enumerate();
        let mut dimensions = 0;
        while let Some((start, ch)) = chars.next() {
            if ch == '[' {
                if dimensions == u8::max_value() {
                    break;
                }

                dimensions += 1;
            } else {
                use BaseType::*;

                let base = match ch {
                    'Z' => Boolean,
                    'B' => Byte,
                    'S' => Short,
                    'I' => Integer,
                    'J' => Long,
                    'F' => Float,
                    'D' => Double,
                    'C' => Char,
                    'L' => {
                        let mut valid = false;
                        while let Some((_, ch)) = chars.next() {
                            if ch == ';' {
                                valid = true;
                                break;
                            }
                        }

                        if !valid {
                            break;
                        }

                        let name = &s[start + 1..s.len() - 1];
                        if name.is_empty() {
                            break;
                        }
                        Object(name)
                    }
                    _ => break,
                };

                if chars.next().is_some() {
                    break;
                }

                return Ok(TypeDescriptor { dimensions, base });
            }
        }

        Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor))
    }

    #[inline]
    pub fn dimensions(&self) -> u8 {
        self.dimensions
    }

    #[inline]
    pub fn base(&self) -> &BaseType<'a> {
        &self.base
    }
}

impl<'a> fmt::Display for TypeDescriptor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.dimensions {
            write!(f, "[")?;
        }

        write!(f, "{}", self.base)
    }
}

pub struct MethodDescriptor<'a> {
    input: &'a MStr,
    return_index: u16,
}

impl<'a> MethodDescriptor<'a> {
    pub fn parse(input: &'a MStr) -> Result<MethodDescriptor, DecodeError> {
        eprintln!("{}", input);
        if input.len() <= u16::max_value() as usize {
            let mut chars = input.chars();
            if let Some('(') = chars.next() {
                loop {
                    let ch = chars.next();
                    if let Some(')') = ch {
                        break;
                    }

                    validate_type(ch, &mut chars, false)?;
                }

                let return_index = (input.len() - chars.as_mstr().len()) as u16;
                validate_type(chars.next(), &mut chars, true)?;
                if chars.next().is_none() {
                    return Ok(MethodDescriptor {
                        input,
                        return_index,
                    });
                }
            }
        }

        Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor))
    }

    pub fn parameters(&self) -> impl Iterator<Item = TypeDescriptor<'a>> + 'a {
        struct Parameters<'a> {
            chars: Chars<'a>,
        }

        impl<'a> Iterator for Parameters<'a> {
            type Item = TypeDescriptor<'a>;

            fn next(&mut self) -> Option<TypeDescriptor<'a>> {
                let ch = self.chars.next();
                if ch == Some(')') || ch == None {
                    self.chars = <&MStr>::default().chars();
                    None
                } else {
                    read_type(ch.unwrap(), &mut self.chars)
                }

            }
        }

        let mut chars = self.input.chars();
        // skip the `(`
        chars.next();
        Parameters { chars }
    }

    pub fn return_type(&self) -> Option<TypeDescriptor<'a>> {
        let input = &self.input[self.return_index as usize..];
        if input.as_bytes() == b"V" {
            None
        } else {
            let mut chars = input.chars();
            read_type(chars.next().unwrap(), &mut chars)
        }
    }
}

fn validate_type(
    mut ch: Option<char>,
    mut chars: impl Iterator<Item = char>,
    return_type: bool,
) -> Result<(), DecodeError> {
    if return_type && ch == Some('V') {
        return Ok(());
    }

    let mut dimensions: u16 = 0;
    while let Some('[') = ch {
        if dimensions == u8::max_value() as u16 {
            return Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor));
        }
        ch = chars.next();
        dimensions += 1;
    }
    if let Some(ch) = ch {
        match ch {
            'Z' | 'B' | 'S' | 'I' | 'J' | 'F' | 'D' | 'C' => return Ok(()),
            'L' => {
                let mut found_semicolon = false;
                let mut found_character = false;
                while let Some(ch) = chars.next() {
                    if ch == ';' {
                        found_semicolon = true;
                        break;
                    } else {
                        found_character = true;
                    }
                }
                if found_semicolon && found_character {
                    return Ok(());
                }
            }
            _ => {}
        }
    }
    Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor))
}

fn read_type<'a>(mut ch: char, chars: &mut Chars<'a>) -> Option<TypeDescriptor<'a>> {
    use BaseType::*;

    let mut dimensions = 0;
    while ch == '[' {
        ch = chars.next().unwrap();
        dimensions += 1;
    }

    let base = match ch {
        'Z' => Boolean,
        'B' => Byte,
        'S' => Short,
        'I' => Integer,
        'J' => Long,
        'F' => Float,
        'D' => Double,
        'C' => Char,
        'L' => {
            let input = chars.as_mstr();
            while let Some(ch) = chars.next() {
                if ch == ';' {
                    break;
                }
            }

            let name = &input[..input.len() - chars.as_mstr().len() - 1];
            Object(name)
        }
        _ => unreachable!("the tag is guaranteed to be valid"),
    };

    return Some(TypeDescriptor { dimensions, base });
}

#[cfg(test)]
mod test {
    use super::{BaseType::*, *};
    use crate::mutf8::MString;
    use std::convert::TryInto;

    #[test]
    fn valid_type() {
        fn eq(s: &str, td: TypeDescriptor) {
            let m: MString = s.into();
            assert_eq!(TypeDescriptor::parse(&m).unwrap(), td);
        }

        eq("Z", TypeDescriptor::new(Boolean, 0));
        eq("B", TypeDescriptor::new(Byte, 0));
        eq("S", TypeDescriptor::new(Short, 0));
        eq("I", TypeDescriptor::new(Integer, 0));
        eq("J", TypeDescriptor::new(Long, 0));
        eq("F", TypeDescriptor::new(Float, 0));
        eq("D", TypeDescriptor::new(Double, 0));
        eq(
            "Ljava/lang/String;",
            TypeDescriptor::new(Object(&MString::from("java/lang/String")), 0),
        );

        eq("[D", TypeDescriptor::new(Double, 1));
        eq(
            "[[Ljava/lang/String;",
            TypeDescriptor::new(Object(&MString::from("java/lang/String")), 2),
        );
        eq("[[[[[[[[[[[[[[[[[[F", TypeDescriptor::new(Float, 18));
        eq(&("[".repeat(255) + "I"), TypeDescriptor::new(Integer, 255));
    }

    #[test]
    fn invalid_type() {
        fn check(s: &str) {
            let m: MString = s.into();
            TypeDescriptor::parse(&m).unwrap_err();
        }

        check("");
        check("X");
        check("JJ");
        check("[");
        check("[]JJ");
        check(&("[".repeat(256) + "I"));
        check("L;");
        check("Ljava/lang/Object;;");
    }

    #[test]
    fn valid_method_descriptor() {
        fn eq(s: &str, parameters: &[TypeDescriptor], return_type: Option<TypeDescriptor>) {
            let m: MString = s.into();
            let desc = MethodDescriptor::parse(&m).unwrap();

            assert_eq!(desc.parameters().collect::<Vec<_>>(), parameters);
            assert_eq!(desc.return_type(), return_type);
        }

        eq("()V", &[], None);
        eq("(I)V", &[TypeDescriptor::new(Integer, 0)], None);
        eq(
            "([[IF)V",
            &[
                TypeDescriptor::new(Integer, 2),
                TypeDescriptor::new(Float, 0),
            ],
            None,
        );
        eq(
            "(I)F",
            &[TypeDescriptor::new(Integer, 0)],
            Some(TypeDescriptor::new(Float, 0)),
        );
        eq(
            "(LFoo;)V",
            &[TypeDescriptor::new(Object(&MString::from("Foo")), 0)],
            None,
        );
        eq(
            "()LBar;",
            &[],
            Some(TypeDescriptor::new(Object(&MString::from("Bar")), 0)),
        );
        eq(
            "(LFoo;)LBar;",
            &[TypeDescriptor::new(Object(&MString::from("Foo")), 0)],
            Some(TypeDescriptor::new(Object(&MString::from("Bar")), 0)),
        );
    }

    #[test]
    fn invalid_method_descriptor() {
        fn check(s: &str) {
            let m: MString = s.into();
            assert!(MethodDescriptor::parse(&m).is_err());
        }

        check("");
        check("()");
        check("I)F");
        check("{I)F");
        check("F");
        check("(V)V");
        check("(L)V");
        check("(L;)V");
        check("(L;;)V");
        check("()LHmm");
        check("(L");
        check(&format!("({}I)V", "[".repeat(256)));
    }
}
