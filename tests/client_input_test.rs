use redis_starter_rust::request_response::{client_input::ClientInput, command::Command};
use std::borrow::Borrow;
use std::io::ErrorKind;
use std::str::from_utf8;

mod mock;
use mock::tcp_stream::mock_tcp_stream::MockTcpStream;
use mock::tcp_stream::mock_tcp_stream_read_error::MockTcpStreamStreamReadError;
use mock::tcp_stream::mock_tcp_stream_read_no_data::MockTcpStreamStreamReadNoData;
use redis_starter_rust::request_response::client_input::HandleClientInput;
use redis_starter_rust::request_response::parsed_command::ParsedCommand;

#[test]
fn read_input_helper_read_ping_input_correctly() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();
    let input = "*1\\r\\n$4\\r\\nPING\\r\\n";
    let input_bytes = input.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }

    let parsed_result = client_input.read_input_helper(&mut mock_tcp_stream, buffer);
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
fn read_input_helper_read_echo_input_correctly() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStream::new();
    let input = "*3\\r\\n$4\\r\\nECHO\\r\\n$5\\r\\nhello\\r\\n$5\\r\\nworld\\r\\n";
    let input_bytes = input.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }

    let parsed_result = client_input.read_input_helper(&mut mock_tcp_stream, buffer);
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
fn read_input_helper_return_err_if_no_data_is_read() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStreamStreamReadNoData::new();
    let input = "";
    let input_bytes = input.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }

    let parsed = client_input.read_input_helper(&mut mock_tcp_stream, buffer);
    assert!(parsed.is_err());

    let error = parsed.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::ConnectionAborted);
}

#[test]
fn read_input_helper_return_err_if_read_returns_error() {
    let mut client_input = ClientInput::new();
    let mut mock_tcp_stream = MockTcpStreamStreamReadError::new();
    let input = "";
    let input_bytes = input.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }

    let parsed = client_input.read_input_helper(&mut mock_tcp_stream, buffer);
    assert!(parsed.is_err());

    let error = parsed.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
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
