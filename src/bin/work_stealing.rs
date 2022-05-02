use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, TryRecvError},
        Arc, Mutex, TryLockError,
    },
    thread,
};

use krate::{run_server, Task, ThreadPool};

fn main() {
    let pool = WorkStealing::new(4);
    run_server(pool);
}

pub struct WorkStealing {
    next_thread: AtomicUsize,
    num_threads: usize,
    senders: Vec<mpsc::Sender<Task>>,
    _workers: Vec<Worker>,
}

impl WorkStealing {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);

        let mut _workers = Vec::with_capacity(num_threads);
        let mut senders = Vec::with_capacity(num_threads);
        let mut receivers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let (sender, receiver) = mpsc::channel();
            senders.push(sender);
            receivers.push(Mutex::new(receiver));
        }

        let receivers = Arc::new(receivers);
        for i in 0..num_threads {
            let worker = Worker::new(i, Arc::clone(&receivers));
            _workers.push(worker);
        }

        Self {
            next_thread: AtomicUsize::new(0),
            num_threads,
            _workers,
            senders,
        }
    }
}

impl ThreadPool for WorkStealing {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // get the current `next_thread` and increment it
        let next_thread = self
            .next_thread
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
                Some((x + 1) % self.num_threads)
            })
            .unwrap();
        self.senders[next_thread].send(Box::new(f)).unwrap();
    }
}

struct Worker {
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receivers: Arc<Vec<Mutex<mpsc::Receiver<Task>>>>) -> Self {
        let num_threads = receivers.len();

        let _thread = thread::spawn({
            move || 'outer: loop {
                'inner: for i in 0..num_threads {
                    let j = (id + i) % num_threads;
                    let receiver = match receivers[j].try_lock() {
                        Ok(receiver) => receiver,
                        Err(TryLockError::WouldBlock) => continue 'inner,
                        Err(TryLockError::Poisoned(_)) => unreachable!(),
                    };
                    let job = match receiver.try_recv() {
                        Ok(job) => job,
                        Err(TryRecvError::Empty) => continue 'inner,
                        Err(TryRecvError::Disconnected) => break 'outer,
                    };
                    job();
                    continue 'outer;
                }

                let job = match receivers[id].lock().unwrap().recv() {
                    Ok(job) => job,
                    Err(_) => break 'outer,
                };
                job();
            }
        });
        Self { _thread }
    }
}
