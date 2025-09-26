use anyhow::Result;

#[allow(dead_code)]
fn tokenize(query: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    for t in query.split_whitespace() {
        match t {
            "(" => tokens.push(Token::LeftParen),
            ")" => tokens.push(Token::RightParen),
            "&" => tokens.push(Token::And),
            "|" => tokens.push(Token::Or),
            t => tokens.push(Token::TagWord(t.to_string())),
        }
    }
    Ok(tokens)
}

// example of possilbe tokens tag:word & word | -tag:word & ( tag:word | tag:word )
// for now whitespace matters, notice the spaces even around the parens
#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
enum Token {
    TagWord(String),
    And,
    Or,
    LeftParen,
    RightParen,
}

#[cfg(test)]
mod tokenize_tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let query = "tag:word & word | -tag:word & ( tag:word | tag:word )";
        let tokens = tokenize(query).unwrap();
        let expected = vec![
            Token::TagWord("tag:word".to_string()),
            Token::And,
            Token::TagWord("word".to_string()),
            Token::Or,
            Token::TagWord("-tag:word".to_string()),
            Token::And,
            Token::LeftParen,
            Token::TagWord("tag:word".to_string()),
            Token::Or,
            Token::TagWord("tag:word".to_string()),
            Token::RightParen,
        ];
        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(t, expected.get(i).unwrap(), "bad token at {i}");
        }
    }
}
