mod parser;
mod tokenizer;

use std::io::{self, Write};

use crate::tokenizer::Token;

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
    let tokens: Vec<Token> = tokenizer::tokenize(query).unwrap();
    // let ast = parser::parse_tokens(seperated_strings).unwrap();
}

#[derive(Debug)]
pub enum CommandStatus {
    Success,
    Failure,
}
