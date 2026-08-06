#![allow(dead_code)]
use std::{
    collections::VecDeque,
    io::{self, Error, ErrorKind},
    os::raw::c_int,
};

use crate::tokenizer::{Symbol, Token, TokenQueue};
use crate::{errors::syntax_error, tokenizer::Identifier};

pub fn create_ast(mut input: TokenQueue) -> Result<RootNode, io::Error> {
    // Convert to a dequeue
    let root: RootNode = match input.peek().ok_or_else(syntax_error) {
        Ok(Token::Keyword(t)) => match t.value() {
            "CREATE" => RootNode::Create(build_create_tree(&mut input)?),
            "SELECT" => build_select_tree(&mut input)?,
            // "INSERT" => build_insert_tree(&mut token_queue)?,
            _ => {
                println!("Error in CREATE");
                return Err(syntax_error());
            }
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Syntax Error: Command is not recognized",
            ));
        }
    };

    Ok(root)
}

fn build_select_tree(queue: &mut TokenQueue) -> Result<RootNode, io::Error> {
    // Grab the first value as a column
    let mut columns: Vec<Token> = vec![];
    match queue.next().ok_or(syntax_error())? {
        Token::Identifier(t) => {
            columns.push(Token::Identifier(t));
        }
        _ => return Err(syntax_error()),
    };

    // After the first value, loop while the next token is an identifier
    loop {
        match queue.next().ok_or(syntax_error())? {
            Token::Identifier(t) => {
                columns.push(Token::Identifier(t));
                continue;
            }
            // If its a comma, keep going
            Token::Symbol(s) => {
                if s.value == "COMMA" {
                    continue;
                } else {
                    return Err(syntax_error());
                }
            }
            _ => break,
        };
    }

    // Ensure the next token is "FROM"
    queue
        .pop_and_check_value(vec!["FROM"])
        .ok_or(syntax_error());

    // Grab the next token as the table identifier
    let table_token: Token = match queue.next().ok_or(syntax_error())? {
        Token::Identifier(t) => Token::Identifier(t),
        _ => return Err(syntax_error()),
    };

    let mut constraints: Vec<(Token, Option<Token>, Option<Token>)> = vec![];

    // Check for an ending Semicolon
    while !queue.peek_and_check_value(vec!["SEMICOLON"]) {
        // If there is a WHERE, Loop over values creating constraints
        if queue.peek_and_check_value(vec!["WHERE"]) {
            'constraints: loop {
                // Exit condition
                if queue.peek_and_check_value(vec!["SEMICOLON"]) || !queue.has_tokens() {
                    break 'constraints;
                }

                // Add the keyword as part of the constraints
                if queue.peek_and_check_value(vec!["AND", "OR"]) {
                    constraints.push((queue.next().ok_or(syntax_error())?, None, None));
                }

                // Create constraints
                let left = queue.next().ok_or(syntax_error())?;
                let op = queue.next().ok_or(syntax_error())?;
                let right = queue.next().ok_or(syntax_error())?;

                constraints.push((left, Some(op), Some(right)));
            }
        }
    }

    // Double check that we have an ending semicolon
    queue
        .pop_and_check_value(vec!["SEMICOLON"])
        .ok_or(syntax_error());

    // Construct the ast from bottom up

    let constraints_nodes = vec![];
    let columns_nodes = vec![];

    for constraint in constraints {}
    Ok(_)
}

