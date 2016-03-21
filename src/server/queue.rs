use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;


pub type BlockingQueueRef<T> = Arc<BlockingQueue<T>>;

pub struct BlockingQueue<T> {
    queue : Mutex<VecDeque<T>>,
    condvar : Condvar,
}

impl<T> BlockingQueue<T> {
    pub fn new(capacity : usize) -> Self {
        BlockingQueue{
            queue : Mutex::new(VecDeque::with_capacity(capacity)),
            condvar : Condvar::new(),
        }
    }
}

impl<T> BlockingQueue<T> {
    pub fn pop_front(&self) -> T {
        let mut q = self.queue.lock().unwrap();
        while let None = q.front() {
            q = self.condvar.wait(q).unwrap();
        }
        q.pop_front().unwrap()
    }

    pub fn push_back(&self, item : T) {
        self.queue.lock().unwrap().push_back(item);
        self.condvar.notify_one();
    }
}
