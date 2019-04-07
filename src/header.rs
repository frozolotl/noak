use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

bitflags! {
    pub struct AccessFlags: u16 {
        const PUBLIC = 1 << 0;
        const PRIVATE = 1 << 1;
        const PROTECTED = 1 << 2;
        const STATIC = 1 << 3;
        const FINAL = 1 << 4;
        const SUPER = 1 << 5;
        const SYNCHRONIZED = 1 << 5;
        const BRIDGE = 1 << 6;
        const VARARGS = 1 << 7;
        const NATIVE = 1 << 8;
        const INTERFACE = 1 << 9;
        const ABSTRACT = 1 << 10;
        const STRICT = 1 << 11;
        const SYNTHETIC = 1 << 12;
        const ANNOTATION = 1 << 13;
        const ENUM = 1 << 14;
        const MANDATED = 1 << 15;
        const MODULE = 1 << 15;
    }
}
