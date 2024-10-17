use core::fmt;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, multispace0, one_of},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Operator(char),
    Identifier(String),
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Token::Number(n) => n.to_string(),
            Token::Operator(c) => c.to_string(),
            Token::Identifier(s) => s.to_string(),
            Token::Eof => "EOF".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl Into<String> for Token {
    fn into(self) -> String {
        match self {
            Token::Number(n) => n.to_string(),
            Token::Operator(c) => c.to_string(),
            Token::Identifier(s) => s,
            Token::Eof => "EOF".to_string(),
        }
    }
}

pub(crate) struct Lexer {
    pub(crate) tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let (_, mut tokens) = Self::tokenize(input).unwrap();
        tokens.reverse();
        Lexer { tokens }
    }

    fn from_tokens(tokens: Vec<Token>) -> Self {
        Lexer { tokens }
    }

    fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
        let (input, tokens) = many0(delimited(
            multispace0,
            alt((parse_number, parse_operator, parse_identifier)),
            multispace0,
        ))(input)?;

        Ok((input, tokens))
    }

    /// Check if the lexer contains an assignment operator
    pub fn is_assignment(&self) -> bool {
        self.tokens.iter().any(|t| t == &Token::Operator('='))
    }

    /// Split the lexer into two lexers at the first `=` operator
    /// Returns the left and right hand side of the assignment
    /// The `=` operator is removed from the lexers
    pub fn split_assignment(&self) -> (Lexer, Lexer) {
        let index = self.tokens.iter().position(|t| t == &Token::Operator('='));
        let mut tokens = self.tokens.clone();
        let rhs = tokens.split_off(index.unwrap());
        tokens.pop(); // Remove the '=' operator
        (
            Lexer::from_tokens(self.tokens.clone()),
            Lexer::from_tokens(rhs),
        )
    }

    /// Take all the tokens after the first `:` operator
    pub fn take_metadata(&mut self) -> Vec<Token> {
        self.tokens.reverse();

        let index = self.tokens.iter().position(|t| t == &Token::Operator(':'));
        let metadata = self.tokens.split_off(index.unwrap_or(self.tokens.len()));

        self.tokens.reverse();
        metadata
    }

    /// Take the next token from the lexer, returns `Token::Eof` if there are no more tokens
    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    /// Peek the next token from the lexer, returns `Token::Eof` if there are no more tokens
    pub fn peek(&mut self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token::Eof)
    }
}

#[rustfmt::skip]
fn parse_number(input: &str) -> IResult<&str, Token> {
    map(
        recognize(
            tuple((
                digit1,
                opt(preceded(tag("."), digit1))
            ))
        ),
        |num_str: &str| Token::Number(num_str.parse().unwrap()),
    )(input)
}

fn parse_operator(input: &str) -> IResult<&str, Token> {
    map(one_of("+-*/^()=:"), Token::Operator)(input)
}

fn parse_identifier(input: &str) -> IResult<&str, Token> {
    map(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        |s: &str| Token::Identifier(s.to_string()),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_math() {
        let input = "1 + 2 * 3";
        let expected = vec![
            Token::Number(3.0),
            Token::Operator('*'),
            Token::Number(2.0),
            Token::Operator('+'),
            Token::Number(1.0),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);

        let input = "1 + 2 * (3 - 4)";
        let expected = vec![
            Token::Operator(')'),
            Token::Number(4.0),
            Token::Operator('-'),
            Token::Number(3.0),
            Token::Operator('('),
            Token::Operator('*'),
            Token::Number(2.0),
            Token::Operator('+'),
            Token::Number(1.0),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);

        let input = "a^2 + 4^3";
        let expected = vec![
            Token::Number(3.0),
            Token::Operator('^'),
            Token::Number(4.0),
            Token::Operator('+'),
            Token::Number(2.0),
            Token::Operator('^'),
            Token::Identifier("a".to_string()),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);
    }

    #[test]
    fn test_sub_expression() {
        let input = "I = sin(2*pi*freq*t) : amp";
        let expected = vec![
            Token::Identifier("amp".to_string()),
            Token::Operator(':'),
            Token::Operator(')'),
            Token::Identifier("t".to_string()),
            Token::Operator('*'),
            Token::Identifier("freq".to_string()),
            Token::Operator('*'),
            Token::Identifier("pi".to_string()),
            Token::Operator('*'),
            Token::Number(2.0),
            Token::Operator('('),
            Token::Identifier("sin".to_string()),
            Token::Operator('='),
            Token::Identifier("I".to_string()),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);
    }

    #[test]
    fn test_equation() {
        let input = "dv/dt = -(v + I)/ tau : volt";
        let expected = vec![
            Token::Identifier("volt".to_string()),
            Token::Operator(':'),
            Token::Identifier("tau".to_string()),
            Token::Operator('/'),
            Token::Operator(')'),
            Token::Identifier("I".to_string()),
            Token::Operator('+'),
            Token::Identifier("v".to_string()),
            Token::Operator('('),
            Token::Operator('-'),
            Token::Operator('='),
            Token::Identifier("dt".to_string()),
            Token::Operator('/'),
            Token::Identifier("dv".to_string()),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);
    }

    #[test]
    fn test_float() {
        let input = "1.0 + 2.0";
        let expected = vec![Token::Number(2.0), Token::Operator('+'), Token::Number(1.0)];
        assert_eq!(Lexer::new(input).tokens, expected);
    }

    #[test]
    fn test_unit() {
        let input = "1.0*mV : volt";
        let expected = vec![
            Token::Identifier("volt".to_string()),
            Token::Operator(':'),
            Token::Identifier("mV".to_string()),
            Token::Operator('*'),
            Token::Number(1.0),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);
    }

    #[test]
    fn test_paran() {
        let input = "((1))";
        let expected = vec![
            Token::Operator(')'),
            Token::Operator(')'),
            Token::Number(1.0),
            Token::Operator('('),
            Token::Operator('('),
        ];
        assert_eq!(Lexer::new(input).tokens, expected);
    }
}
