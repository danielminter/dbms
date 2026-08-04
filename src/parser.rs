use std::{collections::VecDeque, io, os::raw::c_int};

use crate::tokenizer::Token;

pub fn create_ast(input: Vec<Token>) -> Result<RootNode, io::Error> {
    let syntax_error: io::Error = io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error");
    // Convert to a dequeue
    let mut token_queue = VecDeque::from(input);
    let root: RootNode = match token_queue.pop_front().unwrap() {
        Token::Keyword(t) => match t.value() {
            "CREATE" => RootNode::Create(build_create_tree(&mut token_queue).unwrap()),
            // "SELECT" => build_select_tree(&mut token_queue).unwrap(),
            // "INSERT" => build_insert_tree(&mut token_queue).unwrap(),
            _ => {
                println!("Error in CREATE");
                return Err(syntax_error);
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

fn build_create_tree(queue: &mut VecDeque<Token>) -> Result<CreateNode, io::Error> {
    let syntax_error = Err(io::Error::new(io::ErrorKind::InvalidInput, "Syntax Error"));
    // Double check that the next is TABLE then pop it off

    if queue.pop_front().unwrap().string_value() != "TABLE" {
        println!("Error in TABLE");
        return syntax_error;
    }
    // Grab the next token as the table name
    let table_token = queue.pop_front().unwrap();

    // Make sure the next token is VALUES and discard
    if queue.pop_front().unwrap().string_value() != "VALUES" {
        println!("Error in VALUES");
        return syntax_error;
    }

    // Check that an LPAREN is next then discard
    if queue.pop_front().unwrap().string_value() != "LPAREN" {
        println!("Error in LPAREN");
        return syntax_error;
    }

    let mut columns: Vec<(Token, Vec<Option<Token>>)> = vec![];

    // Loop over remaining tokens
    'columns: loop {
        // Break if we hit an RPAREN
        if queue.is_empty() || queue.front().unwrap().string_value() == "RPAREN" {
            break 'columns;
        }

        // First token is column
        let column_token = queue.pop_front().unwrap();
        // Loop until a commma or an RPAREN
        let mut conditions: Vec<Option<Token>> = vec![];
        'conditions: loop {
            if queue.is_empty() {
                println!("Error in CONDITIONS");
                return syntax_error;
            }
            if queue.front().unwrap().string_value() == "RPAREN"
                || queue.front().unwrap().string_value() == "COMMA"
            {
                break 'conditions;
            }
            // TODO: Handle compound conditions

            // Any remaining tokens are conditions
            conditions.push(Some(queue.pop_front().unwrap()));
        }

        columns.push((column_token, conditions));
    }

    // Consume the RPAREN
    queue.pop_front();

    // Check that it ends with SEMICOLON
    if queue.pop_front().unwrap().string_value() != "SEMICOLON" {
        println!("Error in SEMICOLON");
        return syntax_error;
    }

    // Construct the tree from bottom up.
    let mut column_nodes: Vec<ColumnNode> = vec![];
    for col in columns {
        let mut constraint_nodes: Vec<ConstraintNode> = vec![];
        for constraint in col.1 {
            constraint_nodes.push(ConstraintNode {
                value: constraint.unwrap().string_value().to_string(),
            });
        }
        let node = ColumnNode {
            column_name: col.0.string_value().to_string(),
            constraints: constraint_nodes,
        };

        column_nodes.push(node);
    }

    let table_node = IdentifierNode {
        identifier: table_token.string_value().to_string(),
    };

    let root: CreateNode = CreateNode {
        table: table_node,
        children: column_nodes,
    };

    // Return the tree
    Ok(root)
}

// fn build_select_tree(queue: &mut VecDeque<Token>) -> Result<Node, io::Error> {}
//
// fn build_insert_tree(queue: &mut VecDeque<Token>) -> Result<Node, io::Error> {}

trait PrintType {
    fn get_type(&self) -> &str;
}

pub enum RootNode {
    Create(CreateNode),
    Select(SelectNode),
    Insert(InsertNode),
}

impl PrintType for RootNode {
    fn get_type(&self) -> &str {
        match self {
            RootNode::Create(_) => "CREATE",
            RootNode::Select(_) => "SELECT",
            RootNode::Insert(_) => "INSERT",
        }
    }
}

pub enum Node {
    Values(ValuesNode),
    From(FromNode),
    Where(WhereNode),
    Column(ColumnNode),
    Constraint(ConstraintNode),
    Identifier(IdentifierNode),
    StringLiteral(StringLiteralNode),
    IntegerLiteral(IntegerLiteralNode),
    Condition(ConditionNode),
    Operator(OperatorNode),
}

impl PrintType for Node {
    fn get_type(&self) -> &str {
        match self {
            Node::Values(_) => "VALUES",
            Node::From(_) => "FROM",
            Node::Where(_) => "WHERE",
            Node::Column(_) => "COLUMN",
            Node::Constraint(_) => "CONSTRAINT",
            Node::Identifier(_) => "IDENTIFIER",
            Node::StringLiteral(_) => "STRINGLITERAL",
            Node::IntegerLiteral(_) => "INTEGERLITERAL",
            Node::Condition(_) => "CONDIITON",
            Node::Operator(_) => "OPERATOR",
        }
    }
}

pub struct CreateNode {
    pub table: IdentifierNode,
    pub children: Vec<ColumnNode>,
}

pub struct SelectNode {
    pub columns: Vec<IdentifierNode>,
    pub child: Box<Node>,
}

pub struct InsertNode {
    pub table: IdentifierNode,
    pub child: Box<Node>,
}

pub struct ValuesNode {
    pub values: Vec<Node>,
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
        let command: Vec<Token> = vec![
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
        ];

        let completed_tree = create_ast(command).unwrap();

        let RootNode::Create(node) = completed_tree else {
            panic!("Expected a CREATE node, got something else");
        };

        assert_eq!(node.table.identifier, "users");
        assert_eq!(node.children[0].column_name, "id");
    }

    fn test_create_syntax_checking() {
        let command: Vec<Token> = vec![
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
        ];

        let missing_create = Vec::from(command[1..]);

        let test = create_ast(missing_create).expect("Create Failed");
    }
}
