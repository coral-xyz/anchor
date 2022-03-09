use solana_program::program_memory;
use std::cmp;
use std::io::{self, Write};

#[derive(Debug, Default)]
pub struct BpfWriter<T> {
    inner: T,
    pos: u64,
}

impl<T> BpfWriter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

impl Write for BpfWriter<&mut [u8]> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos = cmp::min(self.pos, self.inner.len() as u64);
        let writable_size = self.inner.len().saturating_sub(pos as usize);

        if writable_size == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero));
        }

        program_memory::sol_memcpy(&mut self.inner[(pos as usize)..], buf, writable_size);
        self.pos += writable_size as u64;
        Ok(writable_size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
