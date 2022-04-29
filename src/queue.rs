use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, MutexGuard, TryLockError},
};

#[derive(Debug)]
pub struct Queue<T> {
    queue: Mutex<VecDeque<T>>,
    new_task_ready: Condvar,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            new_task_ready: Condvar::new(),
        }
    }

    /// Add `val` to the queue.
    ///
    /// Blocks if queue is currently locked.
    pub fn push(&self, val: T) {
        self.lock().push_back(val);
        self.new_task_ready.notify_one();
    }

    /// Remove an element from the queue and return it.
    ///
    /// Blocks if queue is currently locked or empty.
    pub fn pop(&self) -> T {
        let mut queue = self.lock();
        loop {
            match queue.is_empty() {
                false => return queue.pop_front().unwrap(),
                true => {
                    queue = self.new_task_ready.wait(queue).unwrap();
                    continue;
                }
            }
        }
    }

    /// Add `val` to the queue.
    ///
    /// Returns `false` if queue is currently locked.
    pub fn try_push(&self, val: T) -> bool {
        match self.queue.try_lock() {
            Ok(mut queue) => {
                queue.push_back(val);
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
        match self.queue.try_lock() {
            Ok(mut queue) => Some(queue.pop_front()),
            Err(TryLockError::WouldBlock) => None,
            Err(err @ TryLockError::Poisoned(_)) => panic!("{}", err),
        }
    }

    fn lock(&self) -> MutexGuard<VecDeque<T>> {
        self.queue.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pop_is_in_order() {
        // Arrange
        let tq = Queue::new();
        tq.push(0);
        tq.push(1);

        // Act
        let a = tq.pop();
        let b = tq.pop();

        // Assert
        assert_eq!(a, 0);
        assert_eq!(b, 1);
    }

    #[test]
    fn try_pop_empty_queue() {
        // Arrange
        let tq = Queue::<i32>::new();

        // Act
        let a = tq.try_pop();

        // Assert
        assert_eq!(a, Some(None));
    }

    #[test]
    fn try_pop_locked_queue() {
        // Arrange
        let tq = Queue::<i32>::new();
        let _handle = tq.lock();

        // Act
        let a = tq.try_pop();

        // Assert
        assert_eq!(a, None);
    }
}
