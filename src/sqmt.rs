use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{Task, ThreadPool};

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
        for i in 0..num_threads {
            _workers.push(Worker::new(i, Arc::clone(&receiver)));
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
    _id: usize,
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let _thread = thread::spawn({
            move || loop {
                let job = match receiver.lock().unwrap().recv() {
                    Ok(job) => job,
                    Err(_) => {
                        println!("Shutting down thread {id}");
                        return;
                    }
                };

                job();
            }
        });

        Self { _id: id, _thread }
    }
}
