use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
};

pub struct ThreadPool {
    next_thread: AtomicUsize,
    num_threads: usize,
    senders: Vec<mpsc::Sender<Job>>,
    _workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);

        let mut _workers = Vec::with_capacity(num_threads);
        let mut senders = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            let (sender, receiver) = mpsc::channel();
            let worker = Worker::new(i, receiver);
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

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // get the index of the next thread and increment the counter
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
    _id: usize,
    _thread: thread::JoinHandle<()>,
    is_working: Arc<AtomicBool>,
}

impl Worker {
    fn new(id: usize, receiver: mpsc::Receiver<Job>) -> Self {
        let is_working = Arc::new(AtomicBool::new(false));

        let _thread = thread::spawn({
            let is_working = Arc::clone(&is_working);
            move || loop {
                let job = match receiver.recv() {
                    Ok(job) => job,
                    Err(_) => return,
                };

                is_working.store(true, Ordering::SeqCst);
                job();
                is_working.store(false, Ordering::SeqCst);
            }
        });

        Self {
            _id: id,
            _thread,
            is_working,
        }
    }

    fn is_working(&self) -> bool {
        self.is_working.load(Ordering::SeqCst)
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn is_working_is_working() {
        // Arrange
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let worker = Worker::new(0, rx);

        // Act
        let before = worker.is_working();
        tx.send(Box::new(|| thread::sleep(Duration::from_secs(2))))
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        let during = worker.is_working();
        thread::sleep(Duration::from_secs(2));
        let after = worker.is_working();

        // Assert
        assert_eq!(before, false);
        assert_eq!(during, true);
        assert_eq!(after, false);
    }
}
