use std::io;

pub fn syntax_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error")
}
