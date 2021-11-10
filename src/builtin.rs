use crate::{
    add_builtin, eval, init_env, is_qexpr, to_num, to_qexpr, to_sym, Lenv, Lerr, LerrType, Lfun,
    Lval,
};

pub fn init_builtins(env: &mut Lenv) {
    add_builtin(env, "+", builtin_add);
    add_builtin(env, "-", builtin_sub);
    add_builtin(env, "*", builtin_mul);
    add_builtin(env, "/", builtin_div);

    add_builtin(env, "head", builtin_head);
    add_builtin(env, "tail", builtin_tail);
    add_builtin(env, "list", builtin_list);
    add_builtin(env, "eval", builtin_eval);
    add_builtin(env, "join", builtin_join);
    add_builtin(env, "def", builtin_def);
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

fn builtin_add(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    builtin_op("+", operands)
}

fn builtin_sub(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    builtin_op("-", operands)
}

fn builtin_mul(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    builtin_op("*", operands)
}

fn builtin_div(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    builtin_op("/", operands)
}

fn builtin_head(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
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

fn builtin_tail(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
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

fn builtin_list(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    Lval::Qexpr(operands)
}

fn builtin_eval(env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    let arg = &operands[0];
    if let Lval::Qexpr(qexpr) = arg {
        eval::eval(env, Lval::Sexpr(qexpr[..].to_vec()))
    } else {
        eval::eval(env, arg.clone())
    }
}

fn builtin_join(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
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

fn builtin_def(env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    // need at least an arguement set and 1 value
    if operands.len() < 2 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }
    // need a param list
    if is_qexpr(&operands[0]) == false {
        return Lval::Error(Lerr::new(LerrType::WrongType));
    }

    // need each argument to be a symbol
    let results: Option<Vec<String>> = to_qexpr(&operands[0]).unwrap().iter().map(to_sym).collect();
    let args = match results {
        None => return Lval::Error(Lerr::new(LerrType::WrongType)),
        Some(v) => v,
    };

    // need to have the same number of args and values to assign
    if args.len() != operands.len() - 1 {
        return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
    }

    // assign each arg to a corresponding value
    for (i, arg) in args.into_iter().enumerate() {
        env.insert(arg, operands[i + 1].clone());
    }

    Lval::Sexpr(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_fun(_env: &mut Lenv, _operands: Vec<Lval>) -> Lval {
        Lval::Sexpr(vec![])
    }

    #[test]
    fn it_correctly_uses_head() {
        let mut env = init_env();
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
            builtin_head(&mut env, vec![expr.clone()]),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(
            builtin_head(&mut env, vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_head(&mut env, vec![Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_head(&mut env, vec![Lval::Qexpr(vec![])]),
            Lval::Error(Lerr::new(LerrType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_tail() {
        let mut env = init_env();
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
            builtin_tail(&mut env, vec![expr.clone()]),
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
            builtin_tail(&mut env, vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_tail(&mut env, vec![Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_tail(&mut env, vec![Lval::Qexpr(vec![])]),
            Lval::Error(Lerr::new(LerrType::EmptyList))
        );
    }

    #[test]
    fn it_correctly_uses_list() {
        let mut env = init_env();
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
            builtin_list(&mut env, expr.clone()),
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
            builtin_list(
                &mut env,
                vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]
            ),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ])
        );
        assert_eq!(builtin_list(&mut env, vec![]), Lval::Qexpr(vec![]));
        assert_eq!(
            builtin_list(&mut env, vec![Lval::Sym(String::from("+"))]),
            Lval::Qexpr(vec![Lval::Sym(String::from("+")),])
        );
        assert_eq!(
            builtin_list(&mut env, vec![Lval::Sexpr(vec![])]),
            Lval::Qexpr(vec![Lval::Sexpr(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let mut env = init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(builtin_eval(&mut env, vec![expr.clone()]), Lval::Num(3_f64));
        assert_eq!(
            builtin_eval(&mut env, vec![expr.clone(), expr.clone()]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(&mut env, vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_eval(&mut env, vec![Lval::Sym(String::from("-"))]),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(
                &mut env,
                vec![Lval::Sexpr(vec![Lval::Sym(String::from("-"))])]
            ),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(&mut env, vec![Lval::Qexpr(vec![])]),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_correctly_uses_join() {
        let mut env = init_env();
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
            builtin_join(&mut env, vec![expr.clone(), expr.clone()]),
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
            builtin_join(&mut env, vec![expr.clone()]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(&mut env, vec![]),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_join(&mut env, vec![expr.clone(), Lval::Sym(String::from("+"))]),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
        assert_eq!(
            builtin_join(&mut env, vec![expr.clone(), Lval::Qexpr(vec![])]),
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

    #[test]
    fn it_correctly_uses_define() {
        let mut env = init_env();
        assert_eq!(
            builtin_def(
                &mut env,
                vec![
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                        Lval::Sym(String::from("c"))
                    ]),
                    Lval::Num(1_f64),
                    Lval::Sym(String::from("+")),
                    Lval::Sexpr(vec![]),
                ]
            ),
            Lval::Sexpr(vec![])
        );
        assert_eq!(
            crate::eval::eval(&mut env, Lval::Sym(String::from("a"))),
            Lval::Num(1_f64)
        );
        assert_eq!(
            crate::eval::eval(&mut env, Lval::Sym(String::from("b"))),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(
            crate::eval::eval(&mut env, Lval::Sym(String::from("c"))),
            Lval::Sexpr(vec![])
        );
        assert_eq!(
            builtin_def(
                &mut env,
                vec![Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                    Lval::Sym(String::from("c"))
                ]),]
            ),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_def(
                &mut env,
                vec![
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                    ]),
                    Lval::Num(1_f64),
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("+")),
                ]
            ),
            Lval::Error(Lerr::new(LerrType::IncorrectParamCount))
        );
        assert_eq!(
            builtin_def(
                &mut env,
                vec![Lval::Qexpr(vec![Lval::Num(1_f64),]), Lval::Num(1_f64),]
            ),
            Lval::Error(Lerr::new(LerrType::WrongType))
        );
    }
}
