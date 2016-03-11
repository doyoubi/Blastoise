use std::collections::HashMap;
use std::vec::Vec;
use std::ptr::null_mut;
use std::option::Option::{Some, None};
use std::hash::{Hash, Hasher, SipHasher};
use std::fmt::Debug;


pub trait CacheValue : Clone + Debug {
    type KeyType : Hash;
    fn is_pinned(&self) -> bool;
}


type NodePtr<ValueType> = *mut Node<ValueType>;

#[derive(Clone, Debug)]
struct Node<ValueType> {
    key : u64,
    value : Option<ValueType>,
    last : NodePtr<ValueType>,
    next : NodePtr<ValueType>,
}

impl<ValueType> Node<ValueType> {
    pub fn new() -> Node<ValueType> {
        Node{
            key : 0,
            value : None,
            last : null_mut(),
            next : null_mut(),
        }
    }
}

#[derive(Debug)]
pub struct LruCache<ValueType : Clone> {
    pub capacity : usize,
    hash_map : HashMap<u64, NodePtr<ValueType>>,
    node_list : Vec<Node<ValueType>>,
    head : NodePtr<ValueType>,
    tail : NodePtr<ValueType>,
}

// dereference
macro_rules! dr {
    ($a:ident) => (unsafe{&mut *($a)});
    ($a:ident . $b:ident) => (unsafe{&mut *dr!($a).$b});
    (self . $a:ident) => (unsafe{ &mut *self.$a });
}

// dereference expression
macro_rules! dre {
    ($p:expr) => (unsafe{&mut *($p)});
}

impl<ValueType : CacheValue> LruCache<ValueType> {
    pub fn new(capacity : usize) -> LruCache<ValueType> {
        assert!(capacity > 0);
        let mut node_list : Vec<Node<ValueType>> = Vec::with_capacity(capacity);
        for _ in 0 .. capacity {
            node_list.push(Node::new());
        }
        for i in 0 .. capacity {
            let next_index = (i + 1) % capacity;
            node_list[i].next = &mut node_list[next_index];
            node_list[next_index].last = &mut node_list[i];
        }
        let mut cache = LruCache{
            capacity : capacity,
            hash_map : HashMap::with_capacity(capacity),
            node_list : node_list,
            head : null_mut(),
            tail : null_mut(),
        };
        cache.head = &mut cache.node_list[0];
        cache.tail = &mut cache.node_list[capacity - 1];
        cache
    }

    pub fn get_load(&self) -> usize {
        self.hash_map.len()
    }

    pub fn get_head(&mut self) -> Option<ValueType> {
        unsafe {
            match dr!(self.head).value {
                Some(ref mut value) => Some(value.clone()),
                None => None,
            }
        }
    }

    pub fn get(&mut self, key : &ValueType::KeyType) -> Option<ValueType> {
        if !self.get_helper(key) {
            return None;
        }
        self.get_head()
    }
    // fight borrow checker
    pub fn get_helper(&mut self, key : &ValueType::KeyType) -> bool {
        let k = hash(key);
        let hash_map = &mut self.hash_map;
        let head = &mut self.head;
        let tail = &mut self.tail;
        if let Some(node) = hash_map.get_mut(&k) {
            Self::node_to_head(head, tail, *node);
            return true;
        };
        false
    }

    pub fn prepare_page(&mut self) -> Option<ValueType> {
        // return tail value if the tail node need flush
        let head = &mut self.head;
        let mut tail = &mut self.tail;
        let first_gotten_tail : NodePtr<ValueType> = *tail;
        while let &mut Some(ref mut old) = &mut dre!(*tail).value {
            if !old.is_pinned() {
                return Some(old.clone());
            }
            let clone = tail.clone();
            Self::node_to_head(head, tail, clone);
            assert!(first_gotten_tail != tail.clone());  // all pages were pinned
        }
        None
    }

    pub fn remove_tail(&mut self) {
        // call this function if returned value of prepare_page is not None
        let tail = &mut self.tail;
        let old = extract!(&mut dre!(*tail).value, &mut Some(ref mut old), old);
        assert!(!old.is_pinned());
        let k = dre!(*tail).key;
        assert!(self.hash_map.remove(&k).is_some());
        dre!(*tail).value = None;
    }

    pub fn put(&mut self, key : &ValueType::KeyType, value : ValueType) {
        // before call this function, you should call prepare_page and remove_tail first
        let k = hash(key);
        let hash_map = &mut self.hash_map;
        assert!(!hash_map.get(&k).is_some());
        let head = &mut self.head;
        let mut tail = &mut self.tail;

        assert!(!dre!(*tail).value.is_some());

        let node = tail.clone();
        Self::node_to_head(head, tail, node);
        let head_node = dre!(*head);
        head_node.value = Some(value);
        head_node.key = k;
        hash_map.insert(k, *head);
    }

    fn node_to_head(
            head : &mut NodePtr<ValueType>,
            tail : &mut NodePtr<ValueType>,
            p : NodePtr<ValueType>) {
        if p == *head {
            return;
        } else if p == *tail {
            *head = p;
            *tail = dre!(*tail).last;
            return;
        }
        // remove
        dr!(p.last).next = dr!(p).next;
        dr!(p.next).last = dr!(p).last;
        // add to head
        dr!(p).last = *tail;
        dr!(p).next = *head;
        dre!(*tail).next = p;
        dre!(*head).last = p;
        *head = p;
    }

    pub fn capacity(&self) -> usize { self.capacity }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = SipHasher::new();
    t.hash(&mut s);
    s.finish()
}
