extern crate core;

#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::thread;

use redis_starter_rust::handle_connection;
use redis_starter_rust::store::redis::{RedisStore, Store};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    RedisStore::initialise();

    for wrapped_stream in listener.incoming() {
        let stream = wrapped_stream.unwrap();
        thread::spawn(move || handle_connection(stream));
    }
}
