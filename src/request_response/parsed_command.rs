use std::borrow::Borrow;
use std::net::TcpStream;

use crate::request_response::{command::Command, response_helper};

#[derive(Debug)]
pub struct ParsedCommand {
    // number of arguments as given from input, not the length of the args field
    num_args_in_input: Option<u8>,
    command: Option<Command>,
    args: Option<Vec<String>>,
}

impl ParsedCommand {
    pub fn new() -> ParsedCommand {
        ParsedCommand {
            num_args_in_input: None,
            command: None,
            args: None,
        }
    }
    pub fn num_args_in_input(&self) -> Option<u8> {
        self.num_args_in_input
    }
    pub fn command(&self) -> &Option<Command> {
        &self.command
    }
    pub fn args(&self) -> &Option<Vec<String>> {
        &self.args
    }

    pub fn set_num_args_in_input(&mut self, num_args_in_input: Option<u8>) {
        self.num_args_in_input = num_args_in_input;
    }
    pub fn set_command(&mut self, command: Option<Command>) {
        self.command = command;
    }
    pub fn set_args(&mut self, args: Option<Vec<String>>) {
        self.args = args;
    }

    pub fn respond(self, stream: &TcpStream) {
        let ParsedCommand {
            args,
            command,
            num_args_in_input,
        } = self;

        match command {
            Some(command) => {
                if command == Command::PING {
                    response_helper::send_pong_response(stream);
                } else if command == Command::ECHO {
                    let mut result = String::from("");

                    for arg in args.unwrap().iter() {
                        let str: &str = arg.borrow();
                        result.push_str(str);
                    }
                    response_helper::send_bulk_string_response(stream, result);
                }
            },
            None => response_helper::send_simple_string_response(&stream, "Unrecognised command")
        };
    }
}