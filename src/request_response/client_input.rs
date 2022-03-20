use std::borrow::Borrow;
use std::io::{Error, ErrorKind, Read, Write};

use crate::request_response::{command::Command, parsed_command::ParsedCommand, response_helper};
use crate::store::redis::{RedisStore, Store};
use crate::store::redis_operation::SetOptionalArgs;

pub struct ClientInput {
    input: String,
}

pub trait HandleClientInput {
    fn read_input<T: Read + Write>(
        &mut self,
        stream: &mut T,
    ) -> Result<Option<ParsedCommand>, Error>;

    fn respond<T: Write>(&self, stream: &mut T, parsed: ParsedCommand);

    fn reset(&mut self);
}

impl HandleClientInput for ClientInput {
    fn read_input<T: Read + Write>(
        &mut self,
        stream: &mut T,
    ) -> Result<Option<ParsedCommand>, Error> {
        let mut buffer: [u8; 1024] = [0; 1024];

        match stream.read(&mut buffer) {
            Ok(size) => {
                println!("Read {} bytes from input", size);

                if size == 0 {
                    return Err(Error::new(
                        ErrorKind::ConnectionAborted,
                        "Connection closed",
                    ));
                }

                let buffer_string = String::from_utf8_lossy(&buffer);
                self.append_input(&buffer_string);

                Ok(self.parse())
            }
            Err(e) => Err(e),
        }
    }

    fn respond<T: Write>(&self, stream: &mut T, parsed: ParsedCommand) {
        let args = parsed.args();
        let command = parsed.command();

        if command.is_none() {
            return response_helper::send_simple_string_response(stream, "Unrecognised command");
        }

        let command_unwrapped = command.as_ref().unwrap();

        if command_unwrapped == &Command::PING {
            response_helper::send_pong_response(stream);
        } else if command_unwrapped == &Command::ECHO {
            let mut result = String::from("");

            if args.is_none() {
                panic!("args is not set in ParsedCommand.")
            }

            for arg in args.as_ref().unwrap().iter() {
                let str: &str = arg.borrow();
                result.push_str(str);
            }
            response_helper::send_bulk_string_response(stream, Some(&result));
        } else if command_unwrapped == &Command::GET {
            let key_expired_for_get = self.get_key_and_expiry(args);
            if key_expired_for_get.is_none() {
                response_helper::send_bulk_string_response(stream, None);
                return;
            }

            let KeyValueExpiry {
                key,
                value,
                is_expired,
            } = key_expired_for_get.unwrap();

            if is_expired {
                self.delete_expired_keys(vec![&key]);
                response_helper::send_bulk_string_response(stream, None);
            } else {
                let value = value.as_ref().map(|v| &**v);
                response_helper::send_bulk_string_response(stream, value);
            }

        } else if command_unwrapped == &Command::SET {
            let arguments = args.as_ref().unwrap();
            let store = &mut RedisStore::get_store();

            // set <key> <value> [ex seconds | px milliseconds]
            let key = arguments.get(0).unwrap();
            let value = arguments.get(1).unwrap();
            let optional_args = self.determine_set_optional_args(arguments);

            store.set(key, value, &optional_args);
            response_helper::send_bulk_string_response(stream, Some("OK"));
        }
    }

    fn reset(&mut self) {
        self.input = String::from("");
    }
}

