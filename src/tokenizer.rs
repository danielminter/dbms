use std::{
    fmt::Error,
    io::{self, Error},
};

fn convert_strings(strings: Vec<&str>) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();

    for mut word in strings {
        let mut prefix: Vec<Option<char>> = vec![None];
        let mut suffix: Vec<Option<char>> = vec![None];
        // Split off the first character if it isn't alphanumeric
        // Loop until we hit alphanumeric
        while !word.chars().next().unwrap().is_alphanumeric() {
            prefix.push(Some(word.chars().next().unwrap()));
            word = &word[1..];
        }

        // Split off the last character if it isn't alphabetic
        // Loop until we hit alphanumeric
        while !word.chars().last().unwrap().is_alphanumeric() {
            suffix.insert(0, Some(word.chars().last().unwrap()));
            word = &word[..(word.len() - 1)];
        }

        // Push each prefix in order onto the token stack
        for item in prefix {
            if let Some(item) = item {
                tokens.push(String::from(item));
            }
        }

        // Push the token onto the token vector
        tokens.push(String::from(word));

        // Push each suffix in order onto the token stack
        for item in suffix {
            if let Some(item) = item {
                tokens.push(String::from(item));
            }
        }
    }

    tokens
}

fn convert_to_tokens(strings: Vec<String>) -> Result<Vec<Token>, io::Error> {
    match strings[0].as_str() {
        "SELECT" => {
            return generate_select_command(strings);
        }

        "INSERT" => {
            return generate_insert_command(strings);
        }

        "CREATE" => {
            return generate_create_command(strings);
        }

        &_ => todo!(),
    }

    Ok(vec![])
}

fn generate_select_command(command: Vec<String>) -> Result<Vec<Token>, io::Error> {
    // Should never happen but kept as a backstop
    if command[0] != "SELECT" {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Expected SELECT, got something else",
        ));
    }

    // Generate an empty select statement
    let mut statement = SelectStatement::new();

    // Parse the remaining tokens
    let mut index = 0;

    // Main loop over all of the tokens
    while index < command.len() {
        // Consume all tokens up to "FROM"
        while command[index] != "FROM" {
            if index >= command.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "No FROM keyword found after SELECT",
                ));
            }

            statement.add_column(&command[index]);

            index += 1;
        }

        // Skip over FROM
        index += 1;

        // Expect a table name next
        statement.set_table(&command[index]);
        index += 1;

        // TODO: Move away from manually incrementing the index.
        if command[index] == "WHERE" {
            while command[index] != ";" {
                let mut clause = Clause::new();
                index += 1;
                clause.set_key(&command[index]);
                index += 1;
                clause.set_operator(&command[index]);
                index += 1;
                clause.set_value(&command[index]);
                index += 1;
            }
        }
    }

    Ok(vec![])
}

fn generate_insert_command(command: Vec<String>) -> Result<Vec<Token>, io::Error> {
    Ok(vec![])
}

fn generate_create_command(command: Vec<String>) -> Result<Vec<Token>, io::Error> {
    Ok(vec![])
}

pub fn tokenize(text: &str) -> Vec<Token> {
    // Split the text on spaces
    let split_text = text.split(" ").collect();

    // Seperate symbols from alphanumberic
    let strings_vector = convert_strings(split_text);

    // Convert the strings to tokens
    convert_to_tokens(strings_vector)
}

pub struct Token {
    token_type: TokenType,
    value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Token {
        Token { token_type, value }
    }
}

pub struct CreateStatement {
    table_name: String,
    columns: Vec<Column>,
}

impl CreateStatement {
    fn new() -> CreateStatement {
        CreateStatement {
            table_name: String::new(),
            columns: Vec::new(),
        }
    }
}

pub struct InsertStatement {
    target: String,
    values: Vec<String>,
}

impl InsertStatement {
    fn new() -> InsertStatement {
        InsertStatement {
            target: String::new(),
            values: Vec::new(),
        }
    }
}

pub struct SelectStatement {
    column_labels: Vec<String>,
    target: String,
    clauses: Vec<String>,
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
