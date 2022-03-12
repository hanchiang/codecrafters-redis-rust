extern crate core;

use std::borrow::Borrow;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::string::FromUtf8Error;
use std::thread;

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
            let parsed = client_input.read_until_input_complete(&mut stream);

            match parsed {
                Some(p) => {
                    println!("Completed reading input: {:#?}", p);
                    p.respond(&stream);
                }
                None => {
                    println!("Input is incomplete. Waiting for further input");
                }
            }
            client_input.reset();
        }
    });
}



#[derive(Debug, PartialEq)]
enum Command {
    PING,
    ECHO,
}

struct ClientInput {
    input: String,
}

impl ClientInput {
    fn new() -> ClientInput {
        ClientInput {
            input: String::from(""),
        }
    }

    fn read_until_input_complete(&mut self, mut stream: &TcpStream) -> Option<ParsedCommand> {
        let mut buffer: [u8; 1024] = [0; 1024];
        stream.read(&mut buffer);
        let buffer_string = String::from_utf8_lossy(&buffer);
        self.append_input(&buffer_string);

        self.parse()
    }

    fn reset(&mut self) {
        self.input = String::from("");
    }

    fn append_input(&mut self, input: &str) {
        println!("received input: {}", input.replace("\0", "").as_str());
        self.input.push_str(input.replace("\0", "").as_str());
    }

    /// Format: *{number of arguments}\r\n${number of bytes}\r\n${number of bytes}\r\n...
    /// https://redis.io/topics/protocol#sending-commands-to-a-redis-server
    fn parse(&self) -> Option<ParsedCommand> {
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

            if (string_split.len() as u8 == num_args) {
                let command_str = string_split.remove(0);
                let mut command: Option<Command> = None;
                if command_str.to_lowercase() == "echo" {
                    command = Some(Command::ECHO);
                } else if command_str.to_lowercase() == "ping" {
                    command = Some(Command::PING);
                }

                Some(ParsedCommand {
                    num_args_in_input: Some(num_args.clone()),
                    args: Some(string_split),
                    command,
                })
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
struct ParsedCommand {
    // number of arguments as given from input, not the length of the args field
    num_args_in_input: Option<u8>,
    command: Option<Command>,
    args: Option<Vec<String>>,
}

impl ParsedCommand {
    fn new() -> ParsedCommand {
        ParsedCommand {
            num_args_in_input: None,
            command: None,
            args: None,
        }
    }

    fn respond(self, stream: &TcpStream) {
        let ParsedCommand {
            args,
            command,
            num_args_in_input,
        } = self;

        match command {
            Some(command) => {
                if command == Command::PING {
                    send_pong_response(stream);
                } else if command == Command::ECHO {
                    let mut result = String::from("");

                    for arg in args.unwrap().iter() {
                        let str: &str = arg.borrow();
                        result.push_str(str);
                    }
                    send_bulk_string_response(stream, result);
                }
            },
            None => send_simple_string_response(&stream, "Unrecognised command")
        };
    }
}

fn send_null_bulk_string_response(stream: &TcpStream) {
    send_bulk_string_response(stream, String::from("-1"));
}

fn send_bulk_string_response(mut stream: &TcpStream, data: String) {
    let response = format!("${}\r\n{}\r\n", data.len(), data);
    stream.write(response.as_bytes());
}

fn send_pong_response(mut stream: &TcpStream) {
    let res = format_string_response("PONG");
    stream.write(res.as_bytes());
}

fn send_simple_string_response(mut stream: &TcpStream, str: &str) {
    stream.write(format_string_response(str).as_bytes());
}

fn format_string_response(res: &str) -> String {
    format!("+{}\r\n", res)
}
