use crate::{Lenv, Lerr, LerrType, Llambda, Lval};

pub fn eval(env: &mut Lenv, expr: Lval) -> Result<Lval, Lerr> {
    match expr {
        Lval::Sym(s) => eval_symbol(env, s),
        Lval::Sexpr(vec) => eval_sexpression(env, vec),
        _ => Ok(expr),
    }
}

fn eval_symbol(env: &mut Lenv, s: String) -> Result<Lval, Lerr> {
    match env.get(&s) {
        Some(lval) => Ok(lval.clone()),
        None => Err(Lerr::new(
            LerrType::UnboundSymbol,
            format!("{:?} has not been defined", s),
        )),
    }
}

fn eval_sexpression(env: &mut Lenv, sexpr: Vec<Lval>) -> Result<Lval, Lerr> {
    // evaluate each element
    let results = sexpr
        .into_iter()
        .map(|expr| eval(env, expr))
        .collect::<Result<Vec<Lval>, Lerr>>()?;

    if results.len() == 0 {
        // if empty return empty
        return Ok(Lval::Sexpr(results));
    } else if results.len() == 1 {
        // if singular value return singular value
        return Ok(results[0].clone());
    } else {
        let operands = (&results[1..]).to_vec();
        // recognize a builtin function or a lambda
        match results[0].clone() {
            Lval::Fun(fun) => fun(env, operands),
            Lval::Lambda(lambda) => call(env, lambda, operands),
            _ => Err(Lerr::new(
                LerrType::BadOp,
                format!("{:?} is not a valid operator", results[0]),
            )),
        }
    }
}

pub fn call(env: &mut Lenv, mut func: Llambda, mut args: Vec<Lval>) -> Result<Lval, Lerr> {
    let given = args.len();
    let total = func.args.len();

    while args.len() != 0 {
        if func.args.len() == 0 {
            return Err(Lerr::new(
                LerrType::IncorrectParamCount,
                format!("Function needed {} args but was given {}", total, given),
            ));
        }

        let sym = func.args[0].clone();
        func.args = func.args[1..].to_vec();

        if sym == ":" {
            if func.args.len() != 1 {
                return Err(Lerr::new(
                    LerrType::IncorrectParamCount,
                    format!(": operator needs to be followed by arg"),
                ));
            }

            let sym = func.args[0].clone();
            func.args = func.args[1..].to_vec();
            func.env.insert(&sym, Lval::Qexpr(args));

            break;
        } else {
            let val = args[0].clone();
            args = args[1..].to_vec();
            func.env.insert(&sym, val);
        }
    }

    if func.args.len() == 0 {
        env.push(func.env.peek().unwrap().clone());
        let res = eval(env, Lval::Sexpr(func.body));
        env.pop();
        res
    } else {
        Ok(Lval::Lambda(func))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_env, to_lambda};

    fn empty_fun(_env: &mut Lenv, _operands: Vec<Lval>) -> Result<Lval, Lerr> {
        Ok(Lval::Sexpr(vec![]))
    }

    #[test]
    fn it_handles_singular_numbers() {
        let env = &mut init_env();
        assert_eq!(eval(env, Lval::Num(1_f64)).unwrap(), Lval::Num(1_f64));
        assert_eq!(
            eval(env, Lval::Sexpr(vec![Lval::Num(1_f64)])).unwrap(),
            Lval::Num(1_f64)
        );
    }

    #[test]
    fn it_handles_singular_symbols() {
        let env = &mut init_env();
        assert_eq!(
            eval(env, Lval::Sym(String::from("+"))).unwrap(),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            eval(env, Lval::Sexpr(vec![Lval::Sym(String::from("*"))])).unwrap(),
            Lval::Fun(empty_fun)
        );
    }

    // #[test]
    // fn it_handles_singular_errors() {
    //     let env = &mut init_env();
    //     let error = Lval::Error(Lerr::new(LerrType::DivZero, String::from("")));
    //     assert_eq!(
    //         to_err(&eval(env, error.clone())).unwrap().etype,
    //         LerrType::DivZero
    //     );
    // }

    #[test]
    fn it_handles_empty_expressions() {
        let env = &mut init_env();
        assert_eq!(eval(env, Lval::Sexpr(vec![])).unwrap(), Lval::Sexpr(vec![]));
        assert_eq!(
            eval(
                env,
                Lval::Sexpr(vec![Lval::Sexpr(vec![Lval::Sexpr(vec![])])])
            )
            .unwrap(),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_uses_operators_properly() {
        let env = &mut init_env();
        assert_eq!(
            eval(
                env,
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            )
            .unwrap(),
            Lval::Num(2_f64)
        );
        let _ = eval(
            env,
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
            ]),
        )
        .map_err(|err| assert_eq!(err.etype, LerrType::BadNum));
        let _ = eval(
            env,
            Lval::Sexpr(vec![Lval::Num(1_f64), Lval::Num(1_f64), Lval::Num(1_f64)]),
        )
        .map_err(|err| assert_eq!(err.etype, LerrType::BadOp));
    }

    #[test]
    fn it_handles_nested_sexpressions() {
        let env = &mut init_env();
        assert_eq!(
            eval(
                env,
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Sexpr(vec![
                        Lval::Sym(String::from("+")),
                        Lval::Num(1_f64),
                        Lval::Num(1_f64),
                    ]),
                ])
            )
            .unwrap(),
            Lval::Num(3_f64)
        );
    }

    #[test]
    fn it_handles_lambdas() {
        let env = &mut init_env();
        let lambda = Llambda::new(
            vec![String::from("a")],
            vec![
                Lval::Sym(String::from("+")),
                Lval::Sym(String::from("a")),
                Lval::Sym(String::from("a")),
            ],
        );
        assert_eq!(
            call(env, lambda, vec![Lval::Num(5_f64)]).unwrap(),
            Lval::Num(10_f64)
        );

        let lambda = Llambda::new(
            vec![String::from("a"), String::from("b")],
            vec![
                Lval::Sym(String::from("*")),
                Lval::Sym(String::from("b")),
                Lval::Sym(String::from("a")),
            ],
        );
        let new_lambda = call(env, lambda, vec![Lval::Num(15_f64)]).unwrap();
        assert_eq!(
            call(env, to_lambda(&new_lambda).unwrap(), vec![Lval::Num(5_f64)]).unwrap(),
            Lval::Num(75_f64)
        );
    }
}
