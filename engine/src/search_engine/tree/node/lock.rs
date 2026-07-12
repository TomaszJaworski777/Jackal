use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

use crate::NodeIndex;

#[derive(Debug)]
pub struct IndexLockGuard<'a>(&'a IndexLock);

impl Drop for IndexLockGuard<'_> {
    fn drop(&mut self) {
        self.0.lock.store(false, Ordering::Release);
    }
}

impl IndexLockGuard<'_> {
    pub fn value(&self) -> NodeIndex {
        NodeIndex::from(self.0.value.load(Ordering::Acquire))
    }

    pub fn store(&self, index: NodeIndex) {
        self.0.value.store(u32::from(index), Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct IndexLock {
    value: AtomicU32,
    lock: AtomicBool,
    payload: AtomicU8,
}

impl Clone for IndexLock {
    fn clone(&self) -> Self {
        Self {
            value: AtomicU32::new(self.value.load(Ordering::Relaxed)),
            lock: AtomicBool::new(self.lock.load(Ordering::Relaxed)),
            payload: AtomicU8::new(self.payload.load(Ordering::Relaxed)),
        }
    }
}

impl IndexLock {
    pub fn new(index: NodeIndex) -> Self {
        Self {
            value: AtomicU32::new(u32::from(index)),
            lock: AtomicBool::new(false),
            payload: AtomicU8::new(0),
        }
    }

    #[inline]
    pub fn payload(&self) -> u8 {
        self.payload.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn set_payload(&self, value: u8) {
        self.payload.store(value, Ordering::Relaxed);
    }

    pub fn read(&self) -> NodeIndex {
        while self.lock.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }

        NodeIndex::from(self.value.load(Ordering::Acquire))
    }

    pub fn write(&self) -> IndexLockGuard<'_> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }

        IndexLockGuard(self)
    }
}
