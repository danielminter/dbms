#![allow(dead_code)]
use std::{
    io::{self, Error, ErrorKind},
    iter::Successors,
    os::raw::c_int,
};

use crate::errors::{SyntaxError, syntax_error};
use crate::tokenizer::{Token, TokenQueue};

pub fn create_ast(mut input: TokenQueue) -> Result<RootNode, SyntaxError> {
    // Convert to a dequeue
    let root: RootNode = match input
        .peek()
        .ok_or_else(|| SyntaxError::new("No token found"))
    {
        Ok(Token::Keyword(t)) => match t.value() {
            "CREATE" => {
                println!("Statement Value: {}", t.value);
                let result = match build_create_tree(&mut input) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(e);
                    }
                };
                RootNode::Create(result)
            }
            "SELECT" => {
                println!("Statement Value: {}", t.value);
                let result = match build_select_tree(&mut input) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(e);
                    }
                };
                RootNode::Select(result)
            }
            // "INSERT" => build_insert_tree(&mut token_queue)?,
            _ => {
                println!("Statement Value: {}", t.value);
                return Err(SyntaxError::new("Invalid token at statement start"));
            }
        },
        _ => {
            return Err(SyntaxError::new("Syntax Error: Command is not recognized"));
        }
    };

    Ok(root)
}

