use std::ptr::{write, read, write_bytes};
use std::vec::Vec;
use std::ffi::CString;
use ::store::buffer::DataPtr;


pub fn char_to_u8(c : char) -> u8 {
    if (c as u32) > 127 {
        return 42;  // invalid ascii to '*'
    }
    c as u8
}

pub unsafe fn write_string(ptr : DataPtr, input : &String, len : usize) {
    assert!(input.len() <= len);
    write_bytes(ptr, 0, len);
    for (i, c) in input.chars().enumerate() {
        let byte = char_to_u8(c);
        write::<u8>((ptr as *mut u8).offset(i as isize), byte);
    }
}

pub unsafe fn read_string(ptr : DataPtr, len : usize) -> String {
    let mut s = String::new();
    for i in 0..len {
        let n = read::<u8>((ptr as *const u8).offset(i as isize));
        if n == 0 {
            return s;
        }
        s.push(n as char);
    }
    s
}

pub fn pointer_offset(ptr : DataPtr, byte_offset : usize) -> DataPtr {
    unsafe{
        (ptr as *mut u8).offset(byte_offset as isize) as DataPtr
    }
}

pub fn to_cstring(s : String) -> CString {
    let mut buf = Vec::new();
    for c in s.chars() {
        buf.push(char_to_u8(c));
    }
    CString::new(buf).unwrap()
}
