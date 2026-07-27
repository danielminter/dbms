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

impl SelectStatement {
    fn new() -> SelectStatement {
        SelectStatement {
            column_labels: Vec::new(),
            target: String::new(),
            clauses: Vec::new(),
        }
    }

    pub fn add_column(&mut self, label: &str) {
        self.column_labels.push(label.to_string());
    }

    pub fn set_table(&mut self, table: &str) {
        self.target = table.to_string();
    }

    pub fn add_clause(&mut self, clause: &str) {
        self.clauses.push(clause.to_string());
    }
}

pub struct Column {
    label: String,
    column_type: ColumnType,
    is_primary_key: bool,
}

pub struct Clause {
    key: String,
    operator: String,
    value: String,
}

impl Clause {
    pub fn new() -> Clause {
        Clause {
            key: String::new(),
            operator: String::new(),
            value: String::new(),
        }
    }

    pub fn set_key(&mut self, key: &str) {
        self.key = key.to_string();
    }

    pub fn set_operator(&mut self, operator: &str) {
        self.operator = operator.to_string();
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }
}

pub enum ColumnType {
    TEXT,
    INTEGER,
}

pub enum Statement {
    CREATE(CreateStatement),
    INSERT(InsertStatement),
    SELECT(SelectStatement),
}

pub enum TokenType {
    KEYWORD,
    IDENTIFIER,
    LITERAL,
    PUNCTUATION,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_split() {
        assert_eq!(
            convert_strings(vec!["#abc", "{def"]),
            vec!["#", "abc", "{", "def"]
        )
    }

    #[test]
    fn test_suffix_split() {
        assert_eq!(
            convert_strings(vec!["abc#", "def)"]),
            vec!["abc", "#", "def", ")"]
        )
    }

    #[test]
    fn test_prefix_and_suffix_split() {
        assert_eq!(
            convert_strings(vec!["(xyz)", "(abc,"]),
            vec!["(", "xyz", ")", "(", "abc", ","]
        )
    }

    #[test]
    fn mixed_alphanumeric_no_split() {
        assert_eq!(convert_strings(vec!["123abc"]), vec!["123abc"]);
    }

    #[test]
    fn test_numeric_with_suffix() {
        assert_eq!(
            convert_strings(vec!["(123)", "(123abc"]),
            vec!["(", "123", ")", "(", "123abc"]
        )
    }

    #[test]
    fn test_long_command() {
        assert_eq!(
            convert_strings(vec![
                "CREATE",
                "TABLE",
                "users",
                "(id",
                "INTEGER",
                "PRIMARY",
                "KEY,",
                "name",
                "TEXT,",
                "age",
                "INTEGER);"
            ]),
            vec![
                "CREATE", "TABLE", "users", "(", "id", "INTEGER", "PRIMARY", "KEY", ",", "name",
                "TEXT", ",", "age", "INTEGER", ")", ";"
            ]
        )
    }
}
