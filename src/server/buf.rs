use std::{cmp, fmt, io, ptr};
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
    pub fn should_shift(&self, add_size : usize) -> bool {
        if add_size <= self.data.capacity() - self.write_index {
            return false;
        }
        let left_size = self.read_index;
        let right_size = self.data.capacity() - self.write_index;
        left_size + right_size >= add_size
    }
    fn left_shift(&mut self) {
        let new_size = self.write_index - self.read_index;
        unsafe{
            let src = self.data.as_mut_ptr().offset(self.read_index as isize);
            let dst = self.data.as_mut_ptr();
            ptr::copy(src, dst, new_size);
        }
        self.data.resize(new_size, 0);
        self.read_index = 0;
        self.write_index = new_size;
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
    fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Buffer[.. {}]", self.data.len())
    }
}

impl io::Write for Buffer {
    fn write(&mut self, buf : &[u8]) -> io::Result<usize> {
        if self.should_shift(buf.len()) {
            self.left_shift();
        }
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
