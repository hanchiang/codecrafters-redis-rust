use crate::request_response::{command::Command};

#[derive(Debug)]
pub struct ParsedCommand {
    // sample input: *2\r\n$4\r\necho\r\n$5\r\nhello\r\n
    // Number of arguments as given from input. The digit after the first character '*'
    num_args_in_input: Option<u8>,
    // i.e. "echo"
    command: Option<Command>,
    // contains the non-byte count arguments after the command
    // args will be ["hello"]
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
}