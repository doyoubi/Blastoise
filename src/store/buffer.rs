use std::ptr::null_mut;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::option::Option;
use std::fmt::Debug;
use std::boxed::Box;
use libc::{c_void, free};
use super::lru::{CacheValue, LruCache};
use ::utils::libwrapper::alloc_page;


pub type DataPtr = *mut c_void;
pub type CacheSaverRef = Box<CacheSaver>;

pub trait CacheSaver : Debug {
    fn save(&mut self, fd : i32, page_index : u32, data : DataPtr);
}


#[derive(Debug)]
struct PageKey {
    fd : i32,
    page_index : u32,
}

impl Hash for PageKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut n = self.fd as u64;
        n.rotate_left(32);
        n = n + (self.page_index as u64);
        n.hash(state);
    }
}


type PageRef = Arc<RwLock<Page>>;

#[derive(Debug)]
pub struct Page {
    pub fd : i32,
    pub page_index : u32,
    pub data : DataPtr,
    pub dirty : bool,
    saver : CacheSaverRef,
}

impl CacheValue for PageRef {
    type KeyType = PageKey;
    fn pop_callback(self, new_value : &mut PageRef)
            -> Result<(), Self> {
        let old_value = try!(Arc::try_unwrap(self));
        let mut old_page = old_value.write().unwrap();
        assert!(!old_page.data.is_null());
        let mut new_page = new_value.write().unwrap();
        // if place new_page.data = old_page.data here, it will crash, maybe a compiler bug
        old_page.save();
        new_page.data = old_page.data;  // place here to get around the crash problem
        old_page.data = null_mut();
        Ok(())
    }
}

impl Page {
    pub fn new(fd : i32, page_index : u32, saver : CacheSaverRef) -> Page {
        Page{
            fd : fd,
            page_index : page_index,
            data : null_mut(),
            dirty : false,
            saver : saver,
        }
    }
    pub fn alloc(&mut self) {
        assert!(self.data.is_null());
        self.data = alloc_page();
    }
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    pub fn save(&mut self) {
        self.saver.save(self.fd, self.page_index, self.data);
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
}

impl PagePool {
    pub fn new(capacity : usize) -> PagePool {
        PagePool{
            cache : LruCache::new(capacity),
        }
    }
    pub fn get_page(&mut self, fd : i32, page_index : u32) -> Option<PageRef> {
        let key = PageKey{ fd : fd, page_index : page_index };
        self.cache.get(&key)
    }
    pub fn put_page(&mut self, fd : i32, page_index : u32, saver : CacheSaverRef) -> bool {
        let key = PageKey{ fd : fd, page_index : page_index };
        let mut new_page = Page::new(fd, page_index, saver);
        if self.cache.get_load() < self.cache.capacity() {
            new_page.alloc();
        }
        let r = Arc::new(RwLock::new(new_page));
        self.cache.put(&key, r)
    }
}
