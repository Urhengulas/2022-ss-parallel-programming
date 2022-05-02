use std::{sync::mpsc, thread};

use krate::{run_server, Task, ThreadPool};

fn main() {
    let pool = SingleQueueSingleThread::new();
    run_server(pool);
}

pub struct SingleQueueSingleThread {
    sender: mpsc::Sender<Task>,
    _worker: Worker,
}

impl SingleQueueSingleThread {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let _worker = Worker::new(receiver);
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
