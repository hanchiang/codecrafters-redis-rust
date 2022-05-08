use std::str::from_utf8;
use std::{thread, time};

use serial_test::serial;

use redis_starter_rust::handle_connection_helper;
use redis_starter_rust::request_response::client_input::ClientInput;
use redis_starter_rust::store::redis::{RedisStore, Store};
use redis_starter_rust::AppError;

mod mock;

use mock::common::mock_input::{
    generate_echo_buffer, generate_get_buffer, generate_incomplete_input_buffer,
    generate_ping_buffer, generate_set_buffer, generate_set_buffer_with_expiry,
};
use mock::common::reset_redis::with_reset_redis;
use mock::tcp_stream::mock_tcp_stream::MockTcpStream;
use mock::tcp_stream::mock_tcp_stream_read_error::MockTcpStreamStreamReadError;
use crate::mock::tcp_stream::mock_tcp_stream_connection_closed::MockTcpStreamConnectionClosed;


#[test]
fn handle_connection_helper_should_return_error_if_unable_to_read_from_stream() {
    let mut mock_tcp_stream = MockTcpStreamStreamReadError::new();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(!result.is_ok());
    assert!(mock_tcp_stream.write_buffer.is_empty());
    assert_eq!(client_input.get_input(), "".as_bytes());
}

#[test]
fn handle_connection_helper_should_return_error_connection_is_closed() {
    let mut mock_tcp_stream = MockTcpStreamConnectionClosed::new();
    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(!result.is_ok());
    assert!(mock_tcp_stream.write_buffer.is_empty());
    assert_eq!(client_input.get_input(), "".as_bytes());
    assert_eq!(result.unwrap_err(), AppError::ConnectionClosed(String::from("Connection closed")));
}

#[test]
fn handle_connection_helper_should_return_error_if_incomplete_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = generate_incomplete_input_buffer();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_err());
    assert!(mock_tcp_stream.write_buffer.is_empty());
    assert_eq!(result.unwrap_err(), AppError::IncompleteInput(String::from("Incomplete input")));
}

#[test]
fn handle_connection_helper_should_process_ping_correctly_and_reset_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = generate_ping_buffer();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);
    println!("{:?}", result);

    assert!(result.is_ok());
    assert_eq!(mock_tcp_stream.write_buffer, "+PONG\r\n".as_bytes());
    assert_eq!(client_input.get_input(), "".as_bytes());
}

#[test]
fn handle_connection_helper_should_process_echo_correctly_and_reset_input() {
    let mut mock_tcp_stream = MockTcpStream::new();
    mock_tcp_stream.read_buffer = generate_echo_buffer();

    let mut client_input = ClientInput::new();
    let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

    assert!(result.is_ok());
    assert_eq!(
        mock_tcp_stream.write_buffer,
        "$10\r\nhelloworld\r\n".as_bytes()
    );
    assert_eq!(client_input.get_input(), "".as_bytes());
}


#[test]
#[serial]
fn handle_connection_helper_return_nil_for_get_command_if_there_is_no_data_and_reset_input() {
    with_reset_redis(|| {
        RedisStore::initialise();

        let mut mock_tcp_stream = MockTcpStream::new();
        mock_tcp_stream.read_buffer = generate_get_buffer();

        let mut client_input = ClientInput::new();
        let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

        println!("{:?}", from_utf8(&mock_tcp_stream.write_buffer).unwrap());

        assert!(result.is_ok());
        assert_eq!(mock_tcp_stream.write_buffer, "$-1\r\n".as_bytes());
        assert_eq!(client_input.get_input(), "".as_bytes());
    });
}

#[test]
#[serial]
fn handle_connection_helper_return_ok_for_set_command_and_can_get_result_and_reset_input() {
    with_reset_redis(|| {
        RedisStore::initialise();
        {
            let mut mock_tcp_stream = MockTcpStream::new();
            mock_tcp_stream.read_buffer = generate_set_buffer();

            let mut client_input = ClientInput::new();
            let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

            assert!(result.is_ok());
            assert_eq!(mock_tcp_stream.write_buffer, "$2\r\nOK\r\n".as_bytes());
            assert_eq!(client_input.get_input(), "".as_bytes());
        }

        {
            let mut mock_tcp_stream = MockTcpStream::new();
            mock_tcp_stream.read_buffer = generate_get_buffer();

            let mut client_input = ClientInput::new();
            let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

            assert!(result.is_ok());
            assert_eq!(mock_tcp_stream.write_buffer, "$5\r\nworld\r\n".as_bytes());
            assert_eq!(client_input.get_input(), "".as_bytes());
        }
    });
}

#[test]
#[serial]
fn handle_connection_helper_return_ok_for_set_command_with_expiry_and_can_get_result_before_expiry_and_reset_input() {
    with_reset_redis(|| {
        RedisStore::initialise();
        {
            let mut mock_tcp_stream = MockTcpStream::new();
            mock_tcp_stream.read_buffer = generate_set_buffer_with_expiry("px", 100);

            let mut client_input = ClientInput::new();
            let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

            assert!(result.is_ok());
            assert_eq!(mock_tcp_stream.write_buffer, "$2\r\nOK\r\n".as_bytes());
            assert_eq!(client_input.get_input(), "".as_bytes());
        }

        // should return value before key expire
        {
            let mut mock_tcp_stream = MockTcpStream::new();
            mock_tcp_stream.read_buffer = generate_get_buffer();

            let mut client_input = ClientInput::new();
            let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

            assert!(result.is_ok());
            assert_eq!(mock_tcp_stream.write_buffer, "$5\r\nworld\r\n".as_bytes());
            assert_eq!(client_input.get_input(), "".as_bytes());
        }

        // shouldn't return value after key expire
        {
            thread::sleep(time::Duration::from_millis(100));
            let mut mock_tcp_stream = MockTcpStream::new();
            mock_tcp_stream.read_buffer = generate_get_buffer();

            let mut client_input = ClientInput::new();
            let result = handle_connection_helper(&mut mock_tcp_stream, &mut client_input);

            assert!(result.is_ok());
            assert_eq!(mock_tcp_stream.write_buffer, "$-1\r\n".as_bytes());
            assert_eq!(client_input.get_input(), "".as_bytes());
        }
    });
}

