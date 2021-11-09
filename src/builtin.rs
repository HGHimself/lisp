use crate::{eval, is_qexpr, to_num, Lerr, LerrType, Lval};

pub fn builtin(sym: &str, operands: Vec<Lval>) -> Lval {
    match sym {
        "head" => builtin_head(operands),
        "tail" => builtin_tail(operands),
        "list" => builtin_list(operands),
        "eval" => builtin_eval(operands),
        "join" => builtin_join(operands),
        _ => builtin_op(sym, operands),
    }
}

fn builtin_op(sym: &str, operands: Vec<Lval>) -> Lval {
    // flatten down the numbers
    let results: Option<Vec<f64>> = operands.iter().map(to_num).collect();
    // kick out anything thats not a number
    let operands = match results {
        Some(operands) => operands,
        None => return Lval::Error(Lerr::new(LerrType::BadNum)),
    };

    // handle unary functions
    if operands.len() == 1 {
        if "-" == sym {
            return Lval::Num(-operands[0]);
        } else {
            return Lval::Num(operands[0]);
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
                    return Lval::Error(Lerr::new(LerrType::DivZero));
                } else {
                    x /= y;
                }
            }
            _ => x += y,
        }
        i += 1;
    }

    Lval::Num(x)
}

fn builtin_head(operands: Vec<Lval>) -> Lval {
    // we want only one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    let arg = &operands[0];
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Lval::Error(Lerr::new(LerrType::EmptyList))
            } else {
                qexpr[0].clone()
            }
        }
        _ => Lval::Error(Lerr::new(LerrType::WrongType)),
    }
}

fn builtin_tail(operands: Vec<Lval>) -> Lval {
    // we want only one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    let arg = &operands[0];
    // need a list/qexpr to work with
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Lval::Error(Lerr::new(LerrType::EmptyList))
            } else {
                Lval::Qexpr(qexpr[1..].to_vec())
            }
        }
        _ => Lval::Error(Lerr::new(LerrType::WrongType)),
    }
}

fn builtin_list(operands: Vec<Lval>) -> Lval {
    Lval::Qexpr(operands)
}

fn builtin_eval(operands: Vec<Lval>) -> Lval {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    let arg = &operands[0];
    if let Lval::Qexpr(qexpr) = arg {
        eval::eval(Lval::Sexpr(qexpr[..].to_vec()))
    } else {
        eval::eval(arg.clone())
    }
}

fn builtin_join(operands: Vec<Lval>) -> Lval {
    // need at least 2 arguements
    if operands.len() < 2 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    // needs all arguements to be qexpr
    let results: Vec<bool> = operands
        .iter()
        .map(is_qexpr)
        .filter(|b| *b == false)
        .collect();
    if results.len() > 0 {
        return Lval::Error(Lerr::new(LerrType::WrongType));
    }

    // push each elements from each arguements into one qexpr
    let mut joined = vec![];
    for qexp in operands {
        if let Lval::Qexpr(v) = qexp {
            for item in v {
                joined.push(item);
            }
        }
    }

    Lval::Qexpr(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_uses_head() {
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_head(vec![expr.clone()]),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(
            builtin_head(vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_head(vec![Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_head(vec![Lval::Qexpr(vec![])]),
            Lval::Error(Lerr::new(LerrType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_tail() {
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_tail(vec![expr.clone()]),
            Lval::Qexpr(vec![
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_tail(vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_tail(vec![Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_tail(vec![Lval::Qexpr(vec![])]),
            Lval::Error(Lerr::new(LerrType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_list() {
        let expr = vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ];
        assert_eq!(
            builtin_list(expr.clone()),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_list(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ])
        );
        assert_eq!(builtin_list(vec![]), Lval::Qexpr(vec![]));
        assert_eq!(
            builtin_list(vec![Lval::Sym(String::from("+"))]),
            Lval::Qexpr(vec![Lval::Sym(String::from("+")),])
        );
        assert_eq!(
            builtin_list(vec![Lval::Sexpr(vec![])]),
            Lval::Qexpr(vec![Lval::Sexpr(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(builtin_eval(vec![expr.clone()]), Lval::Num(3_f64));
        assert_eq!(
            builtin_eval(vec![expr.clone(), expr.clone()]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(vec![Lval::Sym(String::from("+"))]),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(builtin_eval(vec![Lval::Qexpr(vec![])]), Lval::Sexpr(vec![]));
    }

    #[test]
    fn it_correctly_uses_join() {
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_join(vec![expr.clone(), expr.clone()]),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
            ])
        );
        assert_eq!(
            builtin_join(vec![expr.clone()]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(vec![expr.clone(), Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_join(vec![expr.clone(), Lval::Qexpr(vec![])]),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
            ])
        );
    }
}
