use std::{fmt::{self, Error, Write}, str::from_utf8};

#[derive(Debug)]
pub struct StackBuffer<const N: usize> {
    pub buf: [u8; N],
    pos: usize
}

#[allow(dead_code)]
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

impl<const N: usize> std::ops::ShlAssign<&str> for StackBuffer<N> {
    fn shl_assign(&mut self, rhs: &str) {
        let bytes = rhs.as_bytes();
        let len = bytes.len().min(self.buf.len());
        // assert!(self.buf.len() > self.pos + len, "Buffer wasnt big enough ! Size: {}", self.buf.len());
        self.buf[..len].copy_from_slice(&bytes[..len]);
        self.pos += bytes.len();
    }
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

impl<const N: usize> fmt::Display for StackBuffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}