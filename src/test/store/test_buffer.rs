use std::boxed::Box;
use std::option::Option::{Some, None};
use std::rc::Rc;
use std::cell::Cell;
use std::ptr::{write, read};
use ::store::buffer::{CacheSaver, Page, PagePool, DataPtr};


#[derive(Debug, Copy, Clone)]
struct MockCacheSaver {
    pub fd : i32,
    pub page_index : u32,
    pub saved : bool,
}

impl CacheSaver for Rc<Cell<MockCacheSaver>> {
    fn save(&mut self, fd : i32, page_index : u32, data : DataPtr) {
        let mut mock = self.get();
        assert_eq!(mock.saved, false);
        assert_eq!(mock.fd, fd);
        assert_eq!(mock.page_index, page_index);
        assert!(!data.is_null());
        let n = unsafe{read::<i32>(data as *const i32)};
        assert_eq!(666, n);
        mock.saved = true;
        self.set(mock);
    }
}

#[test]
fn test_page_pool() {
    let mut pool = PagePool::new(1);
    let (mut fd, mut page_index) = (11, 12);
    let s1 = Rc::new(Cell::new(MockCacheSaver{
        fd : fd,
        page_index: page_index,
        saved : false,
    }));
    let s2 = Rc::new(Cell::new(MockCacheSaver{
        fd : 21,
        page_index: 22,
        saved : false,
    }));
    assert_pattern!(pool.get_page(fd, page_index), None);
    assert!(pool.put_page(fd, page_index, Box::new(s1.clone())));
    {
        let page1 = extract!(pool.get_page(fd, page_index), Some(page), page);
        let p1 = page1.write().unwrap();
        assert_eq!(p1.fd, fd);
        assert_eq!(p1.page_index, page_index);
        assert!(!p1.data.is_null());
        unsafe{write::<i32>(p1.data as *mut i32, 666);}
        assert!(!pool.put_page(21, 22, Box::new(s2.clone())));
    }
    assert!(pool.put_page(21, 22, Box::new(s2.clone())));
    assert!(s1.get().saved);
    fd = 21;
    page_index = 22;
    {
        let page2 = extract!(pool.get_page(fd, page_index), Some(page), page);
        let p2 = page2.write().unwrap();
        assert_eq!(p2.fd, fd);
        assert_eq!(p2.page_index, page_index);
        assert!(!p2.data.is_null());
        let n = unsafe{read::<i32>(p2.data as *const i32)};
        assert_eq!(n, 666);
    }
}
