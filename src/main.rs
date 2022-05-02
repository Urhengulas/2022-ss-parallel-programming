mod multi_queue_multi_thread;
mod queue;
mod single_queue_multi_thread;
mod single_queue_single_thread;
mod work_stealing;

use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

pub use crate::{
    multi_queue_multi_thread::MultiQueueMultiThread, queue::Queue,
    single_queue_multi_thread::SingleQueueMultiThread,
    single_queue_single_thread::SingleQueueSingleThread, work_stealing::WorkStealing,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = MultiQueueMultiThread::new(6);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let num = ascii_to_u64(buf[5]);
    let result = fibonacci(num * 5);
    let contents = result.to_string();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn ascii_to_u64(ascii: u8) -> u64 {
    (ascii as char).to_digit(10).unwrap() as u64
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
