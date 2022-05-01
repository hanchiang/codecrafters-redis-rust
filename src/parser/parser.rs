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
        - for 1..n, recursively call parse_resp() to parse each element in the array

  Utility:
    - Read input until CRLF: return a tuple of (output RESP, remaining input after CRLF)
    - Check whether we have reached the end of the input
    - Custom errors: unrecognised first character, CRLF not found, incomplete input(bulk string, array)
*/

#[derive(Debug, PartialEq)]
pub enum RESPOutput {
    SimpleString(String),
    Error(String),
    BulkString(String),
    Integer(i64),
    Array(Vec<RESPOutput>),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnrecognisedSymbol,
    CRLFNotFound,
    IncompleteInput,
    InvalidInput,
}

pub type ParseResult<'a> = std::result::Result<(RESPOutput, &'a [u8]), ParseError>;
pub type ParseCRLFResult<'a> = std::result::Result<(&'a [u8], &'a [u8]), ParseError>;

const CR: u8 = b'\r';
const LF: u8 = b'\n';

pub struct Parser {}

impl Parser {
    pub fn parse_resp(input: &[u8]) -> ParseResult {
        if input.len() == 0 {
            return Err(ParseError::IncompleteInput);
        }
        let symbol_temp = String::from_utf8_lossy(&input[0..1]);
        let symbol = symbol_temp.as_ref();
        let remaining = &input[1..];
        match symbol {
            "+" => Parser::parse_simple_string(remaining),
            "-" => Parser::parse_error(remaining),
            "$" => Parser::parse_bulk_string(remaining),
            ":" => Parser::parse_integer(remaining),
            "*" => Parser::parse_array(remaining),
            _ => return Err(ParseError::UnrecognisedSymbol),
        }
    }

    fn parse_simple_string(input: &[u8]) -> ParseResult {
        Parser::parse_until_crlf(input).map(|(result, remaining)| {
            let string = String::from(String::from_utf8_lossy(result));
            (RESPOutput::SimpleString(string), remaining)
        })
    }

    fn parse_error(input: &[u8]) -> ParseResult {
        Parser::parse_until_crlf(input).map(|(result, remaining)| {
            let string = String::from(String::from_utf8_lossy(result));
            (RESPOutput::Error(string), remaining)
        })
    }

    fn parse_bulk_string(input: &[u8]) -> ParseResult {
        // First parse is to get the number of bytes in the bulk string
        let parsed = Parser::parse_until_crlf(input);
        if parsed.is_err() {
            return Err(parsed.unwrap_err());
        }

        let (numBytes, remaining) = parsed.unwrap();
        if String::from_utf8_lossy(numBytes) == "-1" {
            return Ok((RESPOutput::Null, "".as_bytes()));
        }

        // Second parse is to get the string itself
        let parsed = Parser::parse_until_crlf(remaining);
        if parsed.is_err() {
            return Err(parsed.unwrap_err());
        }

        let (result, remaining) = parsed.unwrap();

        let numBytesInt: usize = String::from_utf8_lossy(numBytes).parse().unwrap();
        if result.len().lt(&numBytesInt) {
            return Err(ParseError::IncompleteInput);
        }
        if result.len().gt(&numBytesInt) {
            return Err(ParseError::InvalidInput);
        }

        let res = String::from(String::from_utf8_lossy(result));
        Ok((RESPOutput::BulkString(res), remaining))
    }

    fn parse_integer(input: &[u8]) -> ParseResult {
        let parsed = Parser::parse_until_crlf(input);
        if parsed.is_err() {
            return Err(parsed.unwrap_err());
        }

        let (result, remaining) = parsed.unwrap();
        let string = String::from(String::from_utf8_lossy(result));
        let num: i64 = match string.parse() {
            Ok(res) => res,
            Err(e) => return Err(ParseError::InvalidInput),
        };
        Ok((RESPOutput::Integer(num), remaining))
    }

    fn parse_array(input: &[u8]) -> ParseResult {
        // First parse is to get the number of elements in the array
        let parsed = Parser::parse_until_crlf(input);
        if parsed.is_err() {
            return Err(parsed.unwrap_err());
        }

        let (numElements, remaining) = parsed.unwrap();
        if String::from_utf8_lossy(numElements) == "-1" {
            return Ok((RESPOutput::Null, "".as_bytes()));
        }

        let numElementsInt: u32 = match String::from(String::from_utf8_lossy(numElements)).parse() {
            Ok(res) => res,
            Err(e) => return Err(ParseError::InvalidInput),
        };

        let mut resp_result: Vec<RESPOutput> = vec![];
        let mut remaining = remaining;

        // Recursively parse for each element in the array
        for i in 0..numElementsInt {
            let parsed = Parser::parse_resp(remaining);
            if parsed.is_err() {
                return Err(parsed.unwrap_err());
            }
            let (result, rem) = parsed.unwrap();
            resp_result.push(result);
            remaining = rem;
        }
        return Ok((RESPOutput::Array(resp_result), remaining));
    }

