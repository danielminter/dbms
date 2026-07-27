mod tokenizer;

use std::io::{self, Write};

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

pub fn evaluate_query(query: &String) {
    let tokens = tokenizer::tokenize(&query);
}

#[derive(Debug)]
pub enum CommandStatus {
    Success,
    Failure,
}
