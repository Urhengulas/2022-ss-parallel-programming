use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

use krate::{run_server, Task, ThreadPool};

fn main() {
    let pool = MultiQueueMultiThread::new(1);
    run_server(pool);
}

pub struct MultiQueueMultiThread {
    next_thread: AtomicUsize,
    num_threads: usize,
    senders: Vec<mpsc::Sender<Task>>,
    _workers: Vec<Worker>,
}

impl MultiQueueMultiThread {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);

        let mut _workers = Vec::with_capacity(num_threads);
        let mut senders = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let (sender, receiver) = mpsc::channel();
            let worker = Worker::new(receiver);
            senders.push(sender);
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

impl ThreadPool for MultiQueueMultiThread {
    fn execute<T>(&self, f: T)
    where
        T: FnOnce() + Send + 'static,
    {
        // get the index of the next thread and increment the counter
        let next_thread = self
            .next_thread
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
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
    fn new(receiver: mpsc::Receiver<Task>) -> Self {
        let _thread = thread::spawn({
            move || loop {
                let job = match receiver.recv() {
                    Ok(job) => job,
                    Err(_) => return,
                };

                job();
            }
        });
        Self { _thread }
    }
}
