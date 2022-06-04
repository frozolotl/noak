use crate::error::{DecodeError, DecodeErrorKind};
use crate::mutf8::{CharsLossy, MStr};
use std::fmt;

/// A field type descriptor not wrapped within an array.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            Object(name) => write!(f, "L{};", name.display()),
        }
    }
}

/// A field descriptor represents the type of a class, instance or local variable.
/// It is a [`BaseType`] that may be wrapped in a n-dimensional array.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDescriptor<'a> {
    pub dimensions: u8,
    pub base: BaseType<'a>,
}

impl<'a> TypeDescriptor<'a> {
    /// Parses a field descriptor as described in [§4.3.2](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.3.2).
    ///
    /// # Examples
    /// ```
    /// use noak::descriptor::{BaseType, TypeDescriptor};
    /// use noak::MStr;
    ///
    /// let descriptor = TypeDescriptor::parse(MStr::from_mutf8(b"[F").unwrap()).unwrap();
    /// assert_eq!(descriptor.dimensions, 1);
    /// assert_eq!(descriptor.base, BaseType::Float);
    /// ```
    pub fn parse(s: &'a MStr) -> Result<TypeDescriptor<'a>, DecodeError> {
        let mut chars = s.chars_lossy().enumerate();
        let mut dimensions: u8 = 0;
        while let Some((start, ch)) = chars.next() {
            if ch == '[' {
                // Can't have more than 255 dimensions.
                dimensions = if let Some(dimensions) = dimensions.checked_add(1) {
                    dimensions
                } else {
                    break;
                };
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
                        // Assure that the descriptor contains a ';'.
                        if !chars.by_ref().any(|(_, ch)| ch == ';') {
                            break;
                        }

                        // Extract the name from the descriptor
                        let name = &s[start + 1..s.len() - 1];
                        // Assure that the name is not empty.
                        if name.is_empty() {
                            break;
                        }
                        Object(name)
                    }
                    _ => break,
                };

                // Verify that there is no character after the base type.
                if chars.next().is_some() {
                    break;
                }

                return Ok(TypeDescriptor { dimensions, base });
            }
        }

        Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor))
    }
}

impl<'a> fmt::Display for TypeDescriptor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.dimensions {
            write!(f, "[")?;
        }

        write!(f, "{}", self.base)
    }
}

/// A method descriptor specifies the parameter types and return type of a method.
pub struct MethodDescriptor<'a> {
    input: &'a MStr,
    return_index: usize,
}

impl<'a> MethodDescriptor<'a> {
    /// Parses a method descriptor as described in [§4.3.3](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.3.3).
    ///
    /// # Examples
    /// ```
    /// use noak::descriptor::{BaseType, MethodDescriptor, TypeDescriptor};
    /// use noak::MStr;
    ///
    /// let descriptor = MethodDescriptor::parse(MStr::from_mutf8(b"(Ljava/lang/String;)I").unwrap()).unwrap();
    /// assert_eq!(descriptor.parameters().count(), 1);
    /// assert_eq!(descriptor.return_type(), Some(TypeDescriptor { dimensions: 0, base: BaseType::Integer }));
    /// ```
    pub fn parse(input: &'a MStr) -> Result<MethodDescriptor<'a>, DecodeError> {
        let mut chars = input.chars_lossy();
        if let Some('(') = chars.next() {
            loop {
                let ch = chars.next();
                if let Some(')') = ch {
                    break;
                }

                validate_type(ch, &mut chars, false)?;
            }

            let return_index = input.len() - chars.as_mstr().len();
            validate_type(chars.next(), &mut chars, true)?;
            if chars.next().is_none() {
                return Ok(MethodDescriptor { input, return_index });
            }
        }

        Err(DecodeError::new(DecodeErrorKind::InvalidDescriptor))
    }

    /// Returns an iterator over the method parameters.
    pub fn parameters(&self) -> impl Iterator<Item = TypeDescriptor<'a>> + 'a {
        struct Parameters<'a> {
            chars: CharsLossy<'a>,
        }

        impl<'a> Iterator for Parameters<'a> {
            type Item = TypeDescriptor<'a>;

            fn next(&mut self) -> Option<TypeDescriptor<'a>> {
                let ch = self.chars.next();
                if ch == Some(')') || ch == None {
                    self.chars = <&MStr>::default().chars_lossy();
                    None
                } else {
                    Some(read_type(ch.unwrap(), &mut self.chars))
                }
            }
        }

        let mut chars = self.input.chars_lossy();
        // skip the `(`
        chars.next();
        Parameters { chars }
    }

    /// Returns the return type of this method descriptor.
    /// If the return type is void (`V`), then `None` is returned.
    #[must_use]
    pub fn return_type(&self) -> Option<TypeDescriptor<'a>> {
        let input = &self.input[self.return_index..];
        if input.as_bytes() == b"V" {
            None
        } else {
            let mut chars = input.chars_lossy();
            Some(read_type(chars.next().unwrap(), &mut chars))
        }
    }
}

