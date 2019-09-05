use crate::writer::encoding::*;

#[derive(Clone)]
pub struct ClassWriter {
    encoder: VecEncoder,
}

impl ClassWriter {
    pub fn new() -> ClassWriter {
        ClassWriter::with_capacity(2048)
    }

    pub fn with_capacity(capacity: usize) -> ClassWriter {
        ClassWriter {
            encoder: VecEncoder::with_capacity(capacity),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.encoder.as_bytes()
    }

    pub fn finish(self) -> Vec<u8> {
        self.encoder.into_inner()
    }
}
