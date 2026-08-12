use crate::errors::{
    GenericSyntaxError, InvalidToken, InvalidValue, MissingToken, SyntaxError, UnexpectedSymbol,
};
use crate::tokenizer::{Literal, SymbolType, Token, TokenQueue, TokenTag};

pub fn create_ast(mut input: TokenQueue) -> Result<RootNode, SyntaxError> {
    input.print_queue();
    // Convert to a dequeue
    let root: RootNode = match input.next() {
        Ok(Token::Keyword(t)) => match t.value() {
            "CREATE" => {
                println!("Create Command Start");
                let result = build_create_tree(&mut input)?;
                RootNode::Create(result)
            }
            "SELECT" => {
                println!("Select Command Start");
                let result = build_select_tree(&mut input)?;
                RootNode::Select(result)
            }
            "INSERT" => {
                println!("Insert Command Start");
                let result = build_insert_tree(&mut input)?;
                RootNode::Insert(result)
            }
            t => {
                return Err(SyntaxError::InvalidValue(InvalidValue::new(
                    vec!["Keyword"],
                    t,
                )));
            }
        },
        Ok(t) => {
            println!("Returning Invalid token error");
            return Err(SyntaxError::InvalidToken(InvalidToken::new(
                TokenTag::Keyword,
                t.tag(),
            )));
        }
        Err(_) => {
            println!("Returning Missing token error");
            return Err(SyntaxError::MissingToken(MissingToken::new(Some(
                TokenTag::Keyword,
            ))));
        }
    };

    Ok(root)
}

fn build_insert_tree(queue: &mut TokenQueue) -> Result<InsertNode, SyntaxError> {
    // Check that the next token is "INTO"
    queue.validate_token_value(vec!["INTO"])?;
    // Get the following token as a table identifier
    let table_name = queue.validate_token_type(TokenTag::Identifier)?;
    // Verify that the next token is "VALUES"
    queue.validate_token_value(vec!["VALUES"])?;
    // Verify that the next token is "LPAREN"
    queue.verify_symbol_type(vec![SymbolType::LParen])?;
    // Loop until we find a "RPAREN"
    let mut rows: Vec<Vec<Literal>> = vec![];
    'rows: loop {
        let mut values: Vec<Literal> = vec![];
        'values: loop {
            let next = match queue.next() {
                Ok(Token::Literal(t)) => t,
                Ok(Token::Symbol(s)) => match s {
                    // followed by a "COMMA"
                    SymbolType::Comma => continue 'values,
                    // An "LPAREN" here means we're processing another row,
                    // TODO: Catch this correctly and enforce the right syntax
                    SymbolType::LParen => continue 'values,
                    SymbolType::RParen => break 'values,
                    val => {
                        return Err(SyntaxError::UnexpectedSymbol(UnexpectedSymbol::new(&val)));
                    }
                },
                _ => {
                    return Err(SyntaxError::MissingToken(MissingToken::new(Some(
                        TokenTag::Literal,
                    ))));
                }
            };
            values.push(next);
        }

        rows.push(values);

        match queue.next() {
            Ok(t) => match t {
                Token::Symbol(t) => match t {
                    // If we find a "COMMA" after the "RPAREN", loop again for another row
                    SymbolType::Comma => continue 'rows,
                    SymbolType::Semicolon => break 'rows,
                    val => {
                        return Err(SyntaxError::UnexpectedSymbol(UnexpectedSymbol::new(&val)));
                    }
                },
                val => {
                    return Err(SyntaxError::InvalidToken(InvalidToken::new(
                        TokenTag::Symbol,
                        val.tag(),
                    )));
                }
            },
            Err(_) => {
                return Err(SyntaxError::MissingToken(MissingToken::new(Some(
                    TokenTag::Symbol,
                ))));
            }
        }
    }

    let mut row_nodes: Vec<RowNode> = vec![];
    // Group the rows
    for row in rows {
        let mut literal_nodes: Vec<LiteralNode> = vec![];
        // Group the literals
        for literal in row {
            let node: LiteralNode = LiteralNode {
                value: literal.value,
                literal_type: literal.literal_type,
            };
            literal_nodes.push(node);
        }

        let row_node = RowNode {
            literals: literal_nodes,
        };

        row_nodes.push(row_node);
    }

    // Construct the table identifier node
    let table_node = IdentifierNode {
        identifier: table_name.value().to_string(),
    };

    let root_node = InsertNode {
        table: table_node,
        children: row_nodes,
    };

    // Build the ast from bottom up
    Ok(root_node)
}

