use libc::{_SC_PAGESIZE, sysconf, posix_memalign, c_void};


pub fn alloc_page() -> *mut c_void {
    unsafe {
        let size = sysconf(_SC_PAGESIZE) as usize;
        let mut p = std::ptr::null_mut();
        posix_memalign(&mut p, size, size);
        p
    }
}

pub fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGESIZE) as usize }
}
