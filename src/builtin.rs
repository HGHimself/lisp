use crate::{
    add_builtin, eval, is_qexpr, to_num, to_qexpr, to_sym, Lenv, Lerr, LerrType, Llambda, Lval,
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
    add_builtin(env, "\\", builtin_lambda);
    add_builtin(env, "def", builtin_def);
    add_builtin(env, "die", builtin_exit);
}

fn builtin_op(sym: &str, operands: Vec<Lval>) -> Lval {
    // flatten down the numbers
    let results: Option<Vec<f64>> = operands.iter().map(to_num).collect();
    // kick out anything thats not a number
    let operands = match results {
        Some(operands) => operands,
        None => {
            return Lval::Error(Lerr::new(
                LerrType::BadNum,
                format!("Function {} can operate only on numbers", sym),
            ))
        }
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
                    return Lval::Error(Lerr::new(
                        LerrType::DivZero,
                        format!("You cannot divide {}, or any number, by 0", x),
                    ));
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

fn builtin_exit(_env: &mut Lenv, _operands: Vec<Lval>) -> Lval {
    Lval::Error(Lerr::new(
        LerrType::Interrupt,
        String::from("The thread of execution has been interrupted"),
    ))
}

fn builtin_head(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    // we want only one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function head needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Lval::Error(Lerr::new(
                    LerrType::EmptyList,
                    format!("Function head was given empty list"),
                ))
            } else {
                Lval::Qexpr(vec![qexpr[0].clone()])
            }
        }
        _ => Lval::Error(Lerr::new(
            LerrType::WrongType,
            format!("Function head needed Qexpr but was given {:?}", arg),
        )),
    }
}

fn builtin_tail(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    // we want only one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function tail needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    // need a list/qexpr to work with
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Lval::Error(Lerr::new(
                    LerrType::EmptyList,
                    format!("Function tail was given empty list"),
                ))
            } else {
                Lval::Qexpr(qexpr[1..].to_vec())
            }
        }
        _ => Lval::Error(Lerr::new(
            LerrType::WrongType,
            format!("Function tail needed Qexpr but was given {:?}", arg),
        )),
    }
}

fn builtin_list(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    Lval::Qexpr(operands)
}

fn builtin_eval(env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function eval needed 1 arg but was given {}",
                operands.len()
            ),
        ));
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
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function join needed 2 arg but was given {}",
                operands.len()
            ),
        ));
    }

    // needs all arguements to be qexpr
    let results: Vec<bool> = operands
        .iter()
        .map(is_qexpr)
        .filter(|b| *b == false)
        .collect();
    if results.len() > 0 {
        return Lval::Error(Lerr::new(
            LerrType::WrongType,
            format!("Function join needed Qexpr but was given"),
        ));
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
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function def needed 2 args but was given {}",
                operands.len()
            ),
        ));
    }

    // need a param list
    if is_qexpr(&operands[0]) == false {
        return Lval::Error(Lerr::new(
            LerrType::WrongType,
            format!("Function def needed Qexpr but was given {:?}", operands[0]),
        ));
    }

    // need each argument to be a symbol
    let results: Option<Vec<String>> = to_qexpr(&operands[0]).unwrap().iter().map(to_sym).collect();
    let args = match results {
        None => {
            return Lval::Error(Lerr::new(
                LerrType::WrongType,
                format!("Function def needed a param list of all Symbols"),
            ))
        }
        Some(v) => v,
    };

    // need to have the same number of args and values to assign
    if args.len() != operands.len() - 1 {
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function def needed to assign {} values but was passed {}",
                args.len(),
                operands.len() - 1
            ),
        ));
    }

    // assign each arg to a corresponding value
    for (i, arg) in args.into_iter().enumerate() {
        env.insert_last(&arg, operands[i + 1].clone());
    }

    Lval::Sexpr(vec![])
}

fn builtin_lambda(_env: &mut Lenv, operands: Vec<Lval>) -> Lval {
    if operands.len() != 2 {
        return Lval::Error(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function \\ needed 2 arg but was given {}", operands.len()),
        ));
    }

    // needs all arguements to be qexpr
    let results: Vec<bool> = operands
        .iter()
        .map(is_qexpr)
        .filter(|b| *b == false)
        .collect();
    if results.len() > 0 {
        return Lval::Error(Lerr::new(
            LerrType::WrongType,
            format!("Function \\ needed a Qexpr for arguments and a Qexpr for body"),
        ));
    }

    // need each argument to be a symbol
    let results: Option<Vec<String>> = to_qexpr(&operands[0]).unwrap().iter().map(to_sym).collect();
    let args = match results {
        None => {
            return Lval::Error(Lerr::new(
                LerrType::WrongType,
                format!("Function \\ needed a param list of all Symbols"),
            ))
        }
        Some(v) => v,
    };

    // we reverse these so that we can pop off the back in the call func
    let params = args;
    //.into_iter().rev().collect();
    let body = &operands[1];

    let lambda = Llambda::new(params, to_qexpr(body).unwrap());

    Lval::Lambda(lambda)
}

