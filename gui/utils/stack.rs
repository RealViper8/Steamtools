use std::{fmt::{Error, Write}, str::from_utf8};

#[derive(Debug)]
pub struct StackBuffer<const N: usize> {
    pub buf: [u8; N],
    pos: usize
}

impl<const N: usize> StackBuffer<N> {
    pub fn new() -> Self {
        Self { buf: [0u8; N], pos: 0 }
    }

    pub fn as_str(&self) -> &str {
        from_utf8(&self.buf[..self.pos]).unwrap()
    }

    pub fn clear(&mut self) {
        self.buf.fill(0);
        self.pos = 0;
    }
}

#[macro_export]
macro_rules! stack_buffer {
    ($size:expr) => {
        StackBuffer::<$size>::new()
    };
}


impl<const N: usize> Write for StackBuffer<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();
        if self.pos + bytes.len() > self.buf.len() {
            return Err(Error);
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}