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

pub fn generate_get_buffer() -> [u8; 1024] {
    let input = "*2\\r\\n$3\\r\\nGET\\r\\n$5\\r\\nhello\\r\\n";
    str_to_bytes(input)
}

pub fn generate_set_buffer() -> [u8; 1024] {
    let input = "*3\\r\\n$3\\r\\nSET\\r\\n$5\\r\\nhello\\r\\n$5\\r\\nworld\\r\\n";
    str_to_bytes(input)
}

// expiry_variant: 'EX' or 'PX'
// duration: seconds if expiry_variant is 'EX', else milliseconds
pub fn generate_set_buffer_with_expiry(expiry_variant: &str, duration: u32) -> [u8; 1024] {
    let duration_string = duration.to_string();
    let duration_bytes = duration_string.as_bytes();
    let input = format!("*5\\r\\n$3\\r\\nSET\\r\\n$5\\r\\nhello\\r\\n$5\\r\\nworld\\r\\n$2\\r\\n{}\\r\\n${}\\r\\n{}\\r\\n", expiry_variant.to_uppercase(), duration_bytes.len(), duration);
    str_to_bytes(&input)
}

fn str_to_bytes(str: &str) -> [u8; 1024] {
    let input_bytes = str.as_bytes();
    let mut buffer: [u8; 1024] = [0; 1024];

    for i in 0..input_bytes.len() {
        buffer[i] = input_bytes[i];
    }
    buffer
}
