extern crate core;

#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::thread;

mod request_response;
use request_response::client_input::ClientInput;

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
        let mut client_input = ClientInput::new();

        loop {
            let parsed = client_input.read_input(&mut stream);

            if parsed.is_err() {
                println!("Unable to read input: {}", parsed.unwrap_err());
                break;
            }

            match parsed.unwrap() {
                Some(p) => {
                    println!("Completed reading input: {:#?}", p);
                    p.respond(&stream);
                    client_input.reset();
                }
                None => {
                    println!("Input is incomplete. Waiting for further input");
                }
            }
        }
    });
}
