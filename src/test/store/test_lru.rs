use std::rc::Rc;
use std::cell::Cell;
use ::store::lru::{CacheValue, LruCache};


type CountRef = Rc<Cell<u32>>;
fn gen_count(n : u32) -> CountRef {
    Rc::new(Cell::new(n))
}

#[derive(Debug)]
struct MockValue {
    pub pop_callback_called_times : CountRef,
    pub poppable_called_times : CountRef,
    pub poppable : bool,
    pub key : u64,
}

impl CacheValue for MockValue {
    type KeyType = u64;
    fn pop_callback(&mut self) {
        self.pop_callback_called_times.set((*self.pop_callback_called_times).get() + 1);
    }
    fn poppable(&mut self) -> bool {
        self.poppable_called_times.set((*self.poppable_called_times).get() + 1);
        self.poppable
    }
}

impl MockValue {
    fn new(k : u64) -> Self {
        MockValue{
            pop_callback_called_times : gen_count(0),  // not used
            poppable_called_times : gen_count(0),  // not used
            poppable : true,
            key : k,
        }
    }
    fn new_with_pop_count(k : u64, poppable : CountRef, pop_callback : CountRef) -> Self {
        MockValue{
            pop_callback_called_times : pop_callback,
            poppable_called_times : poppable,
            poppable : true,
            key : k,
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
        c.put(&2, MockValue::new(2));
        assert_head!(c, 2);
        assert_eq!(c.get_load(), 1);
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
        c.put(&3, MockValue::new(3));
        assert_head!(c, 3);
        assert_eq!(c.get_load(), 2);
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
        c.put(&4, MockValue::new(4));
        assert_head!(c, 4);
        assert_eq!(c.get_load(), 3);
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
fn test_callback() {
    {
        let mut c = LruCache::new(1);
        let poppable = gen_count(0);
        let pop_callback = gen_count(0);
        c.put(&1, MockValue::new_with_pop_count(1, poppable.clone(), pop_callback.clone()));
        c.put(&2, MockValue::new(2));
        assert_eq!(poppable.get(), 1);
        assert_eq!(pop_callback.get(), 1);
    }
    {
        let mut c = LruCache::new(1);
        let poppable = gen_count(0);
        let pop_callback = gen_count(0);
        let mut mock = MockValue::new_with_pop_count(1, poppable.clone(), pop_callback.clone());
        mock.poppable = false;
        c.put(&1, mock);
        let success = c.put(&2, MockValue::new(2));
        assert!(!success);
        assert_eq!(poppable.get(), 1);
        assert_eq!(pop_callback.get(), 0);
        assert_head!(c, 1);
    }
}
