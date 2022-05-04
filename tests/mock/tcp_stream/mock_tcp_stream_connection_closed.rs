use std::io::{Error, Read, Write};

pub struct MockTcpStreamConnectionClosed {
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>
}

impl MockTcpStreamConnectionClosed {
    pub fn new() -> MockTcpStreamConnectionClosed {
        MockTcpStreamConnectionClosed {
            read_buffer: Vec::new(),
            write_buffer: Vec::new()
        }
    }
}

impl Read for MockTcpStreamConnectionClosed {
    fn read(&mut self, _: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

impl Write for MockTcpStreamConnectionClosed {
    fn write(&mut self, _: &[u8]) -> Result<usize, Error> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        panic!("not implemented");
    }
}