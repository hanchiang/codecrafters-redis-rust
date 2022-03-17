
pub fn generate_ping_buffer() -> [u8; 1024] {
    let input = "*1\\r\\n$4\\r\\nPING\\r\\n";
    str_to_bytes(input)
}

pub fn generate_echo_buffer() -> [u8; 1024] {
    let input = "*3\\r\\n$4\\r\\nECHO\\r\\n$5\\r\\nhello\\r\\n$5\\r\\nworld\\r\\n";
    str_to_bytes(input)
}

pub fn generate_incomplete_input_buffer() -> [u8; 1024] {
    let input = "*3\\r\\n$4\\r\\nECHO\\r\\n$5\\r\\nhello\\r\\n$5\\r\\n";
    str_to_bytes(input)
}

fn str_to_bytes(str: &str) -> [u8; 1024] {
    let input_bytes = str.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }
    buffer
}