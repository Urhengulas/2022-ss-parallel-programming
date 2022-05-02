use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use krate::{run_server, Task, ThreadPool};

fn main() {
    let pool = SingleQueueMultiThread::new(4);
    run_server(pool);
}

pub struct SingleQueueMultiThread {
    sender: mpsc::Sender<Task>,
    _workers: Vec<Worker>,
}

impl SingleQueueMultiThread {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut _workers = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {
            _workers.push(Worker::new(Arc::clone(&receiver)));
        }
        Self { sender, _workers }
    }
}

impl ThreadPool for SingleQueueMultiThread {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap()
    }
}

struct Worker {
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let _thread = thread::spawn({
            move || loop {
                let job = match receiver.lock().unwrap().recv() {
                    Ok(job) => job,
                    Err(_) => return,
                };
                job();
            }
        });
        Self { _thread }
    }
}
