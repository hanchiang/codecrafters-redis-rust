use std::io::{Error, Read, Write};

pub mod request_response;

use crate::request_response::client_input::HandleClientInput;
use request_response::client_input::ClientInput;

pub fn handle_connection<T: Read + Write + Send>(mut stream: T) {
    let mut client_input = ClientInput::new();
    loop {
        let result = handle_connection_helper(&mut stream, &mut client_input);
        if result.is_err() {
            break;
        }
    }
}

pub fn handle_connection_helper<T: Read + Write + Send>(mut stream: T, client_input: &mut ClientInput) -> Result<(), Error> {
    let parsed = client_input.read_input(&mut stream);
    if parsed.is_err() {
        let err = parsed.unwrap_err();
        println!("Unable to read input: {}", err);
        return Err(err);
    }

    match parsed.unwrap() {
        Some(parsed) => {
            println!("Completed reading input: {:#?}", parsed);
            client_input.respond(&mut stream, parsed);
            client_input.reset();
            Ok(())
        }
        None => {
            println!("Input is incomplete. Waiting for further input");
            Ok(())
        }
    }
}