use std::io::Write;

pub fn send_bulk_string_response<T: Write>(stream: &mut T, data: &str) {
    let response = format!("${}\r\n{}\r\n", data.len(), data);
    match stream.write(response.as_bytes()) {
        Ok(t) => {
            println!("Wrote {} bytes to output", t);
        },
        Err(e) => {
            println!("unable to write to response: {}", e);
        }
    }
}

pub fn send_pong_response<T: Write>(stream: &mut T) {
    send_simple_string_response(stream, "PONG");
}

pub fn send_simple_string_response<T: Write>(stream: &mut T, str: &str) {
    match stream.write(format_string_response(str).as_bytes()) {
        Ok(t) => {
            println!("Wrote {} bytes to output", t);
        },
        Err(e) => {
            println!("unable to write to response: {}", e);
        }
    }
}

pub fn format_string_response(res: &str) -> String {
    format!("+{}\r\n", res)
}
