use std::{
    collections::VecDeque,
    io::{self},
    thread::current,
};

use crate::{
    errors::{InvalidToken, InvalidValue, MissingToken, SyntaxError, UnexpectedSymbol},
    parser::LiteralType,
};

pub fn tokenize(text: &str) -> Result<TokenQueue, io::Error> {
    let mut characters: VecDeque<char> = text.trim().chars().collect();

    // Turn the vecdeque into a vec of individual strings
    let mut statement_strings = statement_to_strings(&mut characters);

    // Convert the strings to tokens
    let tokens: VecDeque<Token> = transform_to_tokens(&mut statement_strings);

    Ok(TokenQueue::new(tokens))
}

fn transform_to_tokens(strings: &mut VecDeque<(String, StringKind)>) -> VecDeque<Token> {
    let mut result_queue: VecDeque<Token> = VecDeque::new();
    while !strings.is_empty() {
        let (value, current_string_type) = strings.pop_front().unwrap();
        match current_string_type {
            StringKind::Keyword => {
                result_queue.push_back(Token::Keyword(Keyword { value }));
            }
            StringKind::Identifier => {
                result_queue.push_back(Token::Identifier(Identifier { value }));
            }
            StringKind::Literal => {
                let mut literal_type = LiteralType::String;
                if !value.chars().next().unwrap().is_ascii_alphabetic() {
                    literal_type = LiteralType::Integer;
                }
                result_queue.push_back(Token::Literal(Literal {
                    value,
                    literal_type,
                }));
            }
            StringKind::Symbol => {
                result_queue.push_back(Token::Symbol(transform_symbol(value)));
            }
            StringKind::None => todo!(),
        }
    }

    result_queue
}

fn statement_to_strings(strings: &mut VecDeque<char>) -> VecDeque<(String, StringKind)> {
    let mut result_vec: VecDeque<(String, StringKind)> = VecDeque::new();
    while !strings.is_empty() {
        result_vec.push_back(parse_single_string(strings));
    }

    result_vec
}

fn parse_single_string(characters: &mut VecDeque<char>) -> (String, StringKind) {
    let next_char = characters.front().unwrap();
    // Match on the next character to determine what type the next string is
    match classify_char(next_char) {
        // Remove the whitespace and recursively call itself to keep parsing
        CharKind::Whitespace => {
            characters.pop_front();
            parse_single_string(characters)
        }
        CharKind::Lowercase => (
            grab_until(characters, |c| !c.is_ascii_lowercase()),
            StringKind::Identifier,
        ),
        CharKind::Uppercase => (
            grab_until(characters, |c| !c.is_ascii_uppercase()),
            StringKind::Keyword,
        ),
        CharKind::Numeric => (
            grab_until(characters, |c| !c.is_ascii_alphanumeric()),
            StringKind::Literal,
        ),
        // ' or " => literal
        CharKind::Literal => {
            // Remove the first quotation mark
            let string_kind = match characters.pop_front().unwrap() {
                '\'' => StringKind::Literal,
                '\"' => StringKind::Identifier,
                _ => StringKind::None,
            };
            let result = (
                grab_until(characters, |c| c == '\'' || c == '\"'),
                string_kind,
            );
            //Consume the trailing quotation
            characters.pop_front();

            result
        }
        // punctuation => symbol, The empty closure will force a single character to be parsed every
        // time
        CharKind::Symbol => (grab_until(characters, |_| true), StringKind::Symbol),
        // None as default
        CharKind::None => (String::new(), StringKind::None),
    }
}

fn grab_until<F>(characters: &mut VecDeque<char>, stop_signal: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut result = String::new();

    // Get the first character
    let mut current_char = match characters.front() {
        Some(_) => characters.pop_front().unwrap(),
        None => return result,
    };
    result.push(current_char);

    // Loop while the next character isn't our stop signal
    while !characters.is_empty() {
        current_char = match characters.front() {
            Some(c) => *c,
            None => return result,
        };
        if stop_signal(current_char) {
            break;
        }
        result.push(characters.pop_front().unwrap());
    }

    result
}

enum StringKind {
    Keyword,
    Identifier,
    Literal,
    Symbol,
    None,
}

enum CharKind {
    Whitespace,
    Uppercase,
    Lowercase,
    Numeric,
    Literal,
    Symbol,
    None,
}

fn classify_char(c: &char) -> CharKind {
    if c.is_whitespace() {
        CharKind::Whitespace
    } else if c.is_ascii_lowercase() {
        CharKind::Lowercase
    } else if c.is_ascii_uppercase() {
        CharKind::Uppercase
    } else if c.is_ascii_alphanumeric() {
        CharKind::Numeric
    } else if *c == '\'' || *c == '\"' {
        CharKind::Literal
    } else if c.is_ascii_punctuation() {
        CharKind::Symbol
    } else {
        CharKind::None
    }
}

