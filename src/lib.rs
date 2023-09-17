use std::convert::TryInto;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;

pub mod request_response;
pub mod store;
pub mod parser;

use crate::request_response::client_input::HandleClientInput;
use request_response::client_input::ClientInput;
use crate::parser::parser::{ParseError, RESPOutput};
use crate::request_response::command::Command;
use crate::request_response::parsed_command::ParsedCommand;

#[derive(Debug, PartialEq)]
pub enum AppError {
    ConnectionClosed(String),
    ParseError(String),
    IncompleteInput(String),
    Error(String),
}

impl From<ParseError> for AppError {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::InvalidInput => AppError::ParseError(String::from("Invalid input")),
            ParseError::CRLFNotFound => AppError::ParseError(String::from("CRLF is not found")),
            ParseError::UnrecognisedSymbol => AppError::ParseError(String::from("Unrecognised symbol")),
            ParseError::IncompleteInput => AppError::IncompleteInput(String::from("Incomplete input"))
        }
    }
}

// Receives TcpStream so that we can use its methods
// TODO: ideally should respond to valid inputs here as well
pub fn handle_connection(mut stream: TcpStream) {
    let mut client_input = ClientInput::new();
    loop {
        let result = handle_connection_helper(&mut stream, &mut client_input);
        if result.is_err() {
            let error = result.unwrap_err();
            if let AppError::IncompleteInput(_) = error {
                println!("Incomplete input. Waiting for more input.");
                continue;
            }

            println!("Error: {:?}", error);

            match error {
                AppError::ParseError(e)  | AppError::Error(e) => {
                    client_input.respond_error(&mut stream, e.as_str());
                    break;
                },
                AppError::ConnectionClosed(_) => { break; },
                _ => {}
            }
        }
    }
}

// From this function onwards, it receives only the relevant trait bound so that it can be swapped
// with a stub during tests
pub fn handle_connection_helper<T: Read + Write + Send>(mut stream: T, client_input: &mut ClientInput) -> Result<(), AppError> {
    let mut buffer: [u8; 1024] = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("Read {} bytes from input", size);

            if size == 0 {
                return Err(AppError::ConnectionClosed(String::from("Connection closed")));
            }

            let parsed = client_input.parse_input(&buffer[..size])?;
            // Move these things below into another file
            let parsed_command = resp_output_to_parsed_command(&parsed);

            client_input.respond(&mut stream, parsed_command);
            client_input.reset();
            Ok(())
        }
        // TODO: test
        Err(e) => Err(AppError::Error(e.to_string())),
    }
}

pub fn resp_output_to_parsed_command(resp_output: &RESPOutput) -> ParsedCommand {
    let mut parsed_command = ParsedCommand::new();
    // Client should only send an array of bulk string

    if let RESPOutput::Array(arr) = resp_output {
        let command_resp = &arr[0];
        let args_resp = &arr[1..];

        println!("command_resp: {:?}, args_resp: {:?}", command_resp, args_resp);

        if let RESPOutput::BulkString(command) = command_resp {
            if command.to_lowercase() == "ping" {
                parsed_command.set_command(Some(Command::PING))
            } else if command.to_lowercase() == "echo" {
                parsed_command.set_command(Some(Command::ECHO))
            } else if command.to_lowercase() == "get" {
                parsed_command.set_command(Some(Command::GET))
            } else if command.to_lowercase() == "set" {
                parsed_command.set_command(Some(Command::SET))
            }
        }

        for arg_resp in args_resp {
            if let RESPOutput::BulkString(arg) = arg_resp {
                parsed_command.append_arg(String::from(arg));
            }
        }
    }
    parsed_command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resp_output_to_parsed_command_success() {
        let input = vec![
            RESPOutput::Array(
                vec![
                    RESPOutput::BulkString(String::from("ping"))
                ]
            ),
            RESPOutput::Array(
                vec![
                    RESPOutput::BulkString(String::from("echo")),
                    RESPOutput::BulkString(String::from("hello"))
                ]
            ),
            RESPOutput::Array(
                vec![
                    RESPOutput::BulkString(String::from("set")),
                    RESPOutput::BulkString(String::from("hello")),
                    RESPOutput::BulkString(String::from("world"))
                ]
            ),
            RESPOutput::Array(
                vec![
                    RESPOutput::BulkString(String::from("get")),
                    RESPOutput::BulkString(String::from("hello"))
                ]
            ),
        ];

        let expected = vec![
            ParsedCommand {
                command: Some(Command::PING),
                args: Vec::new()
            },
            ParsedCommand {
                command: Some(Command::ECHO),
                args: vec![String::from("hello")]
            },
            ParsedCommand {
                command: Some(Command::SET),
                args: vec![
                    String::from("hello"),
                    String::from("world")
                ]
            },
            ParsedCommand {
                command: Some(Command::GET),
                args: vec![
                    String::from("hello")
                ]
            }
        ];

        for (index, inp) in input.iter().enumerate() {
            let res = resp_output_to_parsed_command(inp);
            assert_eq!(res, expected[index]);
        }
    }
}