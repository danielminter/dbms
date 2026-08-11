use std::{
    collections::VecDeque,
    ffi::c_int,
    io::{self},
};

use crate::{
    errors::{InvalidToken, InvalidValue, MissingToken, SyntaxError},
    parser::LiteralType,
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

            let token = Token::Literal(Literal {
                value: current_string,
                literal_type: LiteralType::String,
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
                    Ok(int) => Token::Literal(Literal {
                        value: int.to_string(),
                        literal_type: LiteralType::Integer,
                    }),
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

    pub fn validate_token_type(&mut self, token_type: TokenTag) -> Result<Token, SyntaxError> {
        match self.peek() {
            Some(t) => {
                if token_type == t.tag() {
                    Ok(self.next()?)
                } else {
                    Err(SyntaxError::InvalidToken(InvalidToken::new(
                        token_type,
                        t.tag(),
                    )))
                }
            }
            None => Err(SyntaxError::MissingToken(MissingToken::new(Some(
                token_type,
            )))),
        }
    }

    pub fn validate_token_value(&mut self, value: Vec<&str>) -> Result<Token, SyntaxError> {
        match self.peek() {
            Some(t) => match value.contains(&t.value()) {
                true => {
                    let token = self.next()?;
                    Ok(token)
                }
                false => Err(SyntaxError::InvalidValue(InvalidValue::new(
                    value,
                    t.value(),
                ))),
            },
            None => Err(SyntaxError::MissingToken(MissingToken::new(None))),
        }
    }

    pub fn next_token_type(&self) -> TokenTag {
        match self.peek() {
            Some(t) => t.tag(),
            None => TokenTag::None,
        }
    }

    pub fn next_token_contains(&self, value: Vec<&str>) -> bool {
        match self.peek() {
            Some(t) => value.contains(&t.value()),
            None => false,
        }
    }

    fn peek(&self) -> Option<&Token> {
        if !self.tokens.is_empty() {
            Some(self.tokens.front()?)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Result<Token, SyntaxError> {
        if !self.tokens.is_empty() {
            Ok(self.tokens.pop_front().unwrap())
        } else {
            Err(SyntaxError::MissingToken(MissingToken::new(None)))
        }
    }

    pub fn print_queue(&self) {
        println!();
        for tok in &self.tokens {
            println!("{}: {}", tok.tag().to_string(), tok.value());
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(Identifier),
    Literal(Literal),
    Symbol(Symbol),
}

impl Token {
    pub fn value(&self) -> &str {
        match self {
            Token::Keyword(t) => t.value.as_str(),
            Token::Identifier(t) => t.value.as_str(),
            Token::Literal(t) => t.value.as_str(),
            Token::Symbol(t) => t.value.as_str(),
        }
    }

    pub fn tag(&self) -> TokenTag {
        match self {
            Token::Keyword(_) => TokenTag::Keyword,
            Token::Identifier(_) => TokenTag::Identifier,
            Token::Literal(_) => TokenTag::Literal,
            Token::Symbol(_) => TokenTag::Symbol,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TokenTag {
    Keyword,
    Identifier,
    Literal,
    Symbol,
    None,
}

impl TokenTag {
    pub fn to_string(&self) -> String {
        match self {
            TokenTag::Keyword => "Keyword".to_string(),
            TokenTag::Identifier => "Identifier".to_string(),
            TokenTag::Literal => "Literal".to_string(),
            TokenTag::Symbol => "Symbol".to_string(),
            TokenTag::None => "None".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Keyword {
    pub value: String,
}

impl Keyword {
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    pub fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub value: String,
    pub literal_type: LiteralType,
}

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub value: String,
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
            TokenQueue::new(vec![Token::Literal(Literal {
                value: "abc".to_string(),
                literal_type: LiteralType::String,
            })])
        )
    }

    #[test]
    fn test_int_literal() {
        assert_eq!(
            tokenize("123").unwrap(),
            TokenQueue::new(vec![Token::Literal(Literal {
                value: "123".to_string(),
                literal_type: LiteralType::Integer
            })])
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
                Token::Literal(Literal {
                    value: "123".to_string(),
                    literal_type: LiteralType::Integer
                }),
                Token::Literal(Literal {
                    value: "abc".to_string(),
                    literal_type: LiteralType::String,
                }),
                Token::Literal(Literal {
                    value: "456".to_string(),
                    literal_type: LiteralType::Integer
                }),
                Token::Literal(Literal {
                    value: "def".to_string(),
                    literal_type: LiteralType::String,
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
                Token::Literal(Literal {
                    value: "def".to_string(),
                    literal_type: LiteralType::String,
                }),
                Token::Identifier(Identifier {
                    value: "ghi".to_string()
                }),
                Token::Literal(Literal {
                    value: "jkl".to_string(),
                    literal_type: LiteralType::String,
                }),
            ])
        )
    }
}
