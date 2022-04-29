mod task_queue;
mod thread_pool;

use std::{thread, time::Duration};

pub use crate::{task_queue::TaskQueue, thread_pool::ThreadPool};

fn main() {
    let pool = ThreadPool::new(4);
    for i in 0..20 {
        pool.execute(move || {
            let _ = 1 + i;
        });
    }
    thread::sleep(Duration::from_secs(2));
}
