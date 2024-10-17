use std::collections::HashMap;

use crate::{s::S, tokenize::Token};

pub trait ExpressionEvaluator {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Option<f64>;
}

impl ExpressionEvaluator for S {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Option<f64> {
        match self {
            S::Atom(Token::Number(n)) => Some(*n),
            S::Atom(Token::Identifier(s)) => variables.get(s).cloned(),
            S::Cons(Token::Operator('+'), children) => {
                let mut sum = 0.0;
                for child in children {
                    sum += child.evaluate(variables)?;
                }
                Some(sum)
            }
            S::Cons(Token::Operator('-'), children) => {
                let mut sum = children.first().unwrap().evaluate(variables)?;
                for child in children.iter().skip(1) {
                    sum -= child.evaluate(variables)?;
                }
                Some(sum)
            }
            S::Cons(Token::Operator('*'), children) => {
                let mut product = 1.0;
                for child in children {
                    product *= child.evaluate(variables)?;
                }
                Some(product)
            }
            S::Cons(Token::Operator('/'), children) => {
                let mut product = children.first().unwrap().evaluate(variables)?;
                for child in children.iter().skip(1) {
                    product /= child.evaluate(variables)?;
                }
                Some(product)
            }
            S::Cons(Token::Operator('^'), children) => {
                let base = children.first().unwrap().evaluate(variables)?;
                let exponent = children.last().unwrap().evaluate(variables)?;
                Some(base.powf(exponent))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::equation::parse_equations;

    use super::*;

    #[test]
    fn test_expression_evaluation() {
        let mut variables = HashMap::new();
        variables.insert("a".to_string(), 1.0);
        variables.insert("b".to_string(), 2.0);
        variables.insert("c".to_string(), 3.0);

        let expressions = parse_equations("x = (a + b) * c").unwrap();
        let equation = expressions.first().unwrap().rhs();
        let result = equation.evaluate(&variables);

        assert_eq!(
            result,
            Some((variables["a"] + variables["b"]) * variables["c"])
        );
    }

    #[test]
    fn test_expression_evaluation_complex() {
        let mut variables = HashMap::new();
        variables.insert("a".to_string(), 2.0);
        variables.insert("b".to_string(), 3.0);
        variables.insert("c".to_string(), 4.0);
        variables.insert("e".to_string(), 5.0);

        let expressions = parse_equations("x = a^b + (a * c) / e").unwrap();
        let equation = expressions.first().unwrap().rhs();
        let result = equation.evaluate(&variables);

        assert_eq!(
            result,
            Some(
                variables["a"].powf(variables["b"])
                    + (variables["a"] * variables["c"]) / variables["e"]
            )
        );
    }
}