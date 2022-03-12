#[allow(unused_imports)]
use std::env;
use std::error::Error;
#[allow(unused_imports)]
use std::fs;
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::thread;
use tokio::io::AsyncReadExt;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {
    thread::spawn(move || {
        let mut buffer: [u8; 1024] = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(_size) => {
                    let res = format_response("PONG");
                    stream.write(res.as_bytes());
                }
                Err(e) => {
                    println!("Error reading from stream: {:?}", e);
                    break;
                }
            }
        }
    });
}

fn format_response(res: &str) -> String {
    format!("+{}\r\n", res)
}
