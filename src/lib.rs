mod queue;

use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub trait ThreadPool {
    fn execute<T>(&self, f: T)
    where
        T: FnOnce() + Send + 'static;
}

pub fn run_server<T>(pool: T)
where
    T: ThreadPool,
{
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
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
