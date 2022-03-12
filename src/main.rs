#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut buffer: [u8; 1024] = [0; 1024];

                loop {
                    match s.read(&mut buffer) {
                        Ok(_size) => {
                            let res = format_response("PONG");
                            s.write(res.as_bytes());
                        },
                        Err(e) => {
                            println!("Error reading from stream: {:?}", e);
                            break;
                        }
                    }
                }
            },
            Err(e) => println!("Couldn't accept client: {:?}", e)
        }
    }
}

fn format_response(res: &str) -> String {
    format!("+{}\r\n", res)
}
