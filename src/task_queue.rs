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

    pub fn enque(&self, val: T) {
        self.lock().push_back(val);
        self.new_task_ready.notify_one();
    }

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

    pub fn try_pop(&self) -> Option<T> {
        self.lock().pop_front()
    }

    fn lock(&self) -> MutexGuard<VecDeque<T>> {
        self.data.lock().unwrap()
    }
}