fn build_select_tree(queue: &mut TokenQueue) -> Result<SelectNode, SyntaxError> {
    // Grab the first value as a column
    let mut columns: Vec<Token> = vec![];
    match queue.next() {
        Ok(val) => match val {
            Token::Identifier(t) => {
                columns.push(Token::Identifier(t));
            }
            Token::Symbol(s) => match s {
                SymbolType::Star => columns.push(Token::Symbol(s)),
                val => {
                    return Err(SyntaxError::UnexpectedSymbol(UnexpectedSymbol::new(&val)));
                }
            },
            val => {
                return Err(SyntaxError::InvalidValue(InvalidValue::new(
                    vec!["identifier"],
                    val.value(),
                )));
            }
        },
        Err(_) => {
            return Err(SyntaxError::MissingToken(MissingToken::new(Some(
                TokenTag::Identifier,
            ))));
        }
    }
    {};

    // After the first value, loop while the next token is an identifier
    loop {
        match queue.next_token_type() {
            TokenTag::Identifier => {
                columns.push(queue.next()?);
                continue;
            }
            // If its a comma, keep going, other symbols will error out
            TokenTag::Symbol => {
                if queue.check_next_symbol(vec![SymbolType::Comma]) {
                    continue;
                } else {
                    return Err(SyntaxError::InvalidValue(InvalidValue::new(
                        vec![","],
                        queue.next()?.value(),
                    )));
                }
            }
            _ => break,
        }
    }

    // Ensure the next token is "FROM"
    queue.validate_token_value(vec!["FROM"])?;
    // Grab the next token as the table identifier
    let table_name = queue.validate_token_type(TokenTag::Identifier)?;

    let mut conditions: Vec<(Token, Option<Token>, Option<Token>)> = vec![];

    // Check for an ending Semicolon
    while !queue.check_next_symbol(vec![SymbolType::Semicolon]) {
        // If there is a WHERE, Loop over values creating conditions
        // TODO: Handle conditions wrapped in parenthesis
        if queue.next_token_contains(vec!["WHERE"]) {
            queue.print_queue();
            // Consume the "WHERE"
            queue.next()?;
            queue.print_queue();
            'conditions: loop {
                // Exit condition
                if queue.check_next_symbol(vec![SymbolType::Semicolon]) || !queue.has_tokens() {
                    break 'conditions;
                }

                // Add the keyword as part of the conditions and then continue
                if queue.next_token_contains(vec!["AND", "OR"]) {
                    conditions.push((queue.next()?, None, None));

                    continue;
                }

                // Create conditions
                let left = queue.next()?;
                let op = queue.next()?;
                let right = queue.next()?;

                conditions.push((left, Some(op), Some(right)));
            }
        }
    }

    // Double check that we have an ending semicolon
    queue.verify_symbol_type(vec![SymbolType::Semicolon])?;

    // Construct the ast from bottom up

    let mut condtion_nodes: Vec<ConditionNode> = vec![];
    let mut column_nodes: Vec<IdentifierNode> = vec![];

    let mut conditions = conditions.iter().peekable();
    while let Some(current) = conditions.next() {
        match current.0.tag() {
            // If the next condition is a logical operator, add it to the previous node
            TokenTag::Identifier => {
                let left = IdentifierNode {
                    identifier: current.0.value().to_string(),
                };
                let operator = OperatorNode {
                    operator: match current.1.as_ref() {
                        Some(Token::Symbol(val)) => val.clone(),
                        _ => {
                            return Err(SyntaxError::GenericSyntaxError(GenericSyntaxError::new(
                                "Invalid condition construction",
                            )));
                        }
                    },
                };
                // TODO: Change right condition to be a literal with a generic
                let right: LiteralNode = match current.2.as_ref() {
                    Some(val) => match val {
                        Token::Literal(s) => LiteralNode {
                            value: String::from(&s.value),
                            literal_type: LiteralType::String,
                        },
                        t => {
                            return Err(SyntaxError::InvalidToken(InvalidToken::new(
                                TokenTag::Literal,
                                t.tag(),
                            )));
                        }
                    },
                    None => {
                        return Err(SyntaxError::GenericSyntaxError(GenericSyntaxError::new(
                            "Invalid condition construction",
                        )));
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
            _ => return Err(SyntaxError::GenericSyntaxError(GenericSyntaxError::new(""))),
        }
    }

    let where_node: WhereNode = WhereNode {
        conditions: condtion_nodes,
    };

    for column in columns {
        let c: IdentifierNode = match column {
            Token::Symbol(SymbolType::Star) => IdentifierNode {
                identifier: "STAR".to_string(),
            },
            Token::Identifier(i) => IdentifierNode {
                identifier: i.value,
            },
            t => {
                return Err(SyntaxError::InvalidToken(InvalidToken::new(
                    TokenTag::Identifier,
                    t.tag(),
                )));
            }
        };
        column_nodes.push(c);
    }

    let select_node: SelectNode = SelectNode {
        columns: column_nodes,
        child: FromNode {
            table: IdentifierNode {
                identifier: table_name.value().to_string(),
            },
            children: vec![Node::Where(where_node)],
        },
    };

    Ok(select_node)
}

fn build_create_tree(queue: &mut TokenQueue) -> Result<CreateNode, SyntaxError> {
    // Double check that the next is TABLE then pop it off
    queue.validate_token_value(vec!["TABLE"])?;
    // Grab the next token as the table name
    let table_token = queue.validate_token_type(TokenTag::Identifier)?;

    // Make sure the next token is VALUES and discard
    queue.validate_token_value(vec!["VALUES"])?;
    // Check that an LPAREN is next then discard
    queue.verify_symbol_type(vec![SymbolType::LParen])?;

    let mut columns: Vec<(Token, Vec<Option<Token>>)> = vec![];

    // Loop over remaining tokens
    'columns: loop {
        // Break if we hit an RPAREN
        if queue.check_next_symbol(vec![SymbolType::RParen]) {
            queue.next()?;
            break 'columns;
        }

        // First token is column
        let column_token = queue.validate_token_type(TokenTag::Identifier)?;
        // Loop until a commma or an RPAREN
        let mut constraints: Vec<Option<Token>> = vec![];
        'constraints: loop {
            if queue.check_next_symbol(vec![SymbolType::RParen, SymbolType::Comma]) {
                break 'constraints;
            }

            // TODO: Handle compound conditions

            // Any remaining tokens are constraints
            constraints.push(Some(queue.validate_token_type(TokenTag::Keyword)?));
        }

        columns.push((column_token, constraints));
    }

    // Check that it ends with SEMICOLON
    queue.verify_symbol_type(vec![SymbolType::Semicolon])?;

    // Construct the tree from bottom up.
    let mut column_nodes: Vec<ColumnNode> = vec![];
    for col in columns {
        let mut constraint_nodes: Vec<ConstraintNode> = vec![];
        for constraint in col.1 {
            constraint_nodes.push(ConstraintNode {
                value: match constraint {
                    Some(val) => val.value().to_string(),
                    None => {
                        return Err(SyntaxError::MissingToken(MissingToken::new(None)));
                    }
                },
            })
        }
        let node = ColumnNode {
            column_identifier: IdentifierNode {
                identifier: col.0.value().to_string(),
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

    println!("{}", root.table.identifier);
    println!("{}", root.children[0].column_identifier.identifier);

    // Return the tree
    Ok(root)
}

// fn build_insert_tree(queue: &mut VecDeque<Token>) -> Result<Node, io::Error> {}

#[derive(Debug)]
pub enum RootNode {
    Create(CreateNode),
    Select(SelectNode),
    Insert(InsertNode),
}

#[derive(Debug)]
pub enum Node {
    From(FromNode),
    Where(WhereNode),
    Column(ColumnNode),
    Constraint(ConstraintNode),
    Identifier(IdentifierNode),
    Condition(ConditionNode),
    Operator(OperatorNode),
    Logical(LogicalNode),
    Row(RowNode),
    LiteralNode(LiteralNode),
}

#[derive(Debug, PartialEq)]
pub enum LiteralType {
    String,
    Integer,
}

#[derive(Debug)]
pub struct CreateNode {
    pub table: IdentifierNode,
    pub children: Vec<ColumnNode>,
}

#[derive(Debug)]
pub struct SelectNode {
    pub columns: Vec<IdentifierNode>,
    pub child: FromNode,
}

#[derive(Debug)]
pub struct InsertNode {
    pub table: IdentifierNode,
    pub children: Vec<RowNode>,
}

#[derive(Debug)]
pub struct RowNode {
    pub literals: Vec<LiteralNode>,
}

#[derive(Debug)]
pub struct FromNode {
    pub table: IdentifierNode,
    pub children: Vec<Node>,
}

#[derive(Debug)]
pub struct WhereNode {
    pub conditions: Vec<ConditionNode>,
}

#[derive(Debug)]
pub struct ColumnNode {
    pub column_identifier: IdentifierNode,
    pub constraints: Option<Vec<ConstraintNode>>,
}

#[derive(Debug)]
pub struct ConstraintNode {
    pub value: String,
}

#[derive(Debug)]
pub struct IdentifierNode {
    pub identifier: String,
}

#[derive(Debug)]
pub struct LiteralNode {
    pub value: String,
    pub literal_type: LiteralType,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OperatorNode {
    pub operator: SymbolType,
}

#[derive(Debug)]
pub struct LogicalNode {
    pub logic: String,
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        errors::Printable,
        tokenizer::{Identifier, Keyword, Literal, SymbolType},
    };

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
            Token::Symbol(SymbolType::LParen),
            Token::Identifier(Identifier {
                value: "id".to_string(),
            }),
            Token::Symbol(SymbolType::RParen),
            Token::Symbol(SymbolType::Semicolon),
        ]));

        let node = match create_ast(command) {
            Ok(RootNode::Create(val)) => val,
            Ok(_) => panic!("Unexpected node type created"),
            Err(e) => panic!("{}", e.message()),
        };

        assert_eq!(node.table.identifier, "users");
        assert_eq!(node.children[0].column_identifier.identifier, "id");
    }

    #[test]
    fn test_missing_create_keyword() {
        let command: TokenQueue = TokenQueue::new(vec![Token::Keyword(Keyword {
            value: "NOT_CREATE".to_string(),
        })]);

        command.print_queue();

        let result = create_ast(command);

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_table_keyword() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "CREATE".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "NOT_TABLE".to_string(),
            }),
        ]);

        assert!(create_ast(command).is_err());
    }

    #[test]
    fn test_wrong_token_type() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "CREATE".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "TABLE".to_string(),
            }),
            Token::Symbol(SymbolType::LParen),
        ]));

        assert!(create_ast(command).is_err());
    }

    #[test]
    fn test_simple_select() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "SELECT".to_string(),
            }),
            Token::Symbol(SymbolType::Star),
            Token::Keyword(Keyword {
                value: "FROM".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "users".to_string(),
            }),
            Token::Symbol(SymbolType::Semicolon),
        ]));

        let node = match create_ast(command) {
            Ok(RootNode::Select(val)) => val,
            Ok(_) => panic!("Expected a SELECT node, got something else"),
            Err(e) => panic!("{}", e.message()),
        };

        assert_eq!(node.columns[0].identifier, "STAR");
        assert_eq!(node.child.table.identifier, "users");
    }

    #[test]
    fn test_simple_insert() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "INSERT".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "INTO".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "users".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "VALUES".to_string(),
            }),
            Token::Symbol(SymbolType::LParen),
            Token::Literal(Literal {
                value: "1".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "daniel".to_string(),
                literal_type: LiteralType::String,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "24".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::RParen),
            Token::Symbol(SymbolType::Semicolon),
        ]));

        let completed_tree = create_ast(command);

        let node = match completed_tree {
            Ok(RootNode::Insert(val)) => val,
            Ok(_) => panic!("Expected a INSERT node, got something else"),
            Err(e) => panic!("{}", e.message()),
        };

        assert_eq!(node.children[0].literals[0].value, "1");
        assert!(matches!(
            node.children[0].literals[0].literal_type,
            LiteralType::Integer
        ));
        assert_eq!(node.children[0].literals[1].value, "daniel");
        assert!(matches!(
            node.children[0].literals[1].literal_type,
            LiteralType::String
        ));
    }

    #[test]
    fn test_multi_row_insert() {
        let command: TokenQueue = TokenQueue::new(vec![
            Token::Keyword(Keyword {
                value: "INSERT".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "INTO".to_string(),
            }),
            Token::Identifier(Identifier {
                value: "users".to_string(),
            }),
            Token::Keyword(Keyword {
                value: "VALUES".to_string(),
            }),
            Token::Symbol(SymbolType::LParen),
            Token::Literal(Literal {
                value: "1".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "daniel".to_string(),
                literal_type: LiteralType::String,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "24".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::RParen),
            Token::Symbol(SymbolType::Comma),
            Token::Symbol(SymbolType::LParen),
            Token::Literal(Literal {
                value: "2".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "minter".to_string(),
                literal_type: LiteralType::String,
            }),
            Token::Symbol(SymbolType::Comma),
            Token::Literal(Literal {
                value: "42".to_string(),
                literal_type: LiteralType::Integer,
            }),
            Token::Symbol(SymbolType::RParen),
            Token::Symbol(SymbolType::Semicolon),
        ]));

        let node = match create_ast(command) {
            Ok(RootNode::Insert(val)) => val,
            Ok(_) => panic!("Expected a INSERT node, got something else"),
            Err(e) => panic!("{}", e.message()),
        };

        assert_eq!(node.children[0].literals[0].value, "1");
        assert!(matches!(
            node.children[0].literals[0].literal_type,
            LiteralType::Integer
        ));
        assert_eq!(node.children[1].literals[1].value, "minter");
        assert!(matches!(
            node.children[1].literals[1].literal_type,
            LiteralType::String
        ));
    }
}
