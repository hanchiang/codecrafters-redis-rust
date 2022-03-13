use std::borrow::Borrow;
use std::io::Read;
use std::net::TcpStream;

use crate::request_response::{command::Command, parsed_command::ParsedCommand};

pub struct ClientInput {
    input: String,
}

impl ClientInput {
    pub fn new() -> ClientInput {
        ClientInput {
            input: String::from(""),
        }
    }

    pub fn read_input(&mut self, mut stream: &TcpStream) -> Option<ParsedCommand> {
        let mut buffer: [u8; 1024] = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(size) => {
                println!("Read {} bytes from input", size);
                let buffer_string = String::from_utf8_lossy(&buffer);
                self.append_input(&buffer_string);

                self.parse()
            },
            Err(e) => {
                println!("Unable to read input: {}", e);
                None
            }
        }

    }

    pub fn reset(&mut self) {
        self.input = String::from("");
    }

    fn append_input(&mut self, input: &str) {
        println!("received input: {}", input.replace("\0", "").as_str());
        self.input.push_str(input.replace("\0", "").as_str());
    }

    /// Format: *{number of arguments}\r\n${number of bytes}\r\n${number of bytes}\r\n...
    /// https://redis.io/topics/protocol#sending-commands-to-a-redis-server
    pub fn parse(&self) -> Option<ParsedCommand> {
        let input: &str = self.input.borrow();
        if &input[0..1] != "*" {
            println!("Unrecognised command!");
            None
        } else {
            let num_args_ref: &u8 = &input[1..2].parse().unwrap();
            let num_args: u8 = num_args_ref.clone();
            println!("num args: {}", num_args);

            let input: &str = self.input.borrow();
            let mut string_split: Vec<String> = input
                .replace("\\r\\n", "\n")
                .split("\n")
                .filter(|s| !s.is_empty())
                .map(|s| {
                    String::from(s.trim())
                })
                .collect();

            for s in string_split.iter() {
                println!("string_split before: {}", s);
            }

            string_split = string_split
                .iter()
                .skip(2)
                .step_by(2)
                .map(String::from)
                .collect();

            for s in string_split.iter() {
                println!("string_split after {}", s);
            }

            if string_split.len() as u8 == num_args {
                let command_str = string_split.remove(0);
                let mut command: Option<Command> = None;
                if command_str.to_lowercase() == "echo" {
                    command = Some(Command::ECHO);
                } else if command_str.to_lowercase() == "ping" {
                    command = Some(Command::PING);
                }

                let mut parsed = ParsedCommand::new();
                parsed.set_args(Some(string_split));
                parsed.set_num_args_in_input(Some(num_args.clone()));
                parsed.set_command(command);

                Some(parsed)
            } else {
                None
            }
        }
    }
}