impl ClientInput {
    pub fn new() -> ClientInput {
        ClientInput {
            input: String::from(""),
        }
    }

    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }

    fn append_input(&mut self, input: &str) {
        self.input.push_str(input.replace("\0", "").as_str());
    }

    /// Format: *{number of arguments}\r\n${number of bytes}\r\n${number of bytes}\r\n...
    /// https://redis.io/topics/protocol#sending-commands-to-a-redis-server
    fn parse(&self) -> Option<ParsedCommand> {
        let input = self.input.as_str();

        if input.len() == 0 {
            return None;
        }

        if &input[0..1] != "*" {
            println!("Unrecognised command!");
            None
        } else {
            let num_args_ref: &u8 = &input[1..2].parse().unwrap();
            let num_args: u8 = num_args_ref.clone();

            let mut string_split = self.parse_string_into_vector(&self.input);

            if self.has_complete_input(&string_split, num_args) {
                let command_str = string_split.remove(0);
                let command = Command::from(&command_str);

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

    // TODO: Parsing method can be improved
    fn parse_string_into_vector(&self, input: &str) -> Vec<String> {
        let mut string_split: Vec<String> = input
            .replace("\\r\\n", "\n")
            .split("\n")
            .map(|s| String::from(s.trim()))
            .filter(|s| !s.is_empty())
            .collect();

        string_split = string_split
            .iter()
            .skip(2)       // Discard number of elements(e.g.. *2) and number of bytes in command(e.g. $4)
            .step_by(2) // Discard the number of bytes for each bulk string after command, i.e. ${number of bytes}
            .map(String::from)
            .collect();

        string_split
    }

    fn has_complete_input(&self, string_split: &Vec<String>, num_args: u8) -> bool {
        string_split.len() as u8 == num_args
    }

    fn determine_set_optional_args(&self, arguments: &Vec<String>) -> Option<SetOptionalArgs> {
        let mut optional_args: Option<SetOptionalArgs> = None;

        if arguments.len() != 4 {
            return optional_args;
        }

        let variant = arguments.get(2).unwrap();
        let duration = arguments.get(3).unwrap();
        let mut duration_ms: u64 = 0;

        // if variant is "ex", duration is in seconds
        // if variant is "px", duration is in milliseconds
        if variant.to_lowercase() == "ex" || variant.to_lowercase() == "px" {
            duration_ms = match duration.parse() {
                Ok(d) => d,
                Err(e) => {
                    println!("Error parsing duration: {}", e);
                    0
                }
            };

            if variant.to_lowercase() == "ex" {
                duration_ms *= 1000;
            }
        }
        if duration_ms != 0 {
            optional_args = Some(SetOptionalArgs {
                expire_in_ms: Some(duration_ms),
            });
        }
        optional_args
    }

    fn delete_expired_keys(&self, keys: Vec<&str>) {
        let store = &mut RedisStore::get_store();
        store.delete(keys);
    }

    fn get_key_and_expiry(
        &self,
        args: &Option<Vec<String>>,
    ) -> Option<KeyValueExpiry> {
        let store = RedisStore::get_store();
        let key = args.as_ref().unwrap().get(0).unwrap();
        let value = store.get(key.as_str());

        let is_expired = store.is_key_expired(key);
        Some(KeyValueExpiry {
            key: String::from(key),
            value: value.map(String::from),
            is_expired,
        })
    }
}

#[derive(Debug)]
struct KeyValueExpiry {
    key: String,
    value: Option<String>,
    is_expired: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_input_correctly() {
        let mut client_input = ClientInput::new();
        client_input.append_input("");
        assert_eq!(client_input.get_input(), "");

        client_input.append_input("hello");
        assert_eq!(client_input.get_input(), "hello");

        client_input.append_input(" world");
        assert_eq!(client_input.get_input(), "hello world");
    }

    #[test]
    fn reset_correctly() {
        let mut client_input = ClientInput::new();
        client_input.append_input("hello world");

        client_input.reset();
        assert_eq!(client_input.get_input(), "");
    }

    #[test]
    fn parse_ping_command_correctly() {
        let input = "*1\\r\\n$4\\r\\nPING\\r\\n";

        let mut client_input = ClientInput::new();
        client_input.append_input(input);

        let parsed = match client_input.parse() {
            Some(p) => p,
            None => panic!("Client input should be parsed"),
        };

        assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 1);
        assert_eq!(parsed.command().as_ref().unwrap(), &Command::PING);
        assert_eq!(parsed.args().as_ref().unwrap(), &vec![] as &Vec<String>);
    }

    #[test]
    fn parse_echo_command_correctly() {
        let input = "*3\\r\\n$4\\r\\nECHO\\r\\n$5\\r\\nhello\\r\\n$5\\r\\nworld\\r\\n";

        let mut client_input = ClientInput::new();
        client_input.append_input(input);

        let parsed = match client_input.parse() {
            Some(p) => p,
            None => panic!("Client input should be parsed"),
        };

        assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 3);
        assert_eq!(parsed.command().as_ref().unwrap(), &Command::ECHO);
        assert_eq!(
            parsed.args().as_ref().unwrap(),
            &vec![String::from("hello"), String::from("world")]
        );
    }

    #[test]
    fn parse_get_command_correctly() {
        let input = "*2\\r\\n$3\\r\\nGET\\r\\n$5\\r\\nhello\\r\\n";

        let mut client_input = ClientInput::new();
        client_input.append_input(input);

        let parsed = match client_input.parse() {
            Some(p) => p,
            None => panic!("Client input should be parsed"),
        };

        assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 2);
        assert_eq!(parsed.command().as_ref().unwrap(), &Command::GET);
        assert_eq!(
            parsed.args().as_ref().unwrap(),
            &vec![String::from("hello")]
        );
    }

    #[test]
    fn determine_set_optional_args_return_some_when_expiry_args_are_present() {
        let mut client_input = ClientInput::new();
        let args = vec![
            String::from("hello"),
            String::from("world"),
            String::from("px"),
            String::from("1000"),
        ];

        let set_args = client_input.determine_set_optional_args(&args);
        assert!(set_args.is_some());

        let args = set_args.unwrap();
        assert!(args.expire_in_ms.is_some());
        assert_eq!(args.expire_in_ms.unwrap(), 1000);
    }

    #[test]
    fn determine_set_optional_args_return_none_when_expiry_args_are_not_present() {
        let mut client_input = ClientInput::new();
        let input = vec![
            vec![String::from("hello"), String::from("world")],
            vec![
                String::from("hello"),
                String::from("world"),
                String::from("px"),
            ],
            vec![
                String::from("hello"),
                String::from("world"),
                String::from("px"),
                String::from("px"),
            ],
        ];

        for i in input {
            let set_args = client_input.determine_set_optional_args(&i);
            assert!(set_args.is_none());
        }
    }
}
