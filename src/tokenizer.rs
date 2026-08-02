use std::{
    collections::VecDeque,
    ffi::c_int,
    io::{self, Error},
};

pub fn tokenize(text: &str) -> Vec<Token> {
    let separated_tokens = seperate_tokens(text).unwrap_or(vec![]);
    tokenize_strings(separated_tokens)
}

fn tokenize_strings(strings: Vec<String>) -> Vec<Token> {
    let queue = VecDeque::from(strings);

    let tokens = match queue.front().unwrap().as_str() {
        "CREATE" => tokenize_create_table(queue).unwrap(),
        "INSERT" => tokenize_insert(queue).unwrap(),
        "SELECT" => tokenize_select(queue).unwrap(),
        _ => vec![],
    };

    tokens
}

fn tokenize_create_table(mut queue: VecDeque<String>) -> Result<Vec<Token>, io::Error> {
    let mut tokens: Vec<Token> = vec![];
    if queue.pop_front().unwrap().as_str() != "CREATE"
        || queue.pop_front().unwrap().as_str() != "TABLE"
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    // Get the table name name
    if queue.front().unwrap().as_str() == "DOUBLEQUOTE" {
        // Create a slice of everything between quotes
        let mut slice = pop_until(&mut queue, "DOUBLEQUOTE");

        let token = match parse_enclosed_quotes(&mut slice) {
            Ok(t) => t,
            Err(_) => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
            }
        };

        tokens.push(token);
    } else {
        let token = Token::Keyword(Keyword::new(queue.pop_front().unwrap()));
        tokens.push(token);
    }

    if queue.pop_front().unwrap().as_str() != "LPAREN" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    // Get the columns
    // First get everything between the parenthesis
    let mut slice = pop_until(&mut queue, "RPAREN");
    'slice: loop {
        // Discard the item if its the starting paren
        if slice.front().unwrap().as_str() == "LPAREN" {
            slice.pop_front();
            continue;
        // Break if its the ending paren
        } else if slice.front().unwrap().as_str() == "RPAREN" {
            slice.pop_front();
            break;
        }

        // Grab the column name first
        let item = slice.pop_front().unwrap();

        // Push the column name identifier
        tokens.push(Token::Identifier(Identifier::new(item)));

        // Check for any and all constraints
        'condtions: loop {
            let next = slice.pop_front().unwrap();
            match next.as_str() {
                "COMMA" => break 'condtions,
                "RPAREN" => break 'slice,
                "TEXT" | "INTEGER" => tokens.push(Token::Constraint(Constraint::new(next))),
                "PRIMARY" => {
                    if slice.front().unwrap().as_str() == "KEY" {
                        tokens.push(Token::Constraint(Constraint::new(
                            "PRIMARY KEY".to_string(),
                        )));
                        // Consume the following token
                        slice.pop_front();
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                    }
                }
                &_ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                }
            }
        }
    }

    if queue.pop_front().unwrap().as_str() == "SEMICOLON" {
        Ok(tokens)
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }
}

fn tokenize_insert(mut queue: VecDeque<String>) -> Result<Vec<Token>, io::Error> {
    let mut tokens: Vec<Token> = vec![];

    //Check the first two tokens
    let first = queue.pop_front().unwrap();
    let second = queue.pop_front().unwrap();
    if first != "INSERT".to_string() || &second != &"INTO".to_string() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    // Get the table name
    let token = Token::Keyword(Keyword::new(queue.pop_front().unwrap()));
    tokens.push(token);

    if queue.pop_front().unwrap().as_str() != "VALUES" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    if queue.front().unwrap().as_str() != "LPAREN" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    // Get the values
    // First get everything between the parenthesis
    let mut slice = pop_until(&mut queue, "RPAREN");
    'slice: loop {
        // Discard the item if its the starting paren
        if slice.front().unwrap().as_str() == "LPAREN" {
            slice.pop_front();
            continue 'slice;

        // Consume the token if its a comma
        } else if slice.front().unwrap().as_str() == "COMMA" {
            slice.pop_front();
            continue 'slice;
        // Break if its the ending paren
        } else if slice.front().unwrap().as_str() == "RPAREN" {
            slice.pop_front();
            break 'slice;
        }

        if slice.front().unwrap().as_str() == "SINGLEQUOTE" {
            // Create a subslice of everything between quotes
            let mut subslice = pop_until(&mut slice, "SINGLEQUOTE");
            let token = match parse_enclosed_quotes(&mut subslice) {
                Ok(t) => t,
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                }
            };

            tokens.push(token);
            continue 'slice;
        }

        // Parse as an integer literal if it can be parsed that way
        if slice.front().unwrap().parse::<c_int>().is_ok() {
            tokens.push(Token::IntLit(IntLiteral {
                value: slice.pop_front().unwrap().parse::<c_int>().unwrap(),
            }));
            continue 'slice;
        }
    }

    if queue.pop_front().unwrap().as_str() == "SEMICOLON" {
        Ok(tokens)
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }
}

