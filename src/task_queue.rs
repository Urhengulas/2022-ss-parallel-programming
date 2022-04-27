use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, MutexGuard, TryLockError},
};

#[derive(Debug)]
pub struct TaskQueue<T> {
    data: Mutex<VecDeque<T>>,
    new_task_ready: Condvar,
}

impl<T> TaskQueue<T> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            new_task_ready: Condvar::new(),
        }
    }

    /// Add `val` to the queue.
    ///
    /// Blocks if queue is currently locked.
    pub fn enque(&self, val: T) {
        self.lock().push_back(val);
        self.new_task_ready.notify_one();
    }

    /// Remove an element from the queue and return it.
    ///
    /// Blocks if queue is currently locked or empty.
    pub fn pop(&self) -> T {
        let mut data = self.lock();
        loop {
            match data.is_empty() {
                false => return data.pop_front().unwrap(),
                true => {
                    data = self.new_task_ready.wait(data).unwrap();
                    continue;
                }
            }
        }
    }

    /// Add `val` to the queue.
    ///
    /// Returns `false` if queue is currently locked.
    pub fn try_enque(&self, val: T) -> bool {
        match self.data.try_lock() {
            Ok(mut data) => {
                data.push_back(val);
                self.new_task_ready.notify_one();
                true
            }
            Err(TryLockError::WouldBlock) => false,
            Err(err @ TryLockError::Poisoned(_)) => panic!("{}", err),
        }
    }

    /// Remove an element from the queue and return it.
    ///
    /// Returns `None` if the queue is locked.
    /// Returns `Some(None)` if the queue is empty.
    pub fn try_pop(&self) -> Option<Option<T>> {
        match self.data.try_lock() {
            Ok(mut data) => Some(data.pop_front()),
            Err(TryLockError::WouldBlock) => None,
            Err(err @ TryLockError::Poisoned(_)) => panic!("{}", err),
        }
    }

    fn lock(&self) -> MutexGuard<VecDeque<T>> {
        self.data.lock().unwrap()
    }
}
