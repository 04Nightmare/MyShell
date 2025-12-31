use std::io::{self, Write};

pub fn redraw_entire_line(prompt: &str, buffer: &str) {
    print!("\r\x1b[K{}{}", prompt, buffer);
    io::stdout().flush().unwrap();
}