fn transform_symbol(value: String) -> SymbolType {
    match value.as_str() {
        "<" => SymbolType::LessThan,
        ">" => SymbolType::GreaterThan,
        "+" => SymbolType::Plus,
        "-" => SymbolType::Minus,
        "(" => SymbolType::LParen,
        ")" => SymbolType::RParen,
        "," => SymbolType::Comma,
        "*" => SymbolType::Star,
        "=" => SymbolType::Equals,
        ";" => SymbolType::Semicolon,
        _ => SymbolType::Unknown,
    }
}

fn print_characters(characters: &mut VecDeque<char>) {
    println!();
    for character in characters {
        print!("{}", character);
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenQueue {
    tokens: VecDeque<Token>,
}

impl TokenQueue {
    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    pub fn new(tokens: VecDeque<Token>) -> TokenQueue {
        TokenQueue { tokens }
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

    pub fn check_next_symbol(&self, symbol_type: Vec<SymbolType>) -> bool {
        match self.peek() {
            Some(Token::Symbol(val)) => symbol_type.contains(val),
            _ => false,
        }
    }

    pub fn verify_symbol_type(
        &mut self,
        symbol_type: Vec<SymbolType>,
    ) -> Result<Token, SyntaxError> {
        match self.peek() {
            Some(Token::Symbol(t)) => match symbol_type.contains(t) {
                true => Ok(self.next()?),
                false => Err(SyntaxError::UnexpectedSymbol(UnexpectedSymbol::new(t))),
            },
            Some(val) => Err(SyntaxError::InvalidToken(InvalidToken::new(
                TokenTag::Symbol,
                val.tag(),
            ))),
            None => Err(SyntaxError::MissingToken(MissingToken::new(Some(
                TokenTag::Symbol,
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
                    t.value().to_string(),
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
    Symbol(SymbolType),
}

impl Token {
    pub fn value(&self) -> &str {
        match self {
            Token::Keyword(t) => t.value.as_str(),
            Token::Identifier(t) => t.value.as_str(),
            Token::Literal(t) => t.value.as_str(),
            Token::Symbol(t) => t.print_type(),
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolType {
    Star,
    Plus,
    Minus,
    Comma,
    Equals,
    LParen,
    RParen,
    Semicolon,
    LessThan,
    GreaterThan,
    Unknown,
}

impl SymbolType {
    pub fn print_type(&self) -> &str {
        match self {
            SymbolType::Star => "Star",
            SymbolType::Plus => "Plus",
            SymbolType::Minus => "Minus",
            SymbolType::Comma => "Comma",
            SymbolType::Equals => "Equals",
            SymbolType::LParen => "LParen",
            SymbolType::RParen => "RParen",
            SymbolType::Semicolon => "Semicolon",
            SymbolType::LessThan => "LessThan",
            SymbolType::GreaterThan => "GreaterThan",
            SymbolType::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        assert_eq!(
            tokenize("INSERT").unwrap(),
            TokenQueue::new(VecDeque::from([Token::Keyword(Keyword {
                value: "INSERT".to_string()
            })]),)
        )
    }

    #[test]
    fn test_no_quote_identifier() {
        assert_eq!(
            tokenize("abc").unwrap(),
            TokenQueue::new(VecDeque::from([Token::Identifier(Identifier {
                value: "abc".to_string()
            })]))
        )
    }

    #[test]
    fn test_quoted_identifier() {
        assert_eq!(
            tokenize("\"two words\"").unwrap(),
            TokenQueue::new(VecDeque::from([Token::Identifier(Identifier {
                value: "two words".to_string()
            })]))
        )
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            tokenize("\'abc\'").unwrap(),
            TokenQueue::new(VecDeque::from([Token::Literal(Literal {
                value: "abc".to_string(),
                literal_type: LiteralType::String,
            })]))
        )
    }

    #[test]
    fn test_int_literal() {
        assert_eq!(
            tokenize("123").unwrap(),
            TokenQueue::new(VecDeque::from([Token::Literal(Literal {
                value: "123".to_string(),
                literal_type: LiteralType::Integer
            })]))
        )
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            tokenize("< > ( ) * , = + - ;").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Symbol(SymbolType::LessThan),
                Token::Symbol(SymbolType::GreaterThan),
                Token::Symbol(SymbolType::LParen),
                Token::Symbol(SymbolType::RParen),
                Token::Symbol(SymbolType::Star),
                Token::Symbol(SymbolType::Comma),
                Token::Symbol(SymbolType::Equals),
                Token::Symbol(SymbolType::Plus),
                Token::Symbol(SymbolType::Minus),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }

    #[test]
    fn test_compound_symbols() {
        assert_eq!(
            tokenize("<= >= += -=").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Symbol(SymbolType::LessThan),
                Token::Symbol(SymbolType::Equals),
                Token::Symbol(SymbolType::GreaterThan),
                Token::Symbol(SymbolType::Equals),
                Token::Symbol(SymbolType::Plus),
                Token::Symbol(SymbolType::Equals),
                Token::Symbol(SymbolType::Minus),
                Token::Symbol(SymbolType::Equals),
            ]))
        )
    }

    #[test]
    fn test_mixed_literals() {
        assert_eq!(
            tokenize("123 'abc' 456 'def'").unwrap(),
            TokenQueue::new(VecDeque::from([
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
            ]))
        )
    }

    #[test]
    fn test_mixed_quotes() {
        assert_eq!(
            tokenize("\"abc\" 'def' \"ghi\" 'jkl'").unwrap(),
            TokenQueue::new(VecDeque::from([
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
            ]))
        )
    }

    #[test]
    fn test_create_command() {
        assert_eq!(
            tokenize("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);")
                .unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Keyword(Keyword {
                    value: "CREATE".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "TABLE".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "users".to_string()
                }),
                Token::Symbol(SymbolType::LParen),
                Token::Identifier(Identifier {
                    value: "id".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "INTEGER".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "PRIMARY".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "KEY".to_string()
                }),
                Token::Symbol(SymbolType::Comma),
                Token::Identifier(Identifier {
                    value: "name".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "TEXT".to_string()
                }),
                Token::Symbol(SymbolType::Comma),
                Token::Identifier(Identifier {
                    value: "age".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "INTEGER".to_string()
                }),
                Token::Symbol(SymbolType::RParen),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }

    #[test]
    fn test_insert_command() {
        assert_eq!(
            tokenize("INSERT INTO users VALUES (1, 'daniel', 24);").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Keyword(Keyword {
                    value: "INSERT".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "INTO".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "users".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "VALUES".to_string()
                }),
                Token::Symbol(SymbolType::LParen),
                Token::Literal(Literal {
                    value: "1".to_string(),
                    literal_type: LiteralType::Integer
                }),
                Token::Symbol(SymbolType::Comma),
                Token::Literal(Literal {
                    value: "daniel".to_string(),
                    literal_type: LiteralType::String
                }),
                Token::Symbol(SymbolType::Comma),
                Token::Literal(Literal {
                    value: "24".to_string(),
                    literal_type: LiteralType::Integer
                }),
                Token::Symbol(SymbolType::RParen),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }

    #[test]
    fn test_select_command_with_wildcard() {
        assert_eq!(
            tokenize("SELECT * FROM users;").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Keyword(Keyword {
                    value: "SELECT".to_string()
                }),
                Token::Symbol(SymbolType::Star),
                Token::Keyword(Keyword {
                    value: "FROM".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "users".to_string()
                }),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }

    #[test]
    fn test_select_command_with_conditions() {
        assert_eq!(
            tokenize("SELECT name, age FROM users WHERE id = 1;").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Keyword(Keyword {
                    value: "SELECT".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "name".to_string()
                }),
                Token::Symbol(SymbolType::Comma),
                Token::Identifier(Identifier {
                    value: "age".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "FROM".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "users".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "WHERE".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "id".to_string()
                }),
                Token::Symbol(SymbolType::Equals),
                Token::Literal(Literal {
                    value: "1".to_string(),
                    literal_type: LiteralType::Integer,
                }),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }

    #[test]
    fn test_select_command_with_conditions_and_wildcard() {
        assert_eq!(
            tokenize("SELECT * FROM users WHERE age > 20;").unwrap(),
            TokenQueue::new(VecDeque::from([
                Token::Keyword(Keyword {
                    value: "SELECT".to_string()
                }),
                Token::Symbol(SymbolType::Star),
                Token::Keyword(Keyword {
                    value: "FROM".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "users".to_string()
                }),
                Token::Keyword(Keyword {
                    value: "WHERE".to_string()
                }),
                Token::Identifier(Identifier {
                    value: "age".to_string()
                }),
                Token::Symbol(SymbolType::GreaterThan),
                Token::Literal(Literal {
                    value: "20".to_string(),
                    literal_type: LiteralType::Integer,
                }),
                Token::Symbol(SymbolType::Semicolon),
            ]))
        )
    }
}