fn build_create_tree(queue: &mut TokenQueue) -> Result<CreateNode, io::Error> {
    // Double check that the next is TABLE then pop it off
    queue
        .pop_and_check_value(vec!["TABLE"])
        .ok_or(syntax_error());
    // Grab the next token as the table name
    let table_token = match queue.next().ok_or(syntax_error())? {
        Token::Identifier(t) => t,
        _ => return Err(syntax_error()),
    };

    // Make sure the next token is VALUES and discard
    queue
        .pop_and_check_value(vec!["VALUES"])
        .ok_or(syntax_error());

    // Check that an LPAREN is next then discard
    queue
        .pop_and_check_value(vec!["LPAREN"])
        .ok_or(syntax_error());

    let mut columns: Vec<(Token, Vec<Option<Token>>)> = vec![];

    // Loop over remaining tokens
    'columns: loop {
        // Break if we hit an RPAREN
        if !queue.has_tokens() || queue.peek_and_check_value(vec!["RPAREN"]) {
            break 'columns;
        }

        // First token is column
        let column_token = queue.next().ok_or(syntax_error())?;
        match column_token {
            Token::Identifier(_) => {}
            _ => return Err(syntax_error()),
        }
        // Loop until a commma or an RPAREN
        let mut constraints: Vec<Option<Token>> = vec![];
        'constraints: loop {
            if !queue.has_tokens() {
                return Err(syntax_error());
            }

            if queue.peek_and_check_value(vec!["RPAREN", "COMMA"]) {
                break 'constraints;
            }

            // TODO: Handle compound conditions

            // Any remaining tokens are constraints
            match queue.next().ok_or(syntax_error())? {
                Token::Keyword(t) => constraints.push(Some(Token::Keyword(t))),
                _ => break 'constraints,
            };
        }

        columns.push((column_token, constraints));
    }

    // Consume the RPAREN
    queue
        .pop_and_check_value(vec!["RPAREN"])
        .ok_or(syntax_error());

    // Check that it ends with SEMICOLON
    queue
        .pop_and_check_value(vec!["SEMICOLON"])
        .ok_or(syntax_error());

    // Construct the tree from bottom up.
    let mut column_nodes: Vec<ColumnNode> = vec![];
    for col in columns {
        let mut constraint_nodes: Vec<ConstraintNode> = vec![];
        for constraint in col.1 {
            constraint_nodes.push(ConstraintNode {
                value: constraint
                    .ok_or(Error::new(ErrorKind::NotFound, "Token not found"))?
                    .string_value()
                    .to_string(),
            });
        }
        let node = ColumnNode {
            column_name: col.0.string_value().to_string(),
            constraints: constraint_nodes,
        };

        column_nodes.push(node);
    }

    let table_node = IdentifierNode {
        identifier: table_token.value().to_string(),
    };

    let root: CreateNode = CreateNode {
        table: table_node,
        children: column_nodes,
    };

    // Return the tree
    Ok(root)
}

// fn build_insert_tree(queue: &mut VecDeque<Token>) -> Result<Node, io::Error> {}

pub enum RootNode {
    Create(CreateNode),
    Select(SelectNode),
    Insert(InsertNode),
}

pub enum Node {
    Values(ValuesNode),
    From(FromNode),
    Where(WhereNode),
    Column(ColumnNode),
    Constraint(ConstraintNode),
    Identifier(IdentifierNode),
    Condition(ConditionNode),
    Operator(OperatorNode),
}

pub enum LiteralNode {
    StringLiteral(StringLiteralNode),
    IntegerLiteral(IntegerLiteralNode),
}

pub struct CreateNode {
    pub table: IdentifierNode,
    pub children: Vec<ColumnNode>,
}

pub struct SelectNode {
    pub columns: Vec<IdentifierNode>,
    pub child: FromNode,
}

pub struct InsertNode {
    pub table: IdentifierNode,
    pub children: Vec<ValuesNode>,
}

pub struct ValuesNode {
    pub value: LiteralNode,
}

pub struct FromNode {
    pub table: IdentifierNode,
    pub children: Vec<Node>,
}

pub struct WhereNode {
    pub conditions: Vec<Node>,
}

pub struct ColumnNode {
    pub column_name: String,
    pub constraints: Vec<ConstraintNode>,
}

pub struct ConstraintNode {
    pub value: String,
}

pub struct IdentifierNode {
    pub identifier: String,
}

pub struct StringLiteralNode {
    pub value: String,
}

pub struct IntegerLiteralNode {
    pub value: c_int,
}

pub struct ConditionNode {
    pub left: IdentifierNode,
    pub operator: OperatorNode,
    pub right: Box<Node>,
}

pub struct OperatorNode {
    pub operator: String,
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Identifier, Keyword, Symbol};

    use super::*;

    #[test]
    fn test_simple_create() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "CREATE".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "TABLE".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "users".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "VALUES".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "LPAREN".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "id".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "RPAREN".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "SEMICOLON".to_string(),
            }),
        ]);

        let completed_tree = create_ast(command);

        let RootNode::Create(node) = completed_tree.unwrap() else {
            panic!("Expected a CREATE node, got something else");
        };

        assert_eq!(node.table.identifier, "users");
        assert_eq!(node.children[0].column_name, "id");
    }

    #[test]
    #[should_panic]
    fn test_missing_create_keyword() {
        let command: TokenQueue = TokenQueue::new(vec![Token::Keyword(Keyword {
            value: "NOT_CREATE".to_string(),
        })]);

        create_ast(command).expect("Create Failed");
    }

    #[test]
    #[should_panic]
    fn test_missing_table_keyword() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "CREATE".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "NOT_TABLE".to_string(),
            }),
        ]);

        create_ast(command).expect("Create Failed");
    }

    #[test]
    #[should_panic]
    fn test_wrong_token_type() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "CREATE".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "NOT_TABLE".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "LPAREN".to_string(),
            }),
        ]);

        create_ast(command).expect("Create Failed");
    }
}
