use std::fmt::Arguments;
use std::io::{Error, ErrorKind, IoSlice, Read, Write};

pub struct MockTcpStreamStreamReadError {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>
}

impl MockTcpStreamStreamReadError {
    pub fn new() -> MockTcpStreamStreamReadError {
        MockTcpStreamStreamReadError {
            read_buffer: Vec::new(),
            write_buffer: Vec::new()
        }
    }
}

impl Read for MockTcpStreamStreamReadError {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Err(Error::new(ErrorKind::PermissionDenied, "permission denied"))
    }
}

impl Write for MockTcpStreamStreamReadError {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let buffer: Vec<u8> = buf.iter().cloned().collect();
        self.write_buffer = buffer;
        Ok(self.write_buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        panic!("not implemented");
    }
}