use std::{cmp, fmt, io};
use bytes::Buf;


pub struct Buffer {
    data : Vec<u8>,
    read_index : usize,
    write_index : usize,
}

impl Buffer {
    pub fn new(capacity : usize) -> Buffer {
        Buffer{
            data : Vec::with_capacity(capacity),
            read_index : 0,
            write_index : 0,
        }
    }
    pub fn reset(&mut self) {
        self.read_index = 0;
        self.write_index = 0;
        self.data.clear();
    }
}

impl Buf for Buffer {
    fn remaining(&self) -> usize {
        self.write_index - self.read_index
    }
    fn bytes(&self) -> &[u8] {
        &self.data[self.read_index..self.write_index]
    }
    fn advance(&mut self, cnt : usize) {
        let cnt = cmp::min(cnt, self.write_index - self.read_index);
        self.read_index += cnt;
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Buffer[.. {}]", self.data.len())
    }
}

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.data.write(buf) {
            Ok(size) => {
                self.write_index += size;
                Ok(size)
            }
            err => err,
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