fn build_select_tree(queue: &mut TokenQueue) -> Result<SelectNode, SyntaxError> {
    // Grab the first value as a column
    let mut columns: Vec<Token> = vec![];
    println!("Grabbing columns");
    match queue.next() {
        Some(val) => match val {
            Token::Identifier(t) => {
                columns.push(Token::Identifier(t));
            }
            Token::Symbol(s) => match s.value.as_str() {
                "STAR" => columns.push(Token::Symbol(s)),
                _ => return Err(SyntaxError::new("Invalid symbol passed as column name")),
            },
            _ => {
                return Err(SyntaxError::new("Invalid token, expected identifier"));
            }
        },
        None => {
            return Err(SyntaxError::new("Invalid column identifier"));
        }
    }
    {};

    // After the first value, loop while the next token is an identifier
    loop {
        match queue.next().ok_or(SyntaxError::new("Incomplete statement")) {
            Ok(Token::Identifier(t)) => {
                columns.push(Token::Identifier(t));
                continue;
            }
            // If its a comma, keep going
            Ok(Token::Symbol(s)) => {
                if s.value == "COMMA" {
                    continue;
                } else {
                    return Err(SyntaxError::new("Invalid Symbol"));
                }
            }
            _ => break,
        };
    }

    // Ensure the next token is "FROM"
    let _ = queue
        .pop_and_check_value(vec!["FROM"])
        .ok_or(syntax_error(Some("Invalid token, expected 'FROM'")));

    // Grab the next token as the table identifier
    let table_token: Token = match queue.next().ok_or(SyntaxError::new("Incomplete Statement")) {
        Ok(Token::Identifier(t)) => Token::Identifier(t),
        _ => return Err(SyntaxError::new("Invalid token")),
    };

    let mut conditions: Vec<(Token, Option<Token>, Option<Token>)> = vec![];

    // Check for an ending Semicolon
    while !queue.peek_and_check_value(vec!["SEMICOLON"]) {
        // If there is a WHERE, Loop over values creating conditions
        // TODO: Handle conditions wrapped in parenthesis
        if queue.peek_and_check_value(vec!["WHERE"]) {
            'conditions: loop {
                // Exit condition
                if queue.peek_and_check_value(vec!["SEMICOLON"]) || !queue.has_tokens() {
                    break 'conditions;
                }

                // Add the keyword as part of the conditions and then continue
                if queue.peek_and_check_value(vec!["AND", "OR"]) {
                    conditions.push((
                        match queue.next() {
                            Some(val) => val,
                            None => {
                                return Err(SyntaxError::new("Missing token"));
                            }
                        },
                        None,
                        None,
                    ));

                    continue;
                }

                // Create conditions
                let left = match queue.next() {
                    Some(val) => val,
                    None => {
                        return Err(SyntaxError::new("Missing token"));
                    }
                };
                let op = match queue.next() {
                    Some(val) => Some(val),
                    None => {
                        return Err(SyntaxError::new("Missing token"));
                    }
                };
                let right = match queue.next() {
                    Some(val) => Some(val),
                    None => {
                        return Err(SyntaxError::new("Missing token"));
                    }
                };

                conditions.push((left, op, right));
            }
        }
    }

    // Double check that we have an ending semicolon
    let _ = queue
        .pop_and_check_value(vec!["SEMICOLON"])
        .ok_or(syntax_error(Some("Missing end semicolon")));

    // Construct the ast from bottom up

    let mut condtion_nodes: Vec<ConditionNode> = vec![];
    let mut column_nodes: Vec<IdentifierNode> = vec![];

    let mut conditions = conditions.iter().peekable();
    while let Some(current) = conditions.next() {
        match current.0 {
            // If the next condition is a logical operator, add it to the previous node
            Token::Identifier(_) => {
                let left = IdentifierNode {
                    identifier: current.0.string_value().to_string(),
                };
                let operator = OperatorNode {
                    operator: match current.1.as_ref() {
                        Some(val) => val,
                        None => {
                            return Err(SyntaxError::new("Invalid condition construction"));
                        }
                    }
                    .string_value()
                    .to_string(),
                };
                // TODO: Change right condition to be a literal with a generic
                let right: LiteralNode = match current.2.as_ref() {
                    Some(val) => match val {
                        Token::StringLiteral(s) => LiteralNode::StringLiteral(StringLiteralNode {
                            value: String::from(&s.value),
                        }),
                        Token::IntLiteral(i) => {
                            LiteralNode::IntegerLiteral(IntegerLiteralNode { value: i.value })
                        }
                        _ => return Err(SyntaxError::new("Invalid token type: Expected Literal")),
                    },
                    None => {
                        return Err(SyntaxError::new("Invalid condition construction"));
                    }
                };

                let mut node = ConditionNode {
                    left,
                    operator,
                    right: Box::new(right),
                    logic_with_next: None,
                };

                // If the next token is a keyword, add it as a logical operator
                // TODO: This feels really brittle. Fix with something more robust
                if conditions.peek().is_some()
                    && let Token::Keyword(k) = &conditions.peek().unwrap().0
                {
                    node.set_logic(k.value.as_str());
                }

                condtion_nodes.push(node);
            }
            _ => return Err(SyntaxError::new("")),
        }
    }

    let where_node: WhereNode = WhereNode {
        conditions: condtion_nodes,
    };

    for column in columns {
        let c: IdentifierNode = match column {
            Token::Symbol(s) => IdentifierNode {
                identifier: s.value,
            },
            Token::Identifier(i) => IdentifierNode {
                identifier: i.value,
            },
            _ => return Err(SyntaxError::new("Invalid column identifier")),
        };
        column_nodes.push(c);
    }

    let select_node: SelectNode = SelectNode {
        columns: column_nodes,
        child: FromNode {
            table: IdentifierNode {
                identifier: table_token.string_value().to_string(),
            },
            children: vec![Node::Where(where_node)],
        },
    };

    Ok(select_node)
}

