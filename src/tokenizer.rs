use std::{
    collections::VecDeque,
    ffi::c_int,
    io::{self},
};

pub fn tokenize(text: &str) -> Result<TokenQueue, io::Error> {
    // convert_strings(split_text)
    let mut tokens: Vec<Token> = vec![];
    let mut characters: VecDeque<char> = text.chars().collect();

    // Main parsing loop
    'parsing: while !characters.is_empty() {
        //  Peek at the next character
        let mut next_char = *characters.front().unwrap();

        // Skip over white space
        if char::is_whitespace(next_char) {
            characters.pop_front();
            continue 'parsing;
        }

        // Special Cases for items in double quotes
        if next_char == '\"' {
            // Initialize a new token
            let mut current_string: String = String::new();

            // Drop the starting quote
            characters.pop_front();

            next_char = *characters.front().unwrap();

            while !characters.is_empty() && next_char != '\"' {
                current_string.push(characters.pop_front().unwrap());
                if !characters.is_empty() {
                    next_char = *characters.front().unwrap();
                }
            }

            let token = Token::Identifier(Identifier {
                value: current_string,
            });

            // Push it onto our vector
            tokens.push(token);

            // Make sure we skip the trailing quote
            if *characters.front().unwrap() == '\"' {
                characters.pop_front();
            }

            continue 'parsing;
        }

        // Special Case for items in single quotes
        if next_char == '\'' {
            // Initialize a new token
            let mut current_string: String = String::new();

            // Drop the starting quote
            characters.pop_front();
            next_char = *characters.front().unwrap();

            while !characters.is_empty() && next_char != '\'' {
                current_string.push(characters.pop_front().unwrap());
                if !characters.is_empty() {
                    next_char = *characters.front().unwrap();
                }
            }

            let token = Token::StringLiteral(StringLiteral {
                value: current_string,
            });

            // Push it onto our vector
            tokens.push(token);

            // Make sure we skip the trailing quote
            if *characters.front().unwrap() == '\'' {
                characters.pop_front();
            }

            continue 'parsing;
        }

        // Determine what type of token we have based on the first char
        if char::is_alphabetic(next_char) {
            // Initialize a new token
            let mut current_string: String = String::new();

            if char::is_uppercase(next_char) {
                // Parse it as a keyword if its in Caps
                //Build the current token with all consecutive caps characters
                while !characters.is_empty() && char::is_uppercase(next_char) {
                    current_string.push(characters.pop_front().unwrap());

                    if !characters.is_empty() {
                        next_char = *characters.front().unwrap();
                    }
                }

                // Construct the token
                let token = Token::Keyword(Keyword {
                    value: current_string,
                });

                // Push it onto our vector
                tokens.push(token);

                continue 'parsing;
            } else {
                // If not the above, parse as an identifier
                //Build the current token with all consecutive non-caps characters
                while !characters.is_empty() && char::is_alphabetic(next_char) {
                    current_string.push(characters.pop_front().unwrap());
                    if !characters.is_empty() {
                        next_char = *characters.front().unwrap();
                    }
                }
                // Construct the token
                let token = Token::Identifier(Identifier {
                    value: current_string,
                });

                // Push it onto our vector
                tokens.push(token);
                continue 'parsing;
            }
        }

        // Determine if its numeric
        if char::is_numeric(next_char) {
            // Initialize a new token
            let mut current_string: String = String::new();

            if char::is_digit(next_char, 10) {
                // Try and parse it as a integer literal
                while !characters.is_empty() && char::is_digit(next_char, 10) {
                    current_string.push(characters.pop_front().unwrap());
                    if !characters.is_empty() {
                        next_char = *characters.front().unwrap();
                    }
                }
                // Construct the token
                // Verify its a valid c_int and construct a token
                let token = match current_string.parse::<c_int>() {
                    Ok(int) => Token::IntLiteral(IntLiteral { value: int }),
                    Err(_) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                    }
                };

                // Push it onto our vector
                tokens.push(token);
            }
        }
        if char::is_ascii_punctuation(&next_char) {
            // Initialize the string
            let mut current_string: String = String::new();

            // Numeric but not a digit, parse as a symbol
            while !characters.is_empty() && char::is_ascii_punctuation(&next_char) {
                current_string = tokenize_symbol(&mut characters);

                if !characters.is_empty() {
                    next_char = *characters.front().unwrap();
                }
            }

            let token = Token::Symbol(Symbol {
                value: current_string,
            });

            // Push it onto our vector
            tokens.push(token);

            continue 'parsing;
        }

        // Consume the character if it hasn't been caught by anything
        characters.pop_front();
    }

    Ok(TokenQueue::new(tokens))
}

fn tokenize_symbol(characters: &mut VecDeque<char>) -> String {
    let mut current_string: String;
    let next_char = characters.pop_front().unwrap();
    match next_char {
        '<' => {
            current_string = String::from("LESSTHAN");
            // Handle the compound symbols
            if !characters.is_empty() && *characters.front().unwrap() == '=' {
                append_equals(characters, &mut current_string);
            }
        }
        '>' => {
            current_string = String::from("GREATERTHAN");
            // Handle the compound symbols
            if !characters.is_empty() && *characters.front().unwrap() == '=' {
                append_equals(characters, &mut current_string);
            }
        }
        '(' => {
            current_string = String::from("LPAREN");
        }
        ')' => {
            current_string = String::from("RPAREN");
        }
        ',' => {
            current_string = String::from("COMMA");
        }
        '*' => {
            current_string = String::from("STAR");
        }
        '=' => {
            current_string = String::from("EQUAL");
        }
        '+' => {
            current_string = String::from("PLUS");
            // Handle the compound symbols
            if !characters.is_empty() && *characters.front().unwrap() == '=' {
                append_equals(characters, &mut current_string);
            }
        }
        '-' => {
            current_string = String::from("MINUS");
            // Handle the compound symbols
            if !characters.is_empty() && *characters.front().unwrap() == '=' {
                append_equals(characters, &mut current_string);
            }
        }
        ';' => {
            current_string = String::from("SEMICOLON");
        }
        _ => {
            current_string = String::from("UNK");
        }
    }

    fn append_equals(characters: &mut VecDeque<char>, current_string: &mut String) {
        current_string.push_str("EQUAL");

        // Consume the equals
        characters.pop_front();
    }

    // Return our constructed token
    current_string
}

