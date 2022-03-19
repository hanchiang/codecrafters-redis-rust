use serial_test::serial;

use std::borrow::Borrow;
use std::io::{ErrorKind};

use std::thread;

use redis_starter_rust::request_response::{client_input::ClientInput, command::Command};
use redis_starter_rust::request_response::client_input::HandleClientInput;
use redis_starter_rust::request_response::parsed_command::ParsedCommand;

mod mock;
use mock::tcp_stream::mock_tcp_stream::MockTcpStream;
use mock::tcp_stream::mock_tcp_stream_read_error::MockTcpStreamStreamReadError;

use mock::common::mock_input::{generate_ping_buffer, generate_echo_buffer, generate_get_buffer};
use mock::common::reset_redis::with_reset_redis;
use redis_starter_rust::store::redis::{RedisStore, Store};
use redis_starter_rust::store::redis_operation::SetOptionalArgs;

#[test]
fn read_input_helper_return_err_if_no_data_is_read() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();

    let parsed = client_input.read_input(&mut mock_tcp_stream);
    assert!(parsed.is_err());

    let error = parsed.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::ConnectionAborted);
}

#[test]
fn read_input_helper_return_err_if_read_returns_error() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStreamStreamReadError::new();

    let parsed = client_input.read_input(&mut mock_tcp_stream);
    assert!(parsed.is_err());

    let error = parsed.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
}

#[test]
fn read_input_helper_read_ping_command_correctly() {
    let buffer = generate_ping_buffer();

    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = buffer.to_vec();

    let parsed_result = client_input.read_input(&mut mock_tcp_stream);
    let parsed_option = match parsed_result {
        Ok(p) => p,
        Err(e) => panic!("{}", e),
    };

    let parsed = parsed_option.unwrap();

    assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 1);
    assert_eq!(parsed.command().as_ref().unwrap(), &Command::PING);
    assert_eq!(parsed.args().as_ref().unwrap(), &vec![] as &Vec<String>);
}

#[test]
fn read_input_helper_read_echo_command_correctly() {
    let buffer = generate_echo_buffer();

    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = buffer.to_vec();

    let parsed_result = client_input.read_input(&mut mock_tcp_stream);
    let parsed_option = match parsed_result {
        Ok(p) => p,
        Err(e) => panic!("{}", e),
    };

    let parsed = parsed_option.unwrap();

    assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 3);
    assert_eq!(parsed.command().as_ref().unwrap(), &Command::ECHO);
    assert_eq!(
        parsed.args().as_ref().unwrap(),
        &vec![String::from("hello"), String::from("world")]
    );
}

#[test]
fn read_input_helper_read_get_command_correctly() {
    let buffer = generate_get_buffer();

    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = buffer.to_vec();

    let parsed_result = client_input.read_input(&mut mock_tcp_stream);
    let parsed_option = match parsed_result {
        Ok(p) => p,
        Err(e) => panic!("{}", e),
    };

    let parsed = parsed_option.unwrap();

    assert_eq!(*parsed.num_args_in_input().as_ref().unwrap(), 2);
    assert_eq!(parsed.command().as_ref().unwrap(), &Command::GET);
    assert_eq!(
        parsed.args().as_ref().unwrap(),
        &vec![String::from("hello")]
    );
}

#[test]
fn respond_return_unrecognised_command_if_command_is_none() {
    let mut parsed = ParsedCommand::new();
    parsed.set_command(None);
    parsed.set_args(None);
    parsed.set_num_args_in_input(None);

    let mut mock_tcp_stream = MockTcpStream::new();
    let client_input = ClientInput::new();
    client_input.respond(&mut mock_tcp_stream, parsed);

    let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
    let expected_bytes = "+Unrecognised command\r\n".as_bytes();

    assert_eq!(written_bytes, expected_bytes);
}

#[test]
fn respond_return_pong_if_command_is_ping() {
    let mut parsed = ParsedCommand::new();
    parsed.set_command(Some(Command::PING));
    parsed.set_args(None);
    parsed.set_num_args_in_input(Some(1));

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
    parsed.set_args(Some(vec![String::from("hello "), String::from("world")]));
    parsed.set_num_args_in_input(Some(3));

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
        RedisStore::initialise_test();

        let mut parsed = ParsedCommand::new();
        parsed.set_command(Some(Command::GET));
        parsed.set_args(Some(vec![String::from("hello")]));
        parsed.set_num_args_in_input(Some(2));

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
        RedisStore::initialise_test();

        let mut parsed = ParsedCommand::new();
        let key = "hello";
        let value = "world";
        parsed.set_command(Some(Command::GET));
        parsed.set_args(Some(vec![String::from(key)]));
        parsed.set_num_args_in_input(Some(2));

        {
            let store_lock = RedisStore::get_store();
            let mut store_guard = store_lock.write().unwrap();
            if let Some(store) = store_guard.as_mut() {
                store.set(key, value, &None);
            }
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
        RedisStore::initialise_test();

        let mut parsed = ParsedCommand::new();
        let key = "hello";
        let value = "world";
        let duration = 50;
        parsed.set_command(Some(Command::GET));
        parsed.set_args(Some(vec![String::from(key)]));
        parsed.set_num_args_in_input(Some(2));

        {
            let store_lock = RedisStore::get_store();
            let mut store_guard = store_lock.write().unwrap();
            if let Some(store) = store_guard.as_mut() {
                let set_args = Some(SetOptionalArgs {
                    expire_in_ms: Some(duration)
                });
                store.set(key, value, &set_args);
            }
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
        parsed.set_args(Some(vec![String::from(key)]));
        parsed.set_num_args_in_input(Some(2));

        thread::sleep(std::time::Duration::from_millis(duration));
        client_input.respond(&mut mock_tcp_stream, parsed);

        let written_bytes: &[u8] = mock_tcp_stream.write_buffer.borrow();
        let expected_bytes = "$-1\r\n".as_bytes();
        assert_eq!(written_bytes, expected_bytes);
    });
}
