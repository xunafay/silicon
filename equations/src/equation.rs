use crate::{
    s::{expr, ParseError, S},
    tokenize::Token,
};

#[derive(Debug, Clone)]
pub enum Equation {
    Assignment(S, S, String),
    Differential(S, S, String),
}

impl Equation {
    pub fn new(root: S) -> Self {
        let mut queue = vec![root];
        let mut unit = "unit".to_string();

        while let Some(s) = queue.pop() {
            match s {
                S::Cons(Token::Operator(token), children) => {
                    if token == '=' {
                        let left_node = children.first().unwrap();
                        let right_node = children.last().unwrap();

                        match left_node {
                            S::Cons(Token::Operator('/'), _) => {
                                return Equation::Differential(
                                    left_node.clone(),
                                    right_node.clone(),
                                    unit,
                                );
                            }
                            _ => {
                                return Equation::Assignment(
                                    left_node.clone(),
                                    right_node.clone(),
                                    unit,
                                );
                            }
                        }
                    } else if token == ':' {
                        unit = children.iter().nth(1).unwrap().to_standard_string();
                    }

                    queue.append(&mut children.clone());
                }
                _ => continue,
            }
        }

        panic!("Invalid expression");
    }

    pub fn lhs(&self) -> &S {
        match self {
            Equation::Assignment(lhs, _, _) => lhs,
            Equation::Differential(lhs, _, _) => lhs,
        }
    }

    pub fn rhs(&self) -> &S {
        match self {
            Equation::Assignment(_, rhs, _) => rhs,
            Equation::Differential(_, rhs, _) => rhs,
        }
    }

    pub fn unit(&self) -> &str {
        match self {
            Equation::Assignment(_, _, unit) => unit,
            Equation::Differential(_, _, unit) => unit,
        }
    }
}

pub fn parse_equations(input: &str) -> Result<Vec<Equation>, ParseError> {
    let mut parsed_expressions = vec![];

    let expressions = input.trim().split('\n');
    for expression in expressions {
        parsed_expressions.push(Equation::new(expr(expression)?));
    }

    Ok(parsed_expressions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expressions() {
        let input = "1 + 2 = 3\n4 * 5 = 20";
        let expressions = parse_equations(input).unwrap();
        assert_eq!(expressions.len(), 2);

        let input = "
        x = 1*mV + 2 : m
        dv/dt = 4 * 5 : volt
        I_leak = (I - 1) * 0.2 : amp
        ";
        let expressions = parse_equations(input).unwrap();
        for expression in &expressions {
            println!(
                "{} = {} : {}",
                expression.lhs().to_standard_string(),
                expression.rhs().to_standard_string(),
                expression.unit(),
            );
        }

        assert_eq!(expressions.len(), 3);

        assert_eq!(expressions[0].unit(), "m");
        assert_eq!(expressions[1].unit(), "volt");
        assert_eq!(expressions[2].unit(), "amp");

        assert_eq!(expressions[0].lhs().to_standard_string(), "x");

        assert!(match expressions[0] {
            Equation::Assignment(_, _, _) => true,
            _ => false,
        });

        assert!(match expressions[1] {
            Equation::Differential(_, _, _) => true,
            _ => false,
        });
    }

    #[test]
    fn test_parse_expressions_error() {
        let input = "1 + 2 3\n4 * 5 = 20\n";
        let result = parse_equations(input);
        assert!(result.is_err());
    }
}