    fn parse_until_crlf(input: &[u8]) -> ParseCRLFResult {
        for i in 0..input.len() - 1 {
            if input[i] == CR && input[i + 1] == LF {
                return Ok((&input[0..i], &input[i + 2..]));
            }
        }
        Err(ParseError::CRLFNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_until_crlf_success() {
        let input = vec!["hello world\r\n".as_bytes(), "5\r\nhello\r\n".as_bytes()];
        let expected = vec![
            ("hello world".as_bytes(), "".as_bytes()),
            ("5".as_bytes(), "hello\r\n".as_bytes()),
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_until_crlf(*inp);

            match result {
                Ok(res) => {
                    assert_eq!(res, expected[index]);
                }
                Err(e) => panic!(e),
            }
        }
    }

    #[test]
    fn parse_until_crlf_error() {
        let input = "hello world".as_bytes();
        let expected = ParseError::CRLFNotFound;

        let result = Parser::parse_until_crlf(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn parse_simple_string_success() {
        let input = "hello world\r\n".as_bytes();
        let expected = (
            RESPOutput::SimpleString(String::from("hello world")),
            "".as_bytes(),
        );

        let result = Parser::parse_simple_string(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_error_success() {
        let input = "error\r\n".as_bytes();
        let expected = (RESPOutput::Error(String::from("error")), "".as_bytes());

        let result = Parser::parse_error(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_bulk_string_success() {
        let input = vec![
            "11\r\nhello world\r\n".as_bytes(),
            "0\r\n\r\n".as_bytes(),
            "-1\r\n".as_bytes(),
        ];

        let expected = vec![
            (
                (RESPOutput::BulkString(String::from("hello world"))),
                "".as_bytes(),
            ),
            ((RESPOutput::BulkString(String::from("")), "".as_bytes())),
            ((RESPOutput::Null, "".as_bytes())),
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_bulk_string(inp);
            match result {
                Ok(res) => assert_eq!(res, expected[index]),
                Err(e) => panic!(e),
            }
        }
    }

    #[test]
    fn parse_bulk_string_error() {
        let input = vec!["11\r\nhello\r\n".as_bytes(), "3\r\nhello\r\n".as_bytes()];
        let expected = vec![ParseError::IncompleteInput, ParseError::InvalidInput];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_bulk_string(inp);
            match result {
                Ok(res) => panic!(),
                Err(e) => assert_eq!(e, expected[index]),
            }
        }
    }

    #[test]
    fn parse_integer_success() {
        let input = "1234\r\n".as_bytes();
        let expected = (RESPOutput::Integer(1234), "".as_bytes());

        let result = Parser::parse_integer(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_integer_error() {
        let input = "1a\r\n".as_bytes();
        let expected = ParseError::InvalidInput;

        let result = Parser::parse_integer(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn parse_array_success() {
        let input = vec![
            "2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes(),
            "3\r\n:1000\r\n+hello world\r\n-got error\r\n".as_bytes(),
            "0\r\n".as_bytes(),
            "-1\r\n".as_bytes(),
            "2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes(),
        ];
        let expected = vec![
            (
                RESPOutput::Array(vec![
                    RESPOutput::BulkString(String::from("hello")),
                    RESPOutput::BulkString(String::from("world")),
                ]),
                "".as_bytes(),
            ),
            (
                RESPOutput::Array(vec![
                    RESPOutput::Integer(1000),
                    RESPOutput::SimpleString(String::from("hello world")),
                    RESPOutput::Error(String::from("got error")),
                ]),
                "".as_bytes(),
            ),
            (RESPOutput::Array(vec![]), "".as_bytes()),
            (RESPOutput::Null, "".as_bytes()),
            (
                RESPOutput::Array(vec![
                    RESPOutput::Array(vec![
                        RESPOutput::Integer(1),
                        RESPOutput::Integer(2),
                        RESPOutput::Integer(3),
                    ]),
                    RESPOutput::Array(vec![
                        RESPOutput::BulkString(String::from("hello")),
                        RESPOutput::BulkString(String::from("world")),
                    ]),
                ]),
                "".as_bytes(),
            ),
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_array(*inp);
            match result {
                Ok(res) => assert_eq!(res, expected[index]),
                Err(e) => panic!(e),
            }
        }
    }

    #[test]
    fn parse_array_error() {
        let input = vec![
            "2\r\n$5\r\nhello\r\n".as_bytes(),
            "3\r\n:1000\r\n+hello world\r\n$5\r\nhello world\r\n".as_bytes(),
            "2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n$5\r\nworld".as_bytes(),
        ];
        let expected = vec![
            ParseError::IncompleteInput,
            ParseError::InvalidInput,
            ParseError::CRLFNotFound,
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_array(*inp);
            match result {
                Ok(res) => panic!(),
                Err(e) => assert_eq!(e, expected[index]),
            }
        }
    }

    #[test]
    fn parse_resp_incomplete_input() {
        let input = "*2\r\n:1\r\n".as_bytes();
        let expected = ParseError::IncompleteInput;

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn parse_resp_unrecognised_input() {
        let input = "5\r\nhello\r\n".as_bytes();
        let expected = ParseError::UnrecognisedSymbol;

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn parse_resp_simple_string() {
        let input = "+hello world\r\n".as_bytes();
        let expected = (
            RESPOutput::SimpleString(String::from("hello world")),
            "".as_bytes(),
        );

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_resp_error() {
        let input = "-error\r\n".as_bytes();
        let expected = (RESPOutput::Error(String::from("error")), "".as_bytes());

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_resp_bulk_string_success() {
        let input = vec![
            "$11\r\nhello world\r\n".as_bytes(),
            "$0\r\n\r\n".as_bytes(),
            "$-1\r\n".as_bytes(),
        ];

        let expected = vec![
            (
                (RESPOutput::BulkString(String::from("hello world"))),
                "".as_bytes(),
            ),
            ((RESPOutput::BulkString(String::from("")), "".as_bytes())),
            ((RESPOutput::Null, "".as_bytes())),
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_resp(inp);
            match result {
                Ok(res) => assert_eq!(res, expected[index]),
                Err(e) => panic!(e),
            }
        }
    }

    #[test]
    fn parse_resp_bulk_string_error() {
        let input = vec!["$11\r\nhello\r\n".as_bytes(), "$3\r\nhello\r\n".as_bytes()];
        let expected = vec![ParseError::IncompleteInput, ParseError::InvalidInput];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_resp(inp);
            match result {
                Ok(res) => panic!(),
                Err(e) => assert_eq!(e, expected[index]),
            }
        }
    }

    #[test]
    fn parse_resp_integer_success() {
        let input = ":1234\r\n".as_bytes();
        let expected = (RESPOutput::Integer(1234), "".as_bytes());

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => assert_eq!(res, expected),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn parse_resp_integer_error() {
        let input = ":1a\r\n".as_bytes();
        let expected = ParseError::InvalidInput;

        let result = Parser::parse_resp(input);
        match result {
            Ok(res) => panic!(),
            Err(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn parse_resp_array_success() {
        let input = vec![
            "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes(),
            "*3\r\n:1000\r\n+hello world\r\n-got error\r\n".as_bytes(),
            "*0\r\n".as_bytes(),
            "*-1\r\n".as_bytes(),
            "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes(),
        ];
        let expected = vec![
            (
                RESPOutput::Array(vec![
                    RESPOutput::BulkString(String::from("hello")),
                    RESPOutput::BulkString(String::from("world")),
                ]),
                "".as_bytes(),
            ),
            (
                RESPOutput::Array(vec![
                    RESPOutput::Integer(1000),
                    RESPOutput::SimpleString(String::from("hello world")),
                    RESPOutput::Error(String::from("got error")),
                ]),
                "".as_bytes(),
            ),
            (RESPOutput::Array(vec![]), "".as_bytes()),
            (RESPOutput::Null, "".as_bytes()),
            (
                RESPOutput::Array(vec![
                    RESPOutput::Array(vec![
                        RESPOutput::Integer(1),
                        RESPOutput::Integer(2),
                        RESPOutput::Integer(3),
                    ]),
                    RESPOutput::Array(vec![
                        RESPOutput::BulkString(String::from("hello")),
                        RESPOutput::BulkString(String::from("world")),
                    ]),
                ]),
                "".as_bytes(),
            ),
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_resp(*inp);
            match result {
                Ok(res) => assert_eq!(res, expected[index]),
                Err(e) => panic!(e),
            }
        }
    }

    #[test]
    fn parse_resp_array_error() {
        let input = vec![
            "*2\r\n$5\r\nhello\r\n".as_bytes(),
            "*3\r\n:1000\r\n+hello world\r\n$5\r\nhello world\r\n".as_bytes(),
            "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n$5\r\nworld".as_bytes(),
        ];
        let expected = vec![
            ParseError::IncompleteInput,
            ParseError::InvalidInput,
            ParseError::CRLFNotFound,
        ];

        for (index, inp) in input.iter().enumerate() {
            let result = Parser::parse_resp(*inp);
            match result {
                Ok(res) => panic!(),
                Err(e) => assert_eq!(e, expected[index]),
            }
        }
    }
}