/// Verify that the next type is valid.
fn validate_type(mut ch: Option<char>, chars: &mut CharsLossy<'_>, return_type: bool) -> Result<(), DecodeError> {
    if return_type && ch == Some('V') {
        return Ok(());
    }

    let mut dimensions: u8 = 0;
    while let Some('[') = ch {
        // Can't have more than 255 array dimensions.
        dimensions = dimensions
            .checked_add(1)
            .ok_or_else(|| DecodeError::new(DecodeErrorKind::InvalidDescriptor))?;
        ch = chars.next();
    }
    if let Some(ch) = ch {
        match ch {
            'Z' | 'B' | 'S' | 'I' | 'J' | 'F' | 'D' | 'C' => return Ok(()),
            'L' => {
                let mut found_semicolon = false;
                let mut found_character = false;
                for ch in chars {
                    if ch == ';' {
                        found_semicolon = true;
                        break;
                    }
                    found_character = true;
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

/// Reads a type descriptor from a method descriptor.
/// Does not verify the input.
fn read_type<'a>(mut ch: char, chars: &mut CharsLossy<'a>) -> TypeDescriptor<'a> {
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
            for ch in chars.by_ref() {
                if ch == ';' {
                    break;
                }
            }

            let name = &input[..input.len() - chars.as_mstr().len() - 1];
            Object(name)
        }
        _ => unreachable!("the tag is guaranteed to be valid"),
    };

    TypeDescriptor { dimensions, base }
}

impl<'a> fmt::Debug for MethodDescriptor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MethodDescriptor")
            .field("parameters", &self.parameters().collect::<Vec<_>>())
            .field("return_type", &self.return_type())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::{BaseType::*, *};
    use crate::MString;

    #[test]
    fn valid_type() {
        fn eq(s: &str, td: TypeDescriptor<'_>) {
            let m: MString = s.into();
            assert_eq!(TypeDescriptor::parse(&m).unwrap(), td);
        }

        eq(
            "Z",
            TypeDescriptor {
                base: Boolean,
                dimensions: 0,
            },
        );
        eq(
            "B",
            TypeDescriptor {
                base: Byte,
                dimensions: 0,
            },
        );
        eq(
            "S",
            TypeDescriptor {
                base: Short,
                dimensions: 0,
            },
        );
        eq(
            "I",
            TypeDescriptor {
                base: Integer,
                dimensions: 0,
            },
        );
        eq(
            "J",
            TypeDescriptor {
                base: Long,
                dimensions: 0,
            },
        );
        eq(
            "F",
            TypeDescriptor {
                base: Float,
                dimensions: 0,
            },
        );
        eq(
            "D",
            TypeDescriptor {
                base: Double,
                dimensions: 0,
            },
        );
        eq(
            "Ljava/lang/String;",
            TypeDescriptor {
                base: Object(&MString::from("java/lang/String")),
                dimensions: 0,
            },
        );

        eq(
            "[D",
            TypeDescriptor {
                base: Double,
                dimensions: 1,
            },
        );
        eq(
            "[[Ljava/lang/String;",
            TypeDescriptor {
                base: Object(&MString::from("java/lang/String")),
                dimensions: 2,
            },
        );
        eq(
            "[[[[[[[[[[[[[[[[[[F",
            TypeDescriptor {
                base: Float,
                dimensions: 18,
            },
        );
        eq(
            &("[".repeat(255) + "I"),
            TypeDescriptor {
                base: Integer,
                dimensions: 255,
            },
        );
    }

    #[test]
    fn invalid_type() {
        #[track_caller]
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
        #[track_caller]
        fn eq(s: &str, parameters: &[TypeDescriptor<'_>], return_type: Option<TypeDescriptor<'_>>) {
            let m: MString = s.into();
            let desc = MethodDescriptor::parse(&m).unwrap();

            assert_eq!(desc.parameters().collect::<Vec<_>>(), parameters);
            assert_eq!(desc.return_type(), return_type);
        }

        eq("()V", &[], None);
        eq(
            "(I)V",
            &[TypeDescriptor {
                dimensions: 0,
                base: Integer,
            }],
            None,
        );
        eq(
            "([[IF)V",
            &[
                TypeDescriptor {
                    dimensions: 2,
                    base: Integer,
                },
                TypeDescriptor {
                    dimensions: 0,
                    base: Float,
                },
            ],
            None,
        );
        eq(
            "(I)F",
            &[TypeDescriptor {
                dimensions: 0,
                base: Integer,
            }],
            Some(TypeDescriptor {
                dimensions: 0,
                base: Float,
            }),
        );
        eq(
            "(LFoo;)V",
            &[TypeDescriptor {
                dimensions: 0,
                base: Object(&MString::from("Foo")),
            }],
            None,
        );
        eq(
            "()LBar;",
            &[],
            Some(TypeDescriptor {
                dimensions: 0,
                base: Object(&MString::from("Bar")),
            }),
        );
        eq(
            "(LFoo;)LBar;",
            &[TypeDescriptor {
                dimensions: 0,
                base: Object(&MString::from("Foo")),
            }],
            Some(TypeDescriptor {
                dimensions: 0,
                base: Object(&MString::from("Bar")),
            }),
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
