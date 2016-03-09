use std::rc::Rc;
use std::cell::Cell;
use std::result::Result;
use std::result::Result::Err;
use ::store::lru::{CacheValue, LruCache};


type CountRef = Rc<Cell<u32>>;
fn gen_count(n : u32) -> CountRef {
    Rc::new(Cell::new(n))
}

#[derive(Debug, Clone)]
struct MockValue {
    pub key : u64,
    pub pinned : bool,
}

impl CacheValue for MockValue {
    type KeyType = u64;
    fn is_pinned(&self) -> bool {
        self.pinned
    }
}

impl MockValue {
    fn new(k : u64) -> Self {
        MockValue{
            key : k,
            pinned : false,
        }
    }
    fn new_pinned(k : u64) -> Self {
        MockValue{
            key : k,
            pinned : true,
        }
    }
}

macro_rules! assert_head {
    ($cache:expr, $key:expr) => ({
        let head = $cache.get_head();
        let v = extract!(head, Some(ref v), v);
        assert_eq!(v.key, $key);
    })
}

macro_rules! assert_get {
    ($cache:expr, $key:expr) => ({
        let head = $cache.get(&$key);
        let v = extract!(head, Some(ref v), v);
        assert_eq!(v.key, $key);
    })
}


#[test]
fn test_capacity() {
    let mut c = LruCache::new(3);
    assert_eq!(3, c.capacity());
    c.put(&1, MockValue::new(1));
    assert_eq!(3, c.capacity());
}

#[test]
fn test_get_head() {
    let mut c = LruCache::new(3);
    assert_pattern!(c.get_head(), None);
    c.put(&1, MockValue::new(1));
    assert_head!(c, 1);
}

#[test]
fn test_put() {
    {
        let mut c = LruCache::new(1);
        assert_eq!(c.get_load(), 0);
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 1);
        c.remove_tail();
        c.put(&2, MockValue::new(2));
        assert_head!(c, 2);
        assert_eq!(c.get_load(), 1);
        c.remove_tail();
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 1);
    }
    {
        let mut c = LruCache::new(2);
        assert_eq!(c.get_load(), 0);
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 1);
        c.put(&2, MockValue::new(2));
        assert_head!(c, 2);
        assert_eq!(c.get_load(), 2);
        c.remove_tail();
        c.put(&3, MockValue::new(3));
        assert_head!(c, 3);
        assert_eq!(c.get_load(), 2);
        c.remove_tail();
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 2);
    }
    {
        let mut c = LruCache::new(3);
        assert_eq!(c.get_load(), 0);
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 1);
        c.put(&2, MockValue::new(2));
        assert_head!(c, 2);
        assert_eq!(c.get_load(), 2);
        c.put(&3, MockValue::new(3));
        assert_head!(c, 3);
        assert_eq!(c.get_load(), 3);
        c.remove_tail();
        c.put(&4, MockValue::new(4));
        assert_head!(c, 4);
        assert_eq!(c.get_load(), 3);
        c.remove_tail();
        c.put(&1, MockValue::new(1));
        assert_head!(c, 1);
        assert_eq!(c.get_load(), 3);
    }
}

#[test]
#[should_panic]
fn test_put_with_duplicate_key() {
    let mut c = LruCache::new(1);
    c.put(&1, MockValue::new(1));
    c.put(&1, MockValue::new(2));
}

#[test]
fn test_get() {
    {
        let mut c = LruCache::new(1);
        assert_pattern!(c.get(&0), None);
        c.put(&1, MockValue::new(1));
        assert_get!(c, 1);
        c.remove_tail();
        c.put(&2, MockValue::new(2));
        assert_get!(c, 2);
        assert_pattern!(c.get(&1), None);
    }
    {
        let mut c = LruCache::new(2);
        c.put(&1, MockValue::new(1));
        assert_get!(c, 1);
        assert_head!(c, 1);
        c.put(&2, MockValue::new(2));
        assert_get!(c, 2);
        assert_head!(c, 2);
        c.get(&1);
        assert_head!(c, 1);
        c.get(&2);
        assert_head!(c, 2);
    }
    {
        let mut c = LruCache::new(3);
        c.put(&1, MockValue::new(1));
        c.put(&2, MockValue::new(2));
        c.put(&3, MockValue::new(3));
        assert_head!(c, 3);
        c.get(&1);
        assert_head!(c, 1);
        c.get(&2);
        assert_head!(c, 2);
        c.get(&3);
        assert_head!(c, 3);
    }
}


#[test]
fn test_pinned() {
    let mut c = LruCache::new(3);
    c.put(&1, MockValue::new_pinned(1));
    c.put(&2, MockValue::new_pinned(2));
    c.put(&3, MockValue::new(3));
    assert_head!(c, 3);
    assert_pattern!(c.prepare_page(), Some(..));
    c.remove_tail();
    c.put(&4, MockValue::new(4));
    assert_pattern!(c.get(&4), Some(..));
    assert_pattern!(c.get(&1), Some(..));
    assert_pattern!(c.get(&2), Some(..));
    assert_pattern!(c.get(&3), None);
}

#[test]
#[should_panic]
fn test_page_pool_full() {
    let mut c = LruCache::new(3);
    c.put(&1, MockValue::new_pinned(1));
    c.put(&2, MockValue::new_pinned(2));
    c.put(&3, MockValue::new_pinned(3));
    assert_pattern!(c.prepare_page(), Some(..));  // panic here
    // c.remove_tail();
    // c.put(&4, MockValue::new_pinned(4));
}
