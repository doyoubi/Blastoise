use std::boxed::Box;
use std::option::Option::{Some, None};
use std::rc::Rc;
use std::cell::RefCell;
use std::ptr::{write, read, null_mut};
use ::store::buffer::{Page, PagePool, DataPtr};


#[test]
fn test_page_pool() {
    let mut pool = PagePool::new(1);
    let (mut fd, mut page_index) = (11, 12);
    assert_pattern!(pool.get_page(fd, page_index), None);
    let ptr;
    pool.put_page(fd, page_index, null_mut());
    {
        let page1 = pool.get_page(fd, page_index).unwrap();
        let mut p1 = page1.borrow_mut();
        assert_eq!(p1.fd, fd);
        assert_eq!(p1.page_index, page_index);
        assert!(!p1.data.is_null());
        ptr = p1.data;
        unsafe{write::<i32>(p1.data as *mut i32, 666);}
        p1.data = null_mut();
    }
    pool.remove_tail();
    pool.put_page(21, 22, ptr);
    fd = 21;
    page_index = 22;
    {
        let page2 = pool.get_page(fd, page_index).unwrap();
        let p2 = page2.borrow();
        assert_eq!(p2.fd, fd);
        assert_eq!(p2.page_index, page_index);
        assert!(!p2.data.is_null());
        assert_eq!(p2.data, ptr);
        let n = unsafe{read::<i32>(p2.data as *const i32)};
        assert_eq!(n, 666);
    }
}

#[test]
#[should_panic]
fn test_crash() {
    let mut pool = PagePool::new(1);
    pool.put_page(11, 12, null_mut());
    pool.put_page(11, 13, null_mut());
}
