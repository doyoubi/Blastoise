use std::ptr::null_mut;
use std::hash::{Hash, Hasher};
use std::option::Option;
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_void, free};
use super::lru::{CacheValue, LruCache};
use ::utils::libwrapper::alloc_page;


pub type DataPtr = *mut c_void;


#[derive(Debug, Eq, PartialEq)]
pub struct PageKey {
    pub fd : i32,
    pub page_index : u32,
}

impl Hash for PageKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut n = self.fd as u64;
        n = n.rotate_left(32);
        n = n + (self.page_index as u64);
        n.hash(state);
    }
}


pub type PageRef = Rc<RefCell<Page>>;

#[derive(Debug)]
pub struct Page {
    pub fd : i32,
    pub page_index : u32,
    pub data : DataPtr,
    pub dirty : bool,
    pub pinned : bool,
}

impl CacheValue for PageRef {
    type KeyType = PageKey;
    fn is_pinned(&self) -> bool {
        self.borrow().pinned
    }
}

impl Page {
    pub fn new(fd : i32, page_index : u32) -> Page {
        Page{
            fd : fd,
            page_index : page_index,
            data : null_mut(),
            dirty : false,
            pinned : false,
        }
    }
    pub fn alloc(&mut self) {
        assert!(self.data.is_null());
        self.data = alloc_page();
    }
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }
        unsafe{ free(self.data) };
    }
}


#[derive(Debug)]
pub struct PagePool {
    // should be protected by mutex
    cache: LruCache<PageRef>,
    unpinned : usize,
}

impl PagePool {
    pub fn new(capacity : usize) -> PagePool {
        PagePool{
            cache : LruCache::new(capacity),
            unpinned : capacity,
        }
    }
    pub fn get_capacity(&self) -> usize {
        self.cache.capacity
    }
    pub fn get_page(&mut self, fd : i32, page_index : u32) -> Option<PageRef> {
        let key = PageKey{ fd : fd, page_index : page_index };
        self.cache.get(&key)
    }
    pub fn prepare_page(&mut self) -> Option<PageRef> {
        self.cache.prepare_page()
    }
    pub fn remove_tail(&mut self) {
        self.cache.remove_tail();
    }
    pub fn put_page(&mut self, fd : i32, page_index : u32, ptr : DataPtr) {
        let key = PageKey{ fd : fd, page_index : page_index };
        let mut new_page = Page::new(fd, page_index);
        new_page.data = ptr;
        if ptr.is_null() {
            new_page.alloc();
        }
        self.cache.put(&key, Rc::new(RefCell::new(new_page)));
    }
    pub fn pin_page(&mut self, fd : i32, page_index : u32) {
        assert!(self.unpinned > 0);
        self.unpinned -= 1;
        let page = self.get_page(fd, page_index).unwrap();
        page.borrow_mut().pinned = true;
    }
    pub fn unpin_page(&mut self, fd : i32, page_index : u32) {
        assert!(self.unpinned < self.cache.capacity);
        self.unpinned += 1;
        let page = self.get_page(fd, page_index).unwrap();
        assert!(page.borrow().pinned);
        page.borrow_mut().pinned = false;
    }
    pub fn get_unpinned_num(&self) -> usize { self.unpinned }
}
