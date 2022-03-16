use std::io::{Read, Write};
use std::thread;
use std::thread::JoinHandle;

pub mod request_response;

use crate::request_response::client_input::HandleClientInput;
use request_response::client_input::ClientInput;

// TODO: How to test this?
pub fn handle_connection<
    T: Read + Write + Send + 'static,
>(
    mut stream: T
) -> JoinHandle<()> {
    let mut client_input = ClientInput::new();
    thread::spawn(move || loop {
        let parsed = client_input.read_input(&mut stream);

        if parsed.is_err() {
            println!("Unable to read input: {}", parsed.unwrap_err());
            break;
        }

        match parsed.unwrap() {
            Some(parsed) => {
                println!("Completed reading input: {:#?}", parsed);
                client_input.respond(&mut stream, parsed);
                client_input.reset();
            }
            None => {
                println!("Input is incomplete. Waiting for further input");
            }
        }
    })
}
