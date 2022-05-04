use serial_test::serial;

use std::borrow::Borrow;

use std::thread;

use redis_starter_rust::request_response::{client_input::ClientInput, command::Command};
use redis_starter_rust::request_response::client_input::HandleClientInput;
use redis_starter_rust::request_response::parsed_command::ParsedCommand;

mod mock;
use mock::tcp_stream::mock_tcp_stream::MockTcpStream;

use mock::common::reset_redis::with_reset_redis;
use redis_starter_rust::parser::parser::RESPOutput;
use redis_starter_rust::store::redis::{RedisStore, Store};
use redis_starter_rust::store::redis_operation::SetOptionalArgs;

#[test]
fn parse_input_success() {
    let mut client_input = ClientInput::new();
    let parsed = client_input.parse_input("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes());
    let expected = RESPOutput::Array(vec!(
        RESPOutput::BulkString(String::from("hello")),
        RESPOutput::BulkString(String::from("world"))
    ));

    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), expected);
}

#[test]
fn respond_return_unrecognised_command_if_command_is_none() {
    let mut parsed = ParsedCommand::new();
    parsed.set_command(None);
    parsed.set_args(Vec::new());

    let mut mock_tcp_stream = MockTcpStream::new();
    let client_input = ClientInput::new();
    client_input.respond(&mut mock_tcp_stream, parsed);

    let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
    let expected_bytes = "-Unrecognised command\r\n".as_bytes();

    assert_eq!(written_bytes, expected_bytes);
}

#[test]
fn respond_return_pong_if_command_is_ping() {
    let mut parsed = ParsedCommand::new();
    parsed.set_command(Some(Command::PING));
    parsed.set_args(Vec::new());

    let mut mock_tcp_stream = MockTcpStream::new();
    let client_input = ClientInput::new();
    client_input.respond(&mut mock_tcp_stream, parsed);

    let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
    let expected_bytes = "+PONG\r\n".as_bytes();

    assert_eq!(written_bytes, expected_bytes);
}

#[test]
fn respond_return_input_if_command_is_echo() {
    let mut parsed = ParsedCommand::new();
    parsed.set_command(Some(Command::ECHO));
    parsed.set_args(vec![String::from("hello "), String::from("world")]);

    let mut mock_tcp_stream = MockTcpStream::new();
    let client_input = ClientInput::new();
    client_input.respond(&mut mock_tcp_stream, parsed);

    let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
    let expected_bytes = "$11\r\nhello world\r\n".as_bytes();

    assert_eq!(written_bytes, expected_bytes);
}

#[test]
#[serial]
fn respond_return_none_if_command_is_get_and_key_is_not_set() {
    with_reset_redis(|| {
        RedisStore::initialise();

        let mut parsed = ParsedCommand::new();
        parsed.set_command(Some(Command::GET));
        parsed.set_args(vec![String::from("hello")]);

        let mut mock_tcp_stream = MockTcpStream::new();
        let client_input = ClientInput::new();
        client_input.respond(&mut mock_tcp_stream, parsed);

        let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
        let expected_bytes = "$-1\r\n".as_bytes();

        assert_eq!(written_bytes, expected_bytes);
    });
}

#[test]
#[serial]
fn respond_return_data_if_command_is_get_and_key_is_set_without_expiry() {
    with_reset_redis(|| {
        RedisStore::initialise();

        let mut parsed = ParsedCommand::new();
        let key = "hello";
        let value = "world";
        parsed.set_command(Some(Command::GET));
        parsed.set_args(vec![String::from(key)]);

        {
            let store = RedisStore::get_store();
            store.set(key, value, &None);
        }

        let mut mock_tcp_stream = MockTcpStream::new();
        let client_input = ClientInput::new();
        client_input.respond(&mut mock_tcp_stream, parsed);

        let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
        let expected_bytes = "$5\r\nworld\r\n".as_bytes();

        assert_eq!(written_bytes, expected_bytes);
    });
}

#[test]
#[serial]
fn respond_return_data_if_command_is_get_and_key_is_set_with_expiry() {
    with_reset_redis(|| {
        RedisStore::initialise();

        let mut parsed = ParsedCommand::new();
        let key = "hello";
        let value = "world";
        let duration = 50;
        parsed.set_command(Some(Command::GET));
        parsed.set_args(vec![String::from(key)]);

        {
            let store = RedisStore::get_store();
            let set_args = Some(SetOptionalArgs {
                expire_in_ms: Some(duration)
            });
            store.set(key, value, &set_args);
        }

        // Key should not be expired
        let mut mock_tcp_stream = MockTcpStream::new();
        let client_input = ClientInput::new();
        client_input.respond(&mut mock_tcp_stream, parsed);

        let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
        let expected_bytes = "$5\r\nworld\r\n".as_bytes();
        assert_eq!(written_bytes, expected_bytes);


        // Key should be expired
        let mut parsed = ParsedCommand::new();
        let key = "hello";
        let _value = "world";
        let duration = 50;

        parsed.set_command(Some(Command::GET));
        parsed.set_args(vec![String::from(key)]);

        thread::sleep(std::time::Duration::from_millis(duration));
        client_input.respond(&mut mock_tcp_stream, parsed);

        let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
        let expected_bytes = "$-1\r\n".as_bytes();
        assert_eq!(written_bytes, expected_bytes);
    });
}
