use std::io;

pub fn parse_tokens(input: Vec<String>) -> Result<Statement, io::Error> {
    let statement: Statement = match input[0].as_str() {
        "SELECT" => Statement::Select(parse_select(input).unwrap()),
        "INSERT" => Statement::Insert(parse_insert(input).unwrap()),
        "CREATE" => Statement::Create(parse_create(input).unwrap()),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Syntax Error: Command is not recognized",
            ));
        }
    };

    Ok(statement)
}

fn parse_select(input: Vec<String>) -> Result<SelectStatement, io::Error> {
    let mut statement: SelectStatement = SelectStatement::new();

    let mut statement_section = "columns";
    let mut current_clause: Clause = Clause::new();
    let mut index: usize = 0;

    while index < input.len() {
        let token: &str = input[index].as_str();
        match statement_section {
            "columns" => {
                if token == "FROM" {
                    statement_section = "table";
                    index += 1;
                    continue;
                } else {
                    statement.add_target_column(token);
                    index += 1;
                    continue;
                }
            }
            "table" => {
                if token == "WHERE" {
                    statement_section = "clauses";
                    index += 1;
                    continue;
                } else {
                    statement.set_target_table(token);
                    index += 1;
                    continue;
                }
            }
            "clauses" => {
                while input[index] != "SEMICOLON" && index < input.len() {
                    current_clause.set_left(input[index].as_str());
                    current_clause.set_operator(input[index + 1].as_str());
                    current_clause.set_right(input[index + 2].as_str());

                    statement.add_clause(current_clause);
                    current_clause = Clause::new();

                    index += 3;
                }
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
            }
        }
    }

    Ok(statement)
}

fn parse_insert(input: Vec<String>) -> Result<InsertStatement, io::Error> {
    let mut statement: InsertStatement = InsertStatement::new();

    Ok(statement)
}

fn parse_create(input: Vec<String>) -> Result<CreateStatement, io::Error> {
    let mut statement: CreateStatement = CreateStatement::new();

    Ok(statement)
}

pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Create(CreateStatement),
}

#[derive(Debug, PartialEq)]
struct SelectStatement {
    target_table: String,
    target_columns: Vec<String>,
    clauses: Option<Vec<Clause>>,
}

impl SelectStatement {
    // Create an empty statement
    pub fn new() -> SelectStatement {
        SelectStatement {
            target_table: String::new(),
            target_columns: vec![],
            clauses: None,
        }
    }

    pub fn set_target_table(&mut self, target: &str) {
        self.target_table = String::from(target);
    }

    pub fn add_target_column(&mut self, target_column: &str) {
        self.target_columns.push(String::from(target_column));
    }

    pub fn add_clause(&mut self, clause: Clause) {
        // Initialize if its none
        if self.clauses.is_none() {
            self.clauses = Some(vec![]);
        }
        self.clauses.as_mut().unwrap().push(clause);
    }
}

struct InsertStatement();

impl InsertStatement {
    pub fn new() -> InsertStatement {
        InsertStatement {}
    }
}
struct CreateStatement();

impl CreateStatement {
    pub fn new() -> CreateStatement {
        CreateStatement {}
    }
}

#[derive(Debug, PartialEq)]
struct Clause {
    clause_type: ClauseType,
    left: String,
    operator: Option<String>,
    right: Option<String>,
}

impl Clause {
    pub fn new() -> Clause {
        Clause {
            clause_type: ClauseType::WHERE,
            left: String::new(),
            operator: None,
            right: None,
        }
    }

    pub fn set_left(&mut self, left: &str) {
        self.left = left.to_string();
    }

    pub fn set_operator(&mut self, op: &str) {
        self.operator = Some(op.to_string());
    }

    pub fn set_right(&mut self, right: &str) {
        self.right = Some(right.to_string());
    }
}

impl Default for Clause {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
enum ClauseType {
    WHERE,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