fn tokenize_select(mut queue: VecDeque<String>) -> Result<Vec<Token>, io::Error> {
    let mut tokens: Vec<Token> = vec![];

    //Check the first two tokens
    let first = queue.pop_front().unwrap();
    if first != "SELECT".to_string() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    }

    // Get the columns
    let mut slice = pop_until(&mut queue, "FROM");

    'slice: loop {
        // Break out when we hit the FROM block
        // FROM is consumed by the pop_until so it is no longer in the original queue
        if slice.front().unwrap().as_str() == "FROM" {
            break 'slice;
        // If the slice is empty, it means we never hit the FROM block
        } else if slice.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
        }

        tokens.push(Token::Identifier(Identifier {
            value: slice.pop_front().unwrap(),
        }));
    }

    // Get the table name
    let token = Token::Identifier(Identifier::new(queue.pop_front().unwrap()));
    tokens.push(token);

    if queue.front().unwrap().as_str() == "WHERE" {
        loop {
            // End of statement
            if queue.front().unwrap().as_str() == "SEMICOLON" {
                break;
            }

            // Push the identifier of the clause
            if queue.front().unwrap().as_str() == "DOUBLEQUOTE" {
                // Create a subslice of everything between quotes
                let mut slice = pop_until(&mut queue, "DOUBLEQUOTE");

                let token = match parse_enclosed_quotes(&mut slice) {
                    Ok(t) => t,
                    Err(_) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                    }
                };

                tokens.push(token);
            }

            // Push the operator of the caluse
            tokens.push(Token::Symbol(Symbol::new(queue.pop_front().unwrap())));

            // Push the literal

            if slice.front().unwrap().as_str() == "SINGLEQUOTE" {
                // Create a subslice of everything between quotes
                let mut slice = pop_until(&mut slice, "SINGLEQUOTE");
                let token = match parse_enclosed_quotes(&mut slice) {
                    Ok(t) => t,
                    Err(_) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
                    }
                };

                tokens.push(token);
            }

            // Parse as an integer literal if it can be parsed that way
            if slice.front().unwrap().parse::<c_int>().is_ok() {
                tokens.push(Token::IntLit(IntLiteral {
                    value: slice.pop_front().unwrap().parse::<c_int>().unwrap(),
                }));
            }
        }
        Ok(tokens)
    } else if queue.front().unwrap().as_str() == "SEMICOLON" {
        Ok(tokens)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"))
    }
}

fn parse_enclosed_quotes(queue_slice: &mut VecDeque<String>) -> Result<Token, io::Error> {
    let mut quote_type: String = String::new();
    if queue_slice.front().unwrap().as_str() == "SINGLEQUOTE" {
        quote_type = "SINGLEQUOTE".to_string();
    } else if queue_slice.front().unwrap().as_str() == "DOUBLEQUOTE" {
        quote_type = "DOUBLEQUOTE".to_string();
    }
    if queue_slice.front().unwrap().as_str() == quote_type {
        let mut substring = String::new();

        // Combine everything between quotes into one string
        loop {
            // Discard the quotes
            if queue_slice.front().unwrap().as_str() == quote_type {
                queue_slice.pop_front();
                continue;
            }
            substring.push_str(queue_slice.pop_front().unwrap().as_str());

            if queue_slice.is_empty() {
                break;
            } else {
                substring.push_str(" ");
            }
        }

        if quote_type == "DOUBLEQUOTE" {
            return Ok(Token::Identifier(Identifier::new(substring)));
        } else if quote_type == "SINGLEQUOTE" {
            return Ok(Token::StringLit(StringLiteral::new(substring)));
        }
    }

    Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"))
}

