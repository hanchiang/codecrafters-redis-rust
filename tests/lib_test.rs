use redis_starter_rust::handle_connection_helper;
use redis_starter_rust::request_response::client_input::ClientInput;

mod mock;

use mock::tcp_stream::mock_tcp_stream::MockTcpStream;
use mock::tcp_stream::mock_tcp_stream_read_error::MockTcpStreamStreamReadError;
use mock::common::mock_input::{generate_ping_buffer, generate_echo_buffer, generate_incomplete_input_buffer, generate_get_buffer};

#[test]
fn handle_connection_should_return_error_if_unable_to_read_from_stream() {
    let mut mock_tcp_stream = MockTcpStreamStreamReadError::new();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(!result.is_ok());
    assert!(mock_tcp_stream.write_buffer.is_empty());
    assert_eq!(client_input.get_input(), "");
}

#[test]
fn handle_connection_should_not_respond_to_incomplete_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    let buffer = generate_incomplete_input_buffer();

    mock_tcp_stream.read_buffer = buffer.to_vec();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_ok());
    assert!(mock_tcp_stream.write_buffer.is_empty());
    assert_eq!(client_input.get_input(), "*3\\r\\n$4\\r\\nECHO\\r\\n$5\\r\\nhello\\r\\n$5\\r\\n");
}

#[test]
fn handle_connection_should_process_ping_correctly_and_reset_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    let buffer = generate_ping_buffer();

    mock_tcp_stream.read_buffer = buffer.to_vec();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_ok());
    assert_eq!(mock_tcp_stream.write_buffer, "+PONG\r\n".as_bytes());
    assert_eq!(client_input.get_input(), "");
}

#[test]
fn handle_connection_should_process_echo_correctly_and_reset_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    let buffer = generate_echo_buffer();

    mock_tcp_stream.read_buffer = buffer.to_vec();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_ok());
    assert_eq!(mock_tcp_stream.write_buffer, "$10\r\nhelloworld\r\n".as_bytes());
    assert_eq!(client_input.get_input(), "");
}

#[test]
fn handle_connection_should_process_get_correctly_if_redis_return_none_and_reset_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    let buffer = generate_get_buffer();

    mock_tcp_stream.read_buffer = buffer.to_vec();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_ok());
    assert_eq!(mock_tcp_stream.write_buffer, "$-1\r\n".as_bytes());
    assert_eq!(client_input.get_input(), "");
}