// fn builtin_var(env: Lenv, operands: Vec<Lval>) -> Lval {
//     // need at least an arguement set and 1 value
//     if operands.len() < 2 {
//         return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
//     }
//     // need a param list
//     if is_qexpr(&operands[0]) == false {
//         return Lval::Error(Lerr::new(LerrType::WrongType));
//     }
//
//     // need each argument to be a symbol
//     let results: Option<Vec<String>> = to_qexpr(&operands[0]).unwrap().iter().map(to_sym).collect();
//     let args = match results {
//         None => return Lval::Error(Lerr::new(LerrType::WrongType)),
//         Some(v) => v,
//     };
//
//     // need to have the same number of args and values to assign
//     if args.len() != operands.len() - 1 {
//         return Lval::Error(Lerr::new(LerrType::IncorrectParamCount));
//     }
//
//     // assign each arg to a corresponding value
//     for (i, arg) in args.into_iter().enumerate() {
//         env.insert(arg, operands[i + 1].clone());
//     }
//
//     Lval::Sexpr(vec![])
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_env, to_err, to_lambda};

    fn empty_fun(_env: &mut Lenv, _operands: Vec<Lval>) -> Lval {
        Lval::Sexpr(vec![])
    }

    #[test]
    fn it_correctly_uses_head() {
        let env = &mut init_env();
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
            builtin_head(env, vec![expr.clone()]),
            Lval::Qexpr(vec![Lval::Sym(String::from("+"))])
        );
        assert_eq!(
            to_err(&builtin_head(env, vec![])).unwrap().etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_head(env, vec![Lval::Sym(String::from("+"))]))
                .unwrap()
                .etype,
            LerrType::WrongType
        );
        assert_eq!(
            to_err(&builtin_head(env, vec![Lval::Qexpr(vec![])]))
                .unwrap()
                .etype,
            LerrType::EmptyList
        );
    }

    #[test]
    fn it_correctly_uses_tail() {
        let env = &mut init_env();
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
            builtin_tail(env, vec![expr.clone()]),
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
            to_err(&builtin_tail(env, vec![])).unwrap().etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_tail(env, vec![Lval::Sym(String::from("+"))]))
                .unwrap()
                .etype,
            LerrType::WrongType
        );
        assert_eq!(
            to_err(&builtin_tail(env, vec![Lval::Qexpr(vec![])]))
                .unwrap()
                .etype,
            LerrType::EmptyList
        );
    }

    #[test]
    fn it_correctly_uses_list() {
        let env = &mut init_env();
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
            builtin_list(env, expr.clone()),
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
                env,
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
        assert_eq!(builtin_list(env, vec![]), Lval::Qexpr(vec![]));
        assert_eq!(
            builtin_list(env, vec![Lval::Sym(String::from("+"))]),
            Lval::Qexpr(vec![Lval::Sym(String::from("+")),])
        );
        assert_eq!(
            builtin_list(env, vec![Lval::Sexpr(vec![])]),
            Lval::Qexpr(vec![Lval::Sexpr(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let env = &mut init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(builtin_eval(env, vec![expr.clone()]), Lval::Num(3_f64));
        assert_eq!(
            to_err(&builtin_eval(env, vec![expr.clone(), expr.clone()]))
                .unwrap()
                .etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_eval(env, vec![])).unwrap().etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            builtin_eval(env, vec![Lval::Sym(String::from("-"))]),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(env, vec![Lval::Sexpr(vec![Lval::Sym(String::from("-"))])]),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(env, vec![Lval::Qexpr(vec![])]),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_correctly_uses_join() {
        let env = &mut init_env();
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
            builtin_join(env, vec![expr.clone(), expr.clone()]),
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
            to_err(&builtin_join(env, vec![expr.clone()]))
                .unwrap()
                .etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_join(env, vec![])).unwrap().etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_join(
                env,
                vec![expr.clone(), Lval::Sym(String::from("+"))]
            ))
            .unwrap()
            .etype,
            LerrType::WrongType
        );
        assert_eq!(
            builtin_join(env, vec![expr.clone(), Lval::Qexpr(vec![])]),
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
        let env = &mut init_env();
        assert_eq!(
            builtin_def(
                env,
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
            crate::eval::eval(env, Lval::Sym(String::from("a"))),
            Lval::Num(1_f64)
        );
        assert_eq!(
            crate::eval::eval(env, Lval::Sym(String::from("b"))),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(
            crate::eval::eval(env, Lval::Sym(String::from("c"))),
            Lval::Sexpr(vec![])
        );
        assert_eq!(
            to_err(&builtin_def(
                env,
                vec![Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                    Lval::Sym(String::from("c"))
                ]),]
            ))
            .unwrap()
            .etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_def(
                env,
                vec![
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                    ]),
                    Lval::Num(1_f64),
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("+")),
                ]
            ))
            .unwrap()
            .etype,
            LerrType::IncorrectParamCount
        );
        assert_eq!(
            to_err(&builtin_def(
                env,
                vec![Lval::Qexpr(vec![Lval::Num(1_f64),]), Lval::Num(1_f64),]
            ))
            .unwrap()
            .etype,
            LerrType::WrongType
        );
    }

    //(\ {a b} {* a b}) 1 2
    #[test]
    fn it_correctly_uses_lambda() {
        let env = &mut init_env();
        assert!(to_lambda(&builtin_lambda(
            env,
            vec![
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
            ]
        ))
        .is_some());

        let expr = Lval::Sexpr(vec![
            Lval::Sexpr(vec![
                Lval::Sym(String::from("\\")),
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
            ]),
            Lval::Num(2_f64),
            Lval::Num(2_f64),
        ]);
        assert_eq!(eval::eval(env, expr), Lval::Num(4_f64));
    }
}
