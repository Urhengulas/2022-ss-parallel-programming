use std::{sync::mpsc, thread};

use crate::{Task, ThreadPool};

pub struct SingleQueueSingleThread {
    sender: mpsc::Sender<Task>,
    _worker: Worker,
}

impl SingleQueueSingleThread {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let _worker = Worker::new(0, receiver);
        Self { sender, _worker }
    }
}

impl ThreadPool for SingleQueueSingleThread {
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
    fn new(id: usize, receiver: mpsc::Receiver<Task>) -> Self {
        let _thread = thread::spawn({
            move || loop {
                let job = match receiver.recv() {
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
