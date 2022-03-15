use std::fmt::Arguments;
use std::io::{Error, IoSlice, Read, Write};

pub struct MockTcpStream {
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>
}

impl MockTcpStream {
    pub fn new() -> MockTcpStream {
        MockTcpStream {
            read_buffer: Vec::new(),
            write_buffer: Vec::new()
        }
    }
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let buffer: Vec<u8> = buf.iter().cloned().collect();
        self.read_buffer = buffer;
        Ok(self.read_buffer.len())
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let buffer: Vec<u8> = buf.iter().cloned().collect();
        self.write_buffer = buffer;
        Ok(self.write_buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        panic!("not implemented");
    }
}