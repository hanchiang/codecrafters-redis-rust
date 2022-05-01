use std::io::ErrorKind;

// https://redis.io/docs/reference/protocol-spec/

/*
  Input:
  Simple string: +ok\r\n
  Error: -error message\r\n
  Bulk string: $5\r\nhello\r\n, $0\r\n\r\n(empty), $-1\r\n(null)
  Integer: :1000\r\n
  Array: *2\r\n$3\r\nhey\r\n$5\r\nthere\r\r(2 strings), *3\r\n:1\r\n:2\r\n:3\r\n(3 integers), *0\r\n(empty), *-1\r\n(null)
    - Nested array: *2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n
*/

/*
  Algorithm:
  - Read the first character to determine the input RESP type(simple string, error, integer, bulk string, array)
    - simple string: read input until CRLF, return RESP
    - error: read input until CRLF, return RESP
    - bulk string:
        - read input until CRLF to get number of bytes in the bulk string
        - read input until CRLF to read the string
        - if number of bytes match the input string, return RESP. Otherwise, return custom error
    - integer: read input until CRLF, return RESP
    - array:
        - read input until CRLF to get number of elements in the array
        - for 1..n, recursively call parse() to parse each element in the array

  Utility:
    - Read input until CRLF: return a tuple of (output RESP, remaining input after CRLF)
    - Check whether we have reached the end of the input
    - Custom errors: unrecognised first character, CRLF not found, incomplete input(bulk string, array)
*/

pub enum RESPOutput {
    SimpleString(String),
    Error(String),
    BulkString(String),
    Integer(i64),
    Array(Vec<RESPOutput>),
}

pub enum Error {
    UnrecognisedSymbol,
    CRLFNotFound,
    IncompleteInput,
}

pub type Result<'a> = std::result::Result<(RESPOutput, &'a [u8]), Error>;

const CR: u8 = b'\r';
const LF: u8 = b'\n';

pub struct Parser {}

impl Parser {
    pub fn parse_resp(input: &[u8]) {}

    pub fn parse_until_crlf(input: &[u8]) -> std::result::Result<(&[u8], &[u8]), Error> {
        for i in 0..input.len() - 1 {
            if input[i] == CR && input[i + 1] == LF {
                return Ok((&input[0..i], &input[i + 2..]))
            }
        }
        Err(Error::CRLFNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_until_crlf_success() {
        let input = vec!["hello world\r\n".as_bytes(), "5\r\nhello\r\n".as_bytes()];
        let expected = vec![("hello world".as_bytes(), "".as_bytes()), ("5".as_bytes(), "hello\r\n".as_bytes())];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_until_crlf(*inp);

            match result {
                Ok(res)  => {
                    assert_eq!(res, expected[index]);
                },
                Err(e) => panic!(e)
            }
        }
    }

    #[test]
    fn parse_until_crlf_error() {
        let input  = "hello world".as_bytes();
        let expected = Error::CRLFNotFound;

        let result = Parser::parse_until_crlf(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert!(matches!(e, expected))
        }
    }
}