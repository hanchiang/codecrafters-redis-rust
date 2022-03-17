use redis_starter_rust::request_response::response_helper::send_bulk_string_response;

mod mock;
use mock::tcp_stream::mock_tcp_stream::MockTcpStream;

#[test]
fn send_bulk_string_response_return_non_null_if_input_is_not_null() {
    let mut mock_tcp_stream = MockTcpStream::new();

    let input = "hello";
    send_bulk_string_response(&mut mock_tcp_stream, Some(input));

    assert_eq!(mock_tcp_stream.write_buffer, "$5\r\nhello\r\n".as_bytes());
}

#[test]
fn send_bulk_string_response_return_null_if_input_is_null() {
    let mut mock_tcp_stream = MockTcpStream::new();

    send_bulk_string_response(&mut mock_tcp_stream, None);

    assert_eq!(mock_tcp_stream.write_buffer, "$-1\r\n".as_bytes());
}