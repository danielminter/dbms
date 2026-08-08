pub mod errors;
mod parser;
mod tokenizer;

use std::io::{self, Write};

use crate::tokenizer::{Token, TokenQueue};

pub fn print_prompt() {
    print!("dbms >");
    io::stdout().flush().unwrap(); // Flush the stdout so the print! actually shows up.
}

pub fn get_input() -> String {
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Error reading input");

    String::from(user_input.trim())
}

pub fn evaluate_query(query: &str) {
    let tokens: TokenQueue = tokenizer::tokenize(query).unwrap();
    let ast = parser::create_ast(tokens).unwrap();
}

#[derive(Debug)]
pub enum CommandStatus {
    Success,
    Failure,
}
