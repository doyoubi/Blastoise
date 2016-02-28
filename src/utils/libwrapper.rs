use libc::{_SC_PAGESIZE, sysconf, memalign, c_void};


pub fn alloc_page() -> *mut c_void {
    unsafe {
        let size = sysconf(_SC_PAGESIZE) as usize;
        memalign(size, size)        
    }
}

pub fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGESIZE) as usize }
}