fn pop_until(queue: &mut VecDeque<String>, value: &str) -> VecDeque<String> {
    let mut slice: VecDeque<String> = VecDeque::new();

    // pop the starting item
    slice.push_back(queue.pop_front().unwrap());

    while queue.front().unwrap().as_str() != value {
        // Shift the first value from the queue onto the back of the slice
        slice.push_back(queue.pop_front().unwrap());
    }

    // pop the ending item
    slice.push_back(queue.pop_front().unwrap());

    slice
}

// TODO: Error Handling
pub fn seperate_tokens(text: &str) -> Result<Vec<String>, Error> {
    // convert_strings(split_text)
    let mut token_strings: Vec<String> = vec![];
    let characters: Vec<char> = text.chars().collect();

    let mut index: usize = 0;

    // Main parsing loop
    'parsing: loop {
        // Break out if we're at the end
        if index >= characters.len() {
            break 'parsing;
        }

        // Initialize a new token
        let mut current_token: String = String::new();

        // Get the next character in the string
        let current_char = characters[index];

        // Skip over white space
        if char::is_whitespace(current_char) {
            // Increment the index for the next token
            index += 1;
            continue 'parsing;
        }

        if char::is_alphanumeric(current_char) {
            //Build the current token with all consecutive alphanumeric
            while index < characters.len() && char::is_alphanumeric(characters[index]) {
                current_token.push(characters[index]);

                println!("Processing Character: {}", characters[index]);

                // Increment for the next char
                index += 1;
            }

            // Store the current token
            println!("Storing token: {}", &current_token);
            token_strings.push(current_token.clone());

            // Early exit if we're at the end
            if index >= characters.len() {
                break;
            }
            continue 'parsing;
        }

        if !char::is_alphanumeric(current_char) {
            // Parse symbols
            // The function handles incrementing the index itself
            // This is so that it can handle compound symbols like >= and <=
            current_token = tokenize_non_alphanumeric(&characters, &mut index);

            // Store the token
            println!("Storing token: {}", &current_token);
            token_strings.push(current_token.clone());
        } else {
            return Err(io::Error::other("Non-parsable character encountered"));
        }
    }

    Ok(token_strings)
}

fn tokenize_non_alphanumeric(characters: &[char], index: &mut usize) -> String {
    let mut current_token: String;
    match characters[*index] {
        '<' => {
            current_token = String::from("LESSTHAN");
        }
        '>' => {
            current_token = String::from("GREATERTHAN");
        }
        '(' => {
            current_token = String::from("LPAREN");
        }
        ')' => {
            current_token = String::from("RPAREN");
        }
        ',' => {
            current_token = String::from("COMMA");
        }
        '*' => {
            current_token = String::from("STAR");
        }
        '=' => {
            current_token = String::from("EQUALS");
        }
        '+' => {
            current_token = String::from("PLUS");
        }
        '-' => {
            current_token = String::from("MINUS");
        }
        ';' => {
            current_token = String::from("SEMICOLON");
        }
        '\'' => {
            current_token = String::from("SINGLEQUOTE");
        }
        '"' => {
            current_token = String::from("DOUBLEQUOTE");
        }
        _ => {
            current_token = String::from("UNK");
        }
    }

    *index += 1;

    // Handle the compound comparisons
    if index < &mut characters.len() && characters[*index] == '=' {
        match characters[*index - 1] {
            '<' | '>' | '+' | '-' => {
                current_token.push_str("EQUALS");
                *index += 1;
            }
            _ => {}
        }
    }

    // Return our constructed token
    current_token
}

enum Token {
    Keyword(Keyword),
    Identifier(Identifier),
    Constraint(Constraint),
    StringLit(StringLiteral),
    IntLit(IntLiteral),
    Symbol(Symbol),
}

