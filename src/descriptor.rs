use crate::error::*;
use crate::mutf8::{MStr, Chars};
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
}

impl<'a> MethodDescriptor<'a> {
    pub fn parse(input: &'a MStr) -> Result<MethodDescriptor, DecodeError> {
        let mut chars = input.chars();
        if let Some('(') = chars.next() {
            loop {
                if let Some(')') = chars.next() {
                    break;
                }

                validate_type(&mut chars, false)?;
            }

            validate_type(&mut chars, true)?;
            if chars.next().is_none() {
                return Ok(MethodDescriptor {
                    input,
                })
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
                if let Some(')') = self.chars.next() {
                    return None;
                }

                let mut ch = self.chars.next();
                let mut dimensions = 0;
                while let Some('[') = ch {
                    ch = self.chars.next();
                    dimensions += 1;
                }
                if let Some(ch) = ch {
                    use BaseType::*;

                    let base = match ch {
                        'Z' => Boolean,
                        'B' => Byte,
                        'S' => Short,
                        'I' => Integer,
                        'J' => Long,
                        'F' => Float,
                        'D' => Double,
                        'L' => {
                            let input = self.chars.as_mstr();
                            while let Some(ch) = self.chars.next() {
                                if ch == ';' {
                                    break;
                                }
                            }

                            let name = &input[..input.len() - self.chars.as_mstr().len()];
                            Object(name)
                        }
                        _ => unreachable!("it's guaranteed to be valid"),
                    };

                    return Some(TypeDescriptor { dimensions, base });
                }

                None
            }
        }

        let mut chars = self.input.chars();
        // skip the `(`
        chars.next();
        Parameters {
            chars,
        }
    }
}

fn validate_type(mut chars: impl Iterator<Item = char>, return_type: bool) -> Result<(), DecodeError> {
    let mut ch = chars.next();
    if return_type && ch == Some('V') {
        return Ok(());
    }

    while let Some('[') = ch {
        ch = chars.next();
    }
    if let Some(ch) = ch {
        match ch {
            'Z' | 'B' | 'S' | 'I' | 'J' | 'F' | 'D' => return Ok(()),
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

#[cfg(test)]
mod test {
    use super::{BaseType::*, *};
    use crate::mutf8::MString;

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
}
