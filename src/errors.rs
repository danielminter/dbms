use std::{backtrace::Backtrace, io};

#[derive(Debug)]
pub struct SyntaxError {
    msg: String,
    backtrace: Backtrace,
}

impl SyntaxError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: format!("Syntax Error: {msg}"),
            backtrace: Backtrace::capture(),
        }
    }
}

pub fn syntax_error(message: Option<&str>) -> io::Error {
    let message = match message {
        Some(m) => format!("Syntax Error: {m}"),
        None => String::from("Syntax Error"),
    };
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
