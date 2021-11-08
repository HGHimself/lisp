use crate::{is_qexp, to_num, to_sym, Expression, LispError, LispErrorType, Symbol};

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
            return builtin(sym, operands);
        } else {
            // we needed an operator for the first element to calculate
            return Expression::Error(LispError::new(LispErrorType::BadOp));
        }
    }
}

fn builtin(sym: Symbol, operands: Vec<Expression>) -> Expression {
    match sym {
        Symbol::Head => builtin_head(operands),
        Symbol::Tail => builtin_tail(operands),
        Symbol::List => builtin_list(operands),
        Symbol::Eval => builtin_eval(operands),
        Symbol::Join => builtin_join(operands),
        _ => builtin_op(sym, operands),
    }
}

fn builtin_op(sym: Symbol, operands: Vec<Expression>) -> Expression {
    // flatten down the numbers
    let results: Option<Vec<f64>> = operands.iter().map(to_num).collect();
    // kick out anything thats not a number
    let operands = match results {
        Some(operands) => operands,
        None => return Expression::Error(LispError::new(LispErrorType::BadNum)),
    };

    // handle unary functions
    if operands.len() == 1 {
        if let Symbol::Sub = sym {
            return Expression::Num(-operands[0]);
        } else {
            return Expression::Num(operands[0]);
        }
    } else {
        let mut x = operands[0];
        let mut i = 1;
        // apply the symbol over each operand
        while i < operands.len() {
            let y = operands[i];
            match sym {
                Symbol::Add => x += y,
                Symbol::Sub => x -= y,
                Symbol::Mul => x *= y,
                Symbol::Div => {
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
}

fn builtin_head(operands: Vec<Expression>) -> Expression {
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    } else {
        let arg = &operands[0];
        match arg {
            Expression::Qexp(qexpr) => {
                if qexpr.len() == 0 {
                    return Expression::Error(LispError::new(LispErrorType::EmptyList));
                } else {
                    return qexpr[0].clone();
                }
            }
            _ => return Expression::Error(LispError::new(LispErrorType::WrongType)),
        }
    }
}

fn builtin_tail(operands: Vec<Expression>) -> Expression {
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    } else {
        let arg = &operands[0];
        match arg {
            Expression::Qexp(qexpr) => {
                if qexpr.len() == 0 {
                    return Expression::Error(LispError::new(LispErrorType::EmptyList));
                } else {
                    return Expression::Qexp(qexpr[1..].to_vec());
                }
            }
            _ => return Expression::Error(LispError::new(LispErrorType::WrongType)),
        }
    }
}

fn builtin_list(operands: Vec<Expression>) -> Expression {
    Expression::Qexp(operands)
}

fn builtin_eval(operands: Vec<Expression>) -> Expression {
    if operands.len() != 1 {
        return Expression::Error(LispError::new(LispErrorType::IncorrectParamCount));
    } else {
        let arg = &operands[0];
        if let Expression::Qexp(qexpr) = arg {
            return eval(Expression::Sexp(qexpr[..].to_vec()));
        } else {
            return eval(arg.clone());
        }
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

    return Expression::Qexp(joined);
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
            eval(Expression::Sym(Symbol::Mul)),
            Expression::Sym(Symbol::Mul)
        );
        assert_eq!(
            eval(Expression::Sexp(vec![Expression::Sym(Symbol::Mul)])),
            Expression::Sym(Symbol::Mul)
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
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ])),
            Expression::Num(2_f64)
        );
        assert_eq!(
            eval(Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Sym(Symbol::Add),
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
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
            ])),
            Expression::Num(3_f64)
        );
    }

    #[test]
    fn it_correctly_uses_head() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(Symbol::Add),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_head(vec![expr.clone()]),
            Expression::Sym(Symbol::Add)
        );
        assert_eq!(
            builtin_head(vec![]),
            Expression::Error(LispError::new(LispErrorType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_head(vec![Expression::Sym(Symbol::Add)]),
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
            Expression::Sym(Symbol::Add),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_tail(vec![expr.clone()]),
            Expression::Qexp(vec![
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
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
            builtin_tail(vec![Expression::Sym(Symbol::Add)]),
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
            Expression::Sym(Symbol::Add),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ];
        assert_eq!(
            builtin_list(expr.clone()),
            Expression::Qexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_list(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
            Expression::Qexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ])
        );
        assert_eq!(builtin_list(vec![]), Expression::Qexp(vec![]));
        assert_eq!(
            builtin_list(vec![Expression::Sym(Symbol::Add)]),
            Expression::Qexp(vec![Expression::Sym(Symbol::Add),])
        );
        assert_eq!(
            builtin_list(vec![Expression::Sexp(vec![])]),
            Expression::Qexp(vec![Expression::Sexp(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(Symbol::Add),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
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
            builtin_eval(vec![Expression::Sym(Symbol::Add)]),
            Expression::Sym(Symbol::Add)
        );
        assert_eq!(
            builtin_eval(vec![Expression::Qexp(vec![])]),
            Expression::Sexp(vec![])
        );
    }

    #[test]
    fn it_correctly_uses_join() {
        let expr = Expression::Qexp(vec![
            Expression::Sym(Symbol::Add),
            Expression::Num(1_f64),
            Expression::Sexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_join(vec![expr.clone(), expr.clone()]),
            Expression::Qexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
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
            builtin_join(vec![expr.clone(), Expression::Sym(Symbol::Add)]),
            Expression::Error(LispError::new(LispErrorType::WrongType))
        );
        assert_eq!(
            builtin_join(vec![expr.clone(), Expression::Qexp(vec![])]),
            Expression::Qexp(vec![
                Expression::Sym(Symbol::Add),
                Expression::Num(1_f64),
                Expression::Sexp(vec![
                    Expression::Sym(Symbol::Add),
                    Expression::Num(1_f64),
                    Expression::Num(1_f64),
                ]),
            ])
        );
    }
}
