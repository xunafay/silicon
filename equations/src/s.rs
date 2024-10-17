use std::fmt;

use crate::tokenize::{Lexer, Token};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
}

#[derive(Debug, Clone)]
pub enum S {
    Atom(Token),
    Cons(Token, Vec<S>),
}

impl S {
    /// convert S to a standard string representation instead of a lisp-like representation.
    /// This does not include parentheses!
    pub fn to_standard_string(&self) -> String {
        match self {
            S::Atom(t) => t.to_string(),
            S::Cons(t, rest) => {
                format!(
                    "{} {} {}",
                    rest.first()
                        .unwrap_or(&S::Atom(Token::Operator(' ')))
                        .to_standard_string(),
                    t,
                    rest.iter()
                        .skip(1)
                        .map(|s| s.to_standard_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
    }
}

impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            S::Atom(i) => write!(f, "{}", i),
            S::Cons(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

pub fn expr(input: &str) -> Result<S, ParseError> {
    let mut lexer = Lexer::new(input);
    expr_bp(&mut lexer, 0)
}

pub(crate) fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> Result<S, ParseError> {
    let mut lhs = match lexer.next() {
        Token::Number(n) => S::Atom(Token::Number(n)),
        Token::Identifier(s) => S::Atom(Token::Identifier(s)),
        Token::Operator('(') => {
            let lhs = expr_bp(lexer, 0)?;
            assert_eq!(lexer.next(), Token::Operator(')'));
            lhs
        }
        Token::Operator(op) => {
            let ((), r_bp) = prefix_binding_power(op);
            let rhs = expr_bp(lexer, r_bp)?;
            S::Cons(Token::Operator(op), vec![rhs])
        }
        t => return Err(ParseError::UnexpectedToken(t)),
    };

    loop {
        let op = match lexer.peek() {
            Token::Eof => break,
            Token::Operator(op) => op,
            t => return Err(ParseError::UnexpectedToken(t)),
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }

            lexer.next();

            lhs = S::Cons(Token::Operator(op), vec![lhs]);
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }

            lexer.next();
            let rhs = expr_bp(lexer, r_bp)?;

            lhs = S::Cons(Token::Operator(op), vec![lhs, rhs]);
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn prefix_binding_power(op: char) -> ((), u8) {
    match op {
        '+' | '-' => ((), 5),
        _ => panic!("bad op: {:?}", op),
    }
}

fn infix_binding_power(op: char) -> Option<(u8, u8)> {
    let res = match op {
        '=' | ':' => (0, 1),
        '+' | '-' => (1, 2),
        '*' | '/' => (3, 4),
        '^' => (5, 6),
        _ => return None,
    };
    Some(res)
}

fn postfix_binding_power(op: char) -> Option<(u8, ())> {
    let res = match op {
        '!' => (7, ()),
        _ => return None,
    };
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit() {
        let s = expr("1").unwrap();
        assert_eq!(s.to_string(), "1");

        let s = expr("-1").unwrap();
        assert_eq!(s.to_string(), "(- 1)");

        let s = expr("--1").unwrap();
        assert_eq!(s.to_string(), "(- (- 1))");

        let s = expr("((0))").unwrap();
        assert_eq!(s.to_string(), "0");
    }

    #[test]
    fn test_expr() {
        let input = "1 + 2";
        let output = expr(input).unwrap();
        assert_eq!(format!("{}", output), "(+ 1 2)");
    }

    #[test]
    fn test_expr_with_ident() {
        let input = "I_leak * I";
        let output = expr(input).unwrap();
        assert_eq!(format!("{}", output), "(* I_leak I)");
    }

    #[test]
    fn test_exponentiation() {
        let input = "a^2 + 4^3";
        let output = expr(input).unwrap();
        assert_eq!(format!("{}", output), "(+ (^ a 2) (^ 4 3))");
    }

    #[test]
    fn test_parentheses() {
        let input = "1 + 2 * (3 - 4)";
        let output = expr(input).unwrap();
        assert_eq!(format!("{}", output), "(+ 1 (* 2 (- 3 4)))");

        let input = "(1 + 2) * 3";
        let output = expr(input).unwrap();
        assert_eq!(format!("{}", output), "(* (+ 1 2) 3)");
    }

    #[test]
    fn test_equation() {
        let input = "dv/dt = -(v + I)/ tau : volt";
        let output = expr(input).unwrap();
        assert_eq!(
            format!("{}", output),
            "(: (= (/ dv dt) (/ (- (+ v I)) tau)) volt)"
        );
    }
}
