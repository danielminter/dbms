use crate::tokenizer::{SymbolType, TokenTag};

pub trait Printable {
    fn message(&self) -> String;
}

pub enum SyntaxError {
    InvalidValue(InvalidValue),
    InvalidToken(InvalidToken),
    MissingToken(MissingToken),
    UnexpectedSymbol(UnexpectedSymbol),
    GenericSyntaxError(GenericSyntaxError),
}

pub struct InvalidValue {
    expected: Vec<String>,
    encountered: String,
}

impl InvalidValue {
    pub fn new(expected: Vec<&str>, encountered: String) -> InvalidValue {
        InvalidValue {
            expected: expected.iter().map(|s| s.to_string()).collect(),
            encountered,
        }
    }
}

impl Printable for InvalidValue {
    fn message(&self) -> String {
        let mut expected: String = String::new();
        for item in &self.expected {
            expected.push_str(item);
            expected.push('/');
        }
        // Remove the last trailing /
        _ = expected.pop();
        format!(
            "Invalid token. Expected: {}, Found: {}",
            expected, self.encountered
        )
    }
}

impl Printable for SyntaxError {
    fn message(&self) -> String {
        match self {
            SyntaxError::InvalidValue(t) => t.message(),
            SyntaxError::InvalidToken(t) => t.message(),
            SyntaxError::UnexpectedSymbol(t) => t.message(),
            SyntaxError::MissingToken(t) => t.message(),
            SyntaxError::GenericSyntaxError(t) => t.message(),
        }
    }
}

pub struct InvalidToken {
    expected: TokenTag,
    encountered: TokenTag,
}

impl InvalidToken {
    pub fn new(expected: TokenTag, encountered: TokenTag) -> InvalidToken {
        InvalidToken {
            expected,
            encountered,
        }
    }
}

impl Printable for InvalidToken {
    fn message(&self) -> String {
        format!(
            "Invalid token. Expected: {}, Found: {}",
            self.expected.to_string(),
            self.encountered.to_string()
        )
    }
}

pub struct MissingToken {
    expected: Option<TokenTag>,
}

impl MissingToken {
    pub fn new(expected: Option<TokenTag>) -> MissingToken {
        MissingToken { expected }
    }
}

impl Printable for MissingToken {
    fn message(&self) -> String {
        match self.expected {
            Some(t) => format!("Missing token. Expected: {}", t.to_string()),
            None => "Missing token".to_string(),
        }
    }
}

pub struct UnexpectedSymbol {
    encountered: SymbolType,
}

impl UnexpectedSymbol {
    pub fn new(encountered: &SymbolType) -> UnexpectedSymbol {
        UnexpectedSymbol {
            encountered: encountered.clone(),
        }
    }
}

impl Printable for UnexpectedSymbol {
    fn message(&self) -> String {
        format!("Unexpected symbol: {}", self.encountered.print_type())
    }
}

pub struct GenericSyntaxError {
    msg: String,
}

impl GenericSyntaxError {
    pub fn new(msg: &str) -> GenericSyntaxError {
        GenericSyntaxError {
            msg: msg.to_string(),
        }
    }
}

impl Printable for GenericSyntaxError {
    fn message(&self) -> String {
        self.msg.to_string()
    }
}
