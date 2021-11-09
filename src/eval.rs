use crate::{builtin, to_sym, Expression, LispError, LispErrorType};

pub fn eval(expr: Expression) -> Expression {
    match expr {
        Expression::Sym(_) => expr,
        Expression::Num(_) => expr,
        Expression::Error(_) => expr,
        Expression::Qexp(_) => expr,
        Expression::Sexp(vec) => eval_sexpression(vec),
    }
}

fn eval_sexpression(sexpr: Vec<Expression>) -> Expression {
    // evaluate each element
    let results: Result<Vec<Expression>, LispError> =
        sexpr.into_iter().map(|expr| eval(expr)).collect();
    // surface any errors
    let sexpr = match results {
        Ok(sexpr) => sexpr,
        Err(e) => return Expression::Error(e),
    };

    if sexpr.len() == 0 {
        // if empty return empty
        return Expression::Sexp(sexpr);
    } else if sexpr.len() == 1 {
        // if singular value return singular value
        return sexpr[0].clone();
    } else {
        // else let's try to calculate
        if let Some(sym) = to_sym(&sexpr[0]) {
            // first element needs to be an operator
            let operands = (&sexpr[1..]).to_vec();
            // split off rest of operands and calculate
            return builtin::builtin(&sym, operands);
        } else {
            // we needed an operator for the first element to calculate
            return Expression::Error(LispError::new(LispErrorType::BadOp));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_handles_singular_numbers() {
        assert_eq!(eval(Expression::Num(1_f64)), Expression::Num(1_f64));
        assert_eq!(
            eval(Expression::Sexp(vec![Expression::Num(1_f64)])),
            Expression::Num(1_f64)
        );
    }

    #[test]
    fn it_handles_singular_symbols() {
        assert_eq!(
            eval(Expression::Sym(String::from("*"))),
            Expression::Sym(String::from("*"))
        );
        assert_eq!(
            eval(Expression::Sexp(vec![Expression::Sym(String::from("*"))])),
            Expression::Sym(String::from("*"))
        );
    }

    #[test]
    fn it_handles_singular_errors() {
        let error = Expression::Error(LispError::new(LispErrorType::DivZero));
        assert_eq!(eval(error.clone()), error);
    }

    #[test]
    fn it_handles_empty_expressions() {
        assert_eq!(eval(Expression::Sexp(vec![])), Expression::Sexp(vec![]));
        assert_eq!(
            eval(Expression::Sexp(vec![Expression::Sexp(vec![
                Expression::Sexp(vec![])
            ])])),
            Expression::Sexp(vec![])
        );
    }

    #[test]
    fn it_uses_operators_properly() {
        assert_eq!(
            eval(Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ])),
            Expression::Num(2_f64)
        );
        assert_eq!(
            eval(Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
            ])),
            Expression::Error(LispError::new(LispErrorType::BadNum))
        );
        assert_eq!(
            eval(Expression::Sexp(vec![
                Expression::Num(1_f64),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ])),
            Expression::Error(LispError::new(LispErrorType::BadOp))
        );
    }

    #[test]
    fn it_handles_nested_sexpressions() {
        assert_eq!(
            eval(Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
            ])),
            Expression::Num(3_f64)
        );
    }
}
