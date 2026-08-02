use std::{
    collections::VecDeque,
    io::{self, Error},
    os::raw::c_int,
};

pub fn tokenize(text: &str) -> Vec<Token> {
    let separated_tokens = seperate_tokens(text).unwrap_or(vec![]);
    tokenize_strings(separated_tokens)
}

fn tokenize_strings(strings: Vec<String>) -> Vec<Token> {
    let mut queue = VecDeque::from(strings);
    let mut tokens: Vec<Token> = vec![];

    while queue.front() != None {
        let sequence: Vec<String> = vec![];

        let next = queue.pop_front();

        if next.unwrap().as_str() == "CREATE" {
            match queue.pop_front().unwrap().as_str() {
                "TABLE" => {
                    let token = Token::Keyword(Keyword::new("CREATE TABLE".to_string()));
                    tokens.push(token);
                }

                _ => {}
            }
        }
    }
    vec![]
}

fn pop_until(mut queue: VecDeque<String>, value: &str) -> (VecDeque<String>, Vec<String>) {
    let mut slice: VecDeque<String> = VecDeque::new();

    for item in queue {
        if item != value.to_string() {
            slice.push_back(item);
        } else {
            break;
        }
    }

    (VecDeque::new(), vec![])
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
    StringLit(StringLiteral),
    IntLit(IntLiteral),
    Symbol(Symbol),
}

struct Keyword {
    value: String,
}

impl Keyword {
    fn new(value: String) -> Keyword {
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