fn build_create_tree(queue: &mut TokenQueue) -> Result<CreateNode, SyntaxError> {
    // Double check that the next is TABLE then pop it off
    let _ = queue
        .pop_and_check_value(vec!["TABLE"])
        .ok_or(syntax_error(Some("Invalid token, Expected 'TABLE'")));
    // Grab the next token as the table name
    let table_token = match queue.next() {
        Some(val) => match val {
            Token::Identifier(t) => t,
            _ => return Err(SyntaxError::new("Invalid token, expected identifier")),
        },
        None => {
            return Err(SyntaxError::new("Missing token"));
        }
    };

    // Make sure the next token is VALUES and discard
    let _ = queue
        .pop_and_check_value(vec!["VALUES"])
        .ok_or(syntax_error(Some("Invalid token, Expected 'VALUES'")));

    // Check that an LPAREN is next then discard
    let _ = queue
        .pop_and_check_value(vec!["LPAREN"])
        .ok_or(syntax_error(Some("Invalid token, Expected '('")));

    let mut columns: Vec<(Token, Vec<Option<Token>>)> = vec![];

    // Loop over remaining tokens
    'columns: loop {
        // Break if we hit an RPAREN
        if !queue.has_tokens() || queue.peek_and_check_value(vec!["RPAREN"]) {
            break 'columns;
        }

        // First token is column
        let column_token = match queue.next() {
            Some(val) => match val {
                Token::Identifier(i) => Token::Identifier(i),
                _ => return Err(SyntaxError::new("Invalid token, expected identifier")),
            },
            None => {
                return Err(SyntaxError::new("Missing token"));
            }
        };

        // Loop until a commma or an RPAREN
        let mut constraints: Vec<Option<Token>> = vec![];
        'constraints: loop {
            if !queue.has_tokens() {
                return Err(SyntaxError::new("Missing token"));
            }

            if queue.peek_and_check_value(vec!["RPAREN", "COMMA"]) {
                break 'constraints;
            }

            // TODO: Handle compound conditions

            // Any remaining tokens are constraints
            match queue.next() {
                Some(val) => match val {
                    Token::Keyword(t) => constraints.push(Some(Token::Keyword(t))),
                    _ => break 'constraints,
                },
                None => {
                    return Err(SyntaxError::new("Missing token"));
                }
            }
        }

        columns.push((column_token, constraints));
    }

    // Consume the RPAREN
    let _ = queue
        .pop_and_check_value(vec!["RPAREN"])
        .ok_or(syntax_error(Some("Invalid token, expected ')'")));

    // Check that it ends with SEMICOLON
    let _ = queue
        .pop_and_check_value(vec!["SEMICOLON"])
        .ok_or(syntax_error(Some("Invalid token, expected ';'")));

    // Construct the tree from bottom up.
    let mut column_nodes: Vec<ColumnNode> = vec![];
    for col in columns {
        let mut constraint_nodes: Vec<ConstraintNode> = vec![];
        for constraint in col.1 {
            constraint_nodes.push(ConstraintNode {
                value: match constraint {
                    Some(val) => val.string_value().to_string(),
                    None => {
                        return Err(SyntaxError::new("Token not found"));
                    }
                },
            })
        }
        let node = ColumnNode {
            column_identifier: IdentifierNode {
                identifier: col.0.string_value().to_string(),
            },
            constraints: None,
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
    Logical(LogicalNode),
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
    pub conditions: Vec<ConditionNode>,
}

pub struct ColumnNode {
    pub column_identifier: IdentifierNode,
    pub constraints: Option<Vec<ConstraintNode>>,
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
    pub right: Box<LiteralNode>,
    pub logic_with_next: Option<LogicalNode>,
}

impl ConditionNode {
    pub fn set_logic(&mut self, logic: &str) {
        self.logic_with_next = Some(LogicalNode {
            logic: String::from(logic),
        });
    }
}

pub struct OperatorNode {
    pub operator: String,
}

pub struct LogicalNode {
    pub logic: String,
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

        let RootNode::Create(node) = create_ast(command).unwrap() else {
            panic!("Expected a SELECT node, got something else");
        };

        assert_eq!(node.table.identifier, "users");
        assert_eq!(node.children[0].column_identifier.identifier, "id");
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

    #[test]
    fn test_simple_select() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "SELECT".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "STAR".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "FROM".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "users".to_string(),
            }),
            Token::Symbol(Symbol {
                value: "SEMICOLON".to_string(),
            }),
        ]);

        let completed_tree = create_ast(command);

        let RootNode::Select(node) = completed_tree.unwrap() else {
            panic!("Expected a SELECT node, got something else");
        };

        assert_eq!(node.columns[0].identifier, "STAR");
        assert_eq!(node.child.table.identifier, "users");
    }
}