struct Keyword {
    value: String,
}

impl Keyword {
    pub fn new(value: String) -> Keyword {
        Keyword { value }
    }
}

struct Identifier {
    value: String,
}

impl Identifier {
    fn new(value: String) -> Identifier {
        Identifier { value }
    }
}

struct Constraint {
    value: String,
}

impl Constraint {
    pub fn new(value: String) -> Constraint {
        Constraint { value }
    }
}

struct StringLiteral {
    value: String,
}

impl StringLiteral {
    fn new(value: String) -> StringLiteral {
        StringLiteral { value }
    }
}

struct IntLiteral {
    value: c_int,
}

impl IntLiteral {
    fn new(value: c_int) -> IntLiteral {
        IntLiteral { value }
    }
}

struct Symbol {
    value: String,
}

impl Symbol {
    fn new(value: String) -> Symbol {
        Symbol { value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_split() {
        assert_eq!(
            seperate_tokens("*abc -def").unwrap(),
            vec!["STAR", "abc", "MINUS", "def"]
        )
    }

    #[test]
    fn test_suffix_split() {
        assert_eq!(
            seperate_tokens("abc+ def)").unwrap(),
            vec!["abc", "PLUS", "def", "RPAREN"]
        )
    }

    #[test]
    fn test_prefix_and_suffix_split() {
        assert_eq!(
            seperate_tokens("(xyz) (abc,").unwrap(),
            vec!["LPAREN", "xyz", "RPAREN", "LPAREN", "abc", "COMMA"]
        )
    }

    #[test]
    fn mixed_alphanumeric_no_split() {
        assert_eq!(seperate_tokens("123abc").unwrap(), vec!["123abc"]);
    }

    #[test]
    fn test_numeric_with_suffix() {
        assert_eq!(
            seperate_tokens("(123) (123abc").unwrap(),
            vec!["LPAREN", "123", "RPAREN", "LPAREN", "123abc"]
        )
    }

    #[test]
    fn test_long_command() {
        assert_eq!(
            seperate_tokens("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);")
                .unwrap(),
            vec![
                "CREATE",
                "TABLE",
                "users",
                "LPAREN",
                "id",
                "INTEGER",
                "PRIMARY",
                "KEY",
                "COMMA",
                "name",
                "TEXT",
                "COMMA",
                "age",
                "INTEGER",
                "RPAREN",
                "SEMICOLON"
            ]
        )
    }

    #[test]
    fn test_full_insert_command() {
        assert_eq!(
            seperate_tokens("INSERT INTO users VALUES (1, 'daniel', 24);").unwrap(),
            vec![
                "INSERT",
                "INTO",
                "users",
                "VALUES",
                "LPAREN",
                "1",
                "COMMA",
                "SINGLEQUOTE",
                "daniel",
                "SINGLEQUOTE",
                "COMMA",
                "24",
                "RPAREN",
                "SEMICOLON"
            ]
        )
    }

    #[test]
    fn test_full_select_command_with_wildcard() {
        assert_eq!(
            seperate_tokens("SELECT * FROM users;").unwrap(),
            vec!["SELECT", "STAR", "FROM", "users", "SEMICOLON"]
        )
    }

    #[test]
    fn test_full_select_command_with_columns_and_clause() {
        assert_eq!(
            seperate_tokens("SELECT name, age FROM users WHERE id = 1;").unwrap(),
            vec![
                "SELECT",
                "name",
                "COMMA",
                "age",
                "FROM",
                "users",
                "WHERE",
                "id",
                "EQUALS",
                "1",
                "SEMICOLON"
            ]
        )
    }

    #[test]
    fn test_full_select_command_with_wildcard_and_clause() {
        assert_eq!(
            seperate_tokens("SELECT * FROM users WHERE age > 20;").unwrap(),
            vec![
                "SELECT",
                "STAR",
                "FROM",
                "users",
                "WHERE",
                "age",
                "GREATERTHAN",
                "20",
                "SEMICOLON"
            ]
        )
    }
}
