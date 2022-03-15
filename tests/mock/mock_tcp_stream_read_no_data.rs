use std::fmt::Arguments;
use std::io::{Error, IoSlice, Read, Write};

pub struct MockTcpStreamStreamReadNoData {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>
}

impl MockTcpStreamStreamReadNoData {
    pub fn new() -> MockTcpStreamStreamReadNoData {
        MockTcpStreamStreamReadNoData {
            read_buffer: Vec::new(),
            write_buffer: Vec::new()
        }
    }
}

impl Read for MockTcpStreamStreamReadNoData {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

impl Write for MockTcpStreamStreamReadNoData {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let buffer: Vec<u8> = buf.iter().cloned().collect();
        self.write_buffer = buffer;
        Ok(self.write_buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        panic!("not implemented");
    }
}