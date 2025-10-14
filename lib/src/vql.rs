use std::mem::discriminant;

struct Lexer {
    query: String,
    pos: usize,
    current_char: Option<char>,
}

#[allow(dead_code)]
impl Lexer {
    fn new(query: &str) -> Self {
        let query = query.to_string();
        Self {
            query: query.clone(),
            pos: 0,
            current_char: query.chars().nth(0),
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.current_char = self.query.chars().nth(self.pos);
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            match c.is_whitespace() {
                true => self.advance(),
                false => break,
            }
        }
    }

    fn selector(&mut self) -> Token {
        let op = match self.current_char {
            Some(c) => match c {
                '~' => {
                    self.advance();
                    Op::Similar
                }
                '-' => {
                    self.advance();
                    Op::Exclude
                }
                _ => Op::Include,
            },
            None => Op::Include,
        };
        let mut field_query = String::new();
        let mut quote_started = false;
        while let Some(c) = self.current_char {
            if c == '\'' && !quote_started {
                quote_started = true;
                self.advance();
                while let Some(c) = self.current_char {
                    if c != '\'' {
                        field_query.push(c);
                        self.advance();
                        continue;
                    } else {
                        quote_started = false;
                        break;
                    }
                }
                continue;
            }
            match c.is_alphanumeric() || c == ':' {
                true => {
                    field_query.push(c);
                    self.advance();
                }
                false => break,
            }
        }

        let parts = field_query.split_once(":");
        match parts {
            Some((field, query)) => Token::Selector(op, Some(field.to_string()), query.to_string()),
            _ => Token::Selector(op, None, field_query),
        }
    }

    fn get_next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.current_char {
            Some(c) => match c {
                'a'..='z' | '~' | '-' | '\'' => self.selector(),
                '&' => {
                    self.advance();
                    Token::And
                }
                '|' => {
                    self.advance();
                    Token::Or
                }
                '(' => {
                    self.advance();
                    Token::LeftParen
                }
                ')' => {
                    self.advance();
                    Token::RightParen
                }
                _ => panic!("Unexpected token {c}"),
            },
            None => Token::Eof,
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.get_next_token();
            match token {
                Token::Eof => break,
                _ => tokens.push(token),
            }
        }
        tokens
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum Op {
    Include,
    Exclude,
    Similar,
}

type Field = String;
type Query = String;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum Token {
    Selector(Op, Option<Field>, Query),
    And,
    Or,
    LeftParen,
    RightParen,
    Eof,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Selector(Token),
    BinaryOp(Box<Node>, Token, Box<Node>),
}

#[allow(dead_code)]
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn get_current_token(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens.get(self.pos).unwrap().clone()
        } else {
            Token::Eof
        }
    }

    fn eat(&mut self, token: Token) {
        if discriminant(&self.get_current_token()) == discriminant(&token) {
            self.pos += 1;
        } else {
            panic!(
                "Unexpected token type {:#?}: got {:#?}",
                token,
                self.get_current_token()
            );
        }
    }

    fn factor(&mut self) -> Node {
        let token = self.get_current_token();
        match token {
            Token::Selector(..) => {
                self.eat(token.clone());
                Node::Selector(token)
            }
            Token::LeftParen => {
                self.eat(token);
                let node = self.expression();
                self.eat(Token::RightParen);
                node
            }
            _ => panic!("Unexpected token in factor"),
        }
    }

    fn expression(&mut self) -> Node {
        let mut node = self.factor();
        while let Token::And | Token::Or = self.get_current_token() {
            let op = self.get_current_token();
            self.eat(op.clone());
            node = Node::BinaryOp(Box::new(node), op, Box::new(self.factor()));
        }
        node
    }

    fn parse(&mut self) -> Node {
        self.expression()
    }
}

pub(crate) struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, query: &str) -> Node {
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let query = "title:cats & cats | -body:dog & ( title:dog | ~body:hound )";
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::Selector(Op::Include, Some("title".to_string()), "cats".to_string()),
            Token::And,
            Token::Selector(Op::Include, None, "cats".to_string()),
            Token::Or,
            Token::Selector(Op::Exclude, Some("body".to_string()), "dog".to_string()),
            Token::And,
            Token::LeftParen,
            Token::Selector(Op::Include, Some("title".to_string()), "dog".to_string()),
            Token::Or,
            Token::Selector(Op::Similar, Some("body".to_string()), "hound".to_string()),
            Token::RightParen,
        ];
        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(t, expected.get(i).unwrap(), "bad token at {i}");
        }
    }

    #[test]
    fn test_tokenize_with_quotes_1() {
        let query = "title:'cats and dogs'";
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize();
        let expected = [Token::Selector(
            Op::Include,
            Some("title".to_string()),
            "cats and dogs".to_string(),
        )];
        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(t, expected.get(i).unwrap(), "bad token at {i}");
        }
    }

    #[test]
    fn test_tokenize_with_quotes_2() {
        let query = "'title for things':'cats and dogs'";
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize();
        let expected = [Token::Selector(
            Op::Include,
            Some("title for things".to_string()),
            "cats and dogs".to_string(),
        )];
        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(t, expected.get(i).unwrap(), "bad token at {i}");
        }
    }

    #[test]
    #[should_panic]
    fn test_bad_tokens() {
        let query = "title:cats & cats ? -body:dog";
        let mut lexer = Lexer::new(query);
        lexer.tokenize();
    }

    #[test]
    fn test_whitespace_ignored() {
        let query = "title:cats & cats          | -body:dog";
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::Selector(Op::Include, Some("title".to_string()), "cats".to_string()),
            Token::And,
            Token::Selector(Op::Include, None, "cats".to_string()),
            Token::Or,
            Token::Selector(Op::Exclude, Some("body".to_string()), "dog".to_string()),
        ];
        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(t, expected.get(i).unwrap(), "bad token at {i}");
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![
            Token::Selector(Op::Include, Some("title".to_string()), "cats".to_string()),
            Token::And,
            Token::Selector(Op::Include, None, "cats".to_string()),
            Token::Or,
            Token::Selector(Op::Exclude, Some("body".to_string()), "dog".to_string()),
        ];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let expected = Node::BinaryOp(
            Box::new(Node::BinaryOp(
                Box::new(Node::Selector(Token::Selector(
                    Op::Include,
                    Some("title".into()),
                    "cats".into(),
                ))),
                Token::And,
                Box::new(Node::Selector(Token::Selector(
                    Op::Include,
                    None,
                    "cats".into(),
                ))),
            )),
            Token::Or,
            Box::new(Node::Selector(Token::Selector(
                Op::Exclude,
                Some("body".into()),
                "dog".into(),
            ))),
        );
        assert_eq!(ast, expected);
    }
}
