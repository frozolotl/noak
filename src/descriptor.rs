use crate::error::*;
use crate::mutf8::MStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDescriptor<'a> {
    dimensions: u8,
    base: BaseType<'a>,
}

impl<'a> TypeDescriptor<'a> {
    pub fn new(base: BaseType<'a>, dimensions: u8) -> TypeDescriptor {
        TypeDescriptor {
            dimensions,
            base,
        }
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

    pub fn dimensions(&self) -> u8 {
        self.dimensions
    }

    pub fn base(&self) -> &BaseType<'a> {
        &self.base
    }
}

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

#[cfg(test)]
mod test {
    use super::{*, BaseType::*};
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
        eq("Ljava/lang/String;", TypeDescriptor::new(Object(&MString::from("java/lang/String")), 0));

        eq("[D", TypeDescriptor::new(Double, 1));
        eq("[[Ljava/lang/String;", TypeDescriptor::new(Object(&MString::from("java/lang/String")), 2));
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
