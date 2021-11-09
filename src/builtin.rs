use crate::{eval, is_qexp, to_num, Expression, LispError, LispErrorType};

pub fn builtin(sym: &str, operands: Vec<Expression>) -> Expression {
    match sym {
        "head" => builtin_head(operands),
        "tail" => builtin_tail(operands),
        "list" => builtin_list(operands),
        "eval" => builtin_eval(operands),
        "join" => builtin_join(operands),
        _ => builtin_op(sym, operands),
    }
}

fn builtin_op(sym: &str, operands: Vec<Expression>) -> Expression {
    // flatten down the numbers
    let results: Option<Vec<f64>> = operands.iter().map(to_num).collect();
    // kick out anything thats not a number
    let operands = match results {
        Some(operands) => operands,
        None => return Expression::Error(LispError::new(LispErrorType::BadNum)),
    };

    // handle unary functions
    if operands.len() == 1 {
        if "-" == sym {
            return Expression::Num(-operands[0]);
        } else {
            return Expression::Num(operands[0]);
        }
    }

    let mut x = operands[0];
    let mut i = 1;
    // apply the symbol over each operand
    while i < operands.len() {
        let y = operands[i];
        match sym {
            "+" => x += y,
            "-" => x -= y,
            "*" => x *= y,
            "/" => {
                if y == 0_f64 {
                    return Expression::Error(LispError::new(LispErrorType::DivZero));
                } else {
                    x /= y;
                }
            }
            _ => x += y,
        }
        i += 1;
    }

    Expression::Num(x)
}

fn builtin_head(operands: Vec<Expression>) -> Expression {
    // we want only one arguement
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    }

    let arg = &operands[0];
    match arg {
        Expression::Qexp(qexpr) => {
            if qexpr.len() == 0 {
                Expression::Error(LispError::new(LispErrorType::EmptyList))
            } else {
                qexpr[0].clone()
            }
        }
        _ => Expression::Error(LispError::new(LispErrorType::WrongType)),
    }
}

fn builtin_tail(operands: Vec<Expression>) -> Expression {
    // we want only one arguement
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    }

    let arg = &operands[0];
    // need a list/qexpr to work with
    match arg {
        Expression::Qexp(qexpr) => {
            if qexpr.len() == 0 {
                Expression::Error(LispError::new(LispErrorType::EmptyList))
            } else {
                Expression::Qexp(qexpr[1..].to_vec())
            }
        }
        _ => Expression::Error(LispError::new(LispErrorType::WrongType)),
    }
}

fn builtin_list(operands: Vec<Expression>) -> Expression {
    Expression::Qexp(operands)
}

fn builtin_eval(operands: Vec<Expression>) -> Expression {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    }

    let arg = &operands[0];
    if let Expression::Qexp(qexpr) = arg {
        eval::eval(Expression::Sexp(qexpr[..].to_vec()))
    } else {
        eval::eval(arg.clone())
    }
}

fn builtin_join(operands: Vec<Expression>) -> Expression {
    // need at least 2 arguements
    if operands.len() < 2 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    }

    // needs all arguements to be qexpr
    let results: Vec<bool> = operands
        .iter()
        .map(is_qexp)
        .filter(|b| *b == false)
        .collect();
    if results.len() > 0 {
        return Expression::Error(LispError::new(LispErrorType::WrongType));
    }

    // push each elements from each arguements into one qexpr
    let mut joined = vec![];
    for qexp in operands {
        if let Expression::Qexp(v) = qexp {
            for item in v {
                joined.push(item);
            }
        }
    }

    Expression::Qexp(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_uses_head() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(String::from("+")),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_head(vec![expr.clone()]),
            Expression::Sym(String::from("+"))
        );
        assert_eq!(
            builtin_head(vec![]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_head(vec![Expression::Sym(String::from("+"))]),
            Expression::Error(LispError::new(LispErrorType::WrongType))
        );
        assert_eq!(
            builtin_head(vec![Expression::Qexp(vec![])]),
            Expression::Error(LispError::new(LispErrorType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_tail() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(String::from("+")),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_tail(vec![expr.clone()]),
            Expression::Qexp(vec![
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_tail(vec![]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_tail(vec![Expression::Sym(String::from("+"))]),
            Expression::Error(LispError::new(LispErrorType::WrongType))
        );
        assert_eq!(
            builtin_tail(vec![Expression::Qexp(vec![])]),
            Expression::Error(LispError::new(LispErrorType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_list() {
        let expr = vec![
            Expression::Sym(String::from("+")),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ];
        assert_eq!(
            builtin_list(expr.clone()),
            Expression::Qexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_list(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
            Expression::Qexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ])
        );
        assert_eq!(builtin_list(vec![]), Expression::Qexp(vec![]));
        assert_eq!(
            builtin_list(vec![Expression::Sym(String::from("+"))]),
            Expression::Qexp(vec![Expression::Sym(String::from("+")),])
        );
        assert_eq!(
            builtin_list(vec![Expression::Sexp(vec![])]),
            Expression::Qexp(vec![Expression::Sexp(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(String::from("+")),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(builtin_eval(vec![expr.clone()]), Expression::Num(3_f64));
        assert_eq!(
            builtin_eval(vec![expr.clone(), expr.clone()]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(vec![]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(vec![Expression::Sym(String::from("+"))]),
            Expression::Sym(String::from("+"))
        );
        assert_eq!(
            builtin_eval(vec![Expression::Qexp(vec![])]),
            Expression::Sexp(vec![])
        );
    }

    #[test]
    fn it_correctly_uses_join() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(String::from("+")),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_join(vec![expr.clone(), expr.clone()]),
            Expression::Qexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
            ])
        );
        assert_eq!(
            builtin_join(vec![expr.clone()]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(vec![]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(vec![expr.clone(), Expression::Sym(String::from("+"))]),
            Expression::Error(LispError::new(LispErrorType::WrongType))
        );
        assert_eq!(
            builtin_join(vec![expr.clone(), Expression::Qexp(vec![])]),
            Expression::Qexp(vec![
                Expression::Sym(String::from("+")),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(String::from("+")),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
            ])
        );
    }
}