#[derive(Debug, PartialEq)]
pub struct TokenQueue {
    tokens: VecDeque<Token>,
}

impl TokenQueue {
    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    pub fn new(token_vec: Vec<Token>) -> TokenQueue {
        TokenQueue {
            tokens: VecDeque::from(token_vec),
        }
    }

    pub fn peek_and_check_value(&self, value: Vec<&str>) -> bool {
        match self.peek() {
            Some(t) => value.contains(&t.string_value()),
            None => false,
        }
    }

    pub fn pop_and_check_value(&mut self, value: Vec<&str>) -> Option<Token> {
        match self.peek() {
            Some(t) => {
                if value.contains(&t.string_value()) {
                    self.next()
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if !self.tokens.is_empty() {
            Some(self.tokens.front()?)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if !self.tokens.is_empty() {
            Some(self.tokens.pop_front()?)
        } else {
            None
        }
    }

    pub fn print_queue(&self) {
        for tok in &self.tokens {
            let value = tok.string_value();
            println!("{value}");
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    IntLiteral(IntLiteral),
    Symbol(Symbol),
    Operator(Operator),
}

impl Token {
    pub fn int_value(&self) -> i32 {
        match self {
            Token::IntLiteral(t) => *t.value(),
            _ => 0,
        }
    }

    pub fn string_value(&self) -> &str {
        match self {
            Token::Keyword(t) => t.value(),
            Token::Identifier(t) => t.value(),
            Token::StringLiteral(t) => t.value(),
            Token::Symbol(t) => t.value(),
            Token::Operator(t) => t.value(),
            _ => "",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Keyword {
    pub value: String,
}

impl Keyword {
    pub fn value(&self) -> &str {
        &self.value.as_str()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    pub fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StringLiteral {
    pub value: String,
}

impl StringLiteral {
    fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntLiteral {
    pub value: c_int,
}

impl IntLiteral {
    fn value(&self) -> &c_int {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    pub value: String,
}

impl Symbol {
    fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Operator {
    pub value: String,
}

impl Operator {
    fn value(&self) -> &String {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        assert_eq!(
            tokenize("INSERT").unwrap(),
            TokenQueue::new(vec![Token::Keyword(Keyword {
                value: "INSERT".to_string()
            })],)
        )
    }

    #[test]
    fn test_no_quote_identifier() {
        assert_eq!(
            tokenize("abc").unwrap(),
            TokenQueue::new(vec![Token::Identifier(Identifier {
                value: "abc".to_string()
            })])
        )
    }

    #[test]
    fn test_quoted_identifier() {
        assert_eq!(
            tokenize("\"two words\"").unwrap(),
            TokenQueue::new(vec![Token::Identifier(Identifier {
                value: "two words".to_string()
            })])
        )
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            tokenize("\'abc\'").unwrap(),
            TokenQueue::new(vec![Token::StringLiteral(StringLiteral {
                value: "abc".to_string()
            })])
        )
    }

    #[test]
    fn test_int_literal() {
        assert_eq!(
            tokenize("123").unwrap(),
            TokenQueue::new(vec![Token::IntLiteral(IntLiteral { value: 123 })])
        )
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            tokenize("< > ( ) * , = + - ; ").unwrap(),
            TokenQueue::new(vec![
                Token::Symbol(Symbol {
                    value: "LESSTHAN".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "GREATERTHAN".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "LPAREN".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "RPAREN".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "STAR".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "COMMA".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "EQUAL".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "PLUS".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "MINUS".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "SEMICOLON".to_string()
                }),
            ])
        )
    }

    #[test]
    fn test_compound_symbols() {
        assert_eq!(
            tokenize("<= >= += -=").unwrap(),
            TokenQueue::new(vec![
                Token::Symbol(Symbol {
                    value: "LESSTHANEQUAL".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "GREATERTHANEQUAL".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "PLUSEQUAL".to_string()
                }),
                Token::Symbol(Symbol {
                    value: "MINUSEQUAL".to_string()
                }),
            ])
        )
    }

    #[test]
    fn test_mixed_literals() {
        assert_eq!(
            tokenize("123 'abc' 456 'def'").unwrap(),
            TokenQueue::new(vec![
                Token::IntLiteral(IntLiteral { value: 123 }),
                Token::StringLiteral(StringLiteral {
                    value: "abc".to_string()
                }),
                Token::IntLiteral(IntLiteral { value: 456 }),
                Token::StringLiteral(StringLiteral {
                    value: "def".to_string()
                })
            ])
        )
    }

    #[test]
    fn test_mixed_quotes() {
        assert_eq!(
            tokenize("\"abc\" 'def' \"ghi\" 'jkl'").unwrap(),
            TokenQueue::new(vec![
                Token::Identifier(Identifier {
                    value: "abc".to_string()
                }),
                Token::StringLiteral(StringLiteral {
                    value: "def".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "ghi".to_string()
                }),
                Token::StringLiteral(StringLiteral {
                    value: "jkl".to_string()
                }),
            ])
        )
    }
}
