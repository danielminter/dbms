use std::backtrace::Backtrace;

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
