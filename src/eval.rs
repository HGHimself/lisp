use crate::{to_fun, to_lambda, Lenv, Lerr, LerrType, Llambda, Lval};

pub fn eval(env: &mut Lenv, expr: Lval) -> Lval {
    match expr {
        Lval::Sym(s) => eval_symbol(env, s),
        Lval::Num(_) => expr,
        Lval::Error(_) => expr,
        Lval::Qexpr(_) => expr,
        Lval::Fun(_) => expr,
        Lval::Lambda(_) => expr,
        Lval::Sexpr(vec) => eval_sexpression(env, vec),
    }
}

fn eval_symbol(env: &mut Lenv, s: String) -> Lval {
    let key = s.to_string();
    match env.get(&key) {
        Some(lval) => lval.clone(),
        None => Lval::Error(Lerr::new(
            LerrType::UnboundSymbol,
            format!("{:?} has not been defined", key),
        )),
    }
}

fn eval_sexpression(env: &mut Lenv, sexpr: Vec<Lval>) -> Lval {
    // evaluate each element
    let results: Result<Vec<Lval>, Lerr> = sexpr.into_iter().map(|expr| eval(env, expr)).collect();
    // surface any errors
    let sexpr = match results {
        Ok(sexpr) => sexpr,
        Err(e) => return Lval::Error(e),
    };

    if sexpr.len() == 0 {
        // if empty return empty
        return Lval::Sexpr(sexpr);
    } else if sexpr.len() == 1 {
        // if singular value return singular value
        return sexpr[0].clone();
    } else {
        let operands = (&sexpr[1..]).to_vec();
        // else let's try to calculate
        if let Some(fun) = to_fun(&sexpr[0]) {
            // builtin
            return fun(env, operands);
        } else if let Some(lambda) = to_lambda(&sexpr[0]) {
            // lambda
            return call(env, lambda, operands);
        } else {
            // we needed an operator for the first element to calculate
            return Lval::Error(Lerr::new(
                LerrType::BadOp,
                format!("{:?} is not a valid operator", sexpr[0]),
            ));
        }
    }
}

pub fn call(env: &mut Lenv, mut func: Llambda, mut args: Vec<Lval>) -> Lval {
    let given = args.len();
    let total = func.args.len();

    while args.len() != 0 {
        if func.args.len() == 0 {
            return Lval::Error(Lerr::new(
                LerrType::IncorrectParamCount,
                format!("Function needed {} args but was given {}", total, given),
            ));
        }

        let sym = func.args.pop().unwrap();
        let val = args.pop().unwrap();

        func.env.insert(&sym, val);
    }

    if func.args.len() == 0 {
        env.push(func.env.peek().unwrap().clone());
        let res = eval(env, Lval::Sexpr(func.body));
        env.pop();
        res
    } else {
        Lval::Lambda(func)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_env, to_err};

    fn empty_fun(_env: &mut Lenv, _operands: Vec<Lval>) -> Lval {
        Lval::Sexpr(vec![])
    }

    #[test]
    fn it_handles_singular_numbers() {
        let env = &mut init_env();
        assert_eq!(eval(env, Lval::Num(1_f64)), Lval::Num(1_f64));
        assert_eq!(
            eval(env, Lval::Sexpr(vec![Lval::Num(1_f64)])),
            Lval::Num(1_f64)
        );
    }

    #[test]
    fn it_handles_singular_symbols() {
        let env = &mut init_env();
        assert_eq!(
            eval(env, Lval::Sym(String::from("+"))),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            eval(env, Lval::Sexpr(vec![Lval::Sym(String::from("*"))])),
            Lval::Fun(empty_fun)
        );
    }

    #[test]
    fn it_handles_singular_errors() {
        let env = &mut init_env();
        let error = Lval::Error(Lerr::new(LerrType::DivZero, String::from("")));
        assert_eq!(
            to_err(&eval(env, error.clone())).unwrap().etype,
            LerrType::DivZero
        );
    }

    #[test]
    fn it_handles_empty_expressions() {
        let env = &mut init_env();
        assert_eq!(eval(env, Lval::Sexpr(vec![])), Lval::Sexpr(vec![]));
        assert_eq!(
            eval(
                env,
                Lval::Sexpr(vec![Lval::Sexpr(vec![Lval::Sexpr(vec![])])])
            ),
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
            ),
            Lval::Num(2_f64)
        );
        assert_eq!(
            to_err(&eval(
                env,
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                ])
            ))
            .unwrap()
            .etype,
            LerrType::BadNum
        );
        assert_eq!(
            to_err(&eval(
                env,
                Lval::Sexpr(vec![Lval::Num(1_f64), Lval::Num(1_f64), Lval::Num(1_f64),])
            ))
            .unwrap()
            .etype,
            LerrType::BadOp
        );
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
            ),
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
        assert_eq!(call(env, lambda, vec![Lval::Num(5_f64)]), Lval::Num(10_f64));

        let lambda = Llambda::new(
            vec![String::from("a"), String::from("b")],
            vec![
                Lval::Sym(String::from("*")),
                Lval::Sym(String::from("b")),
                Lval::Sym(String::from("a")),
            ],
        );
        let new_lambda = call(env, lambda, vec![Lval::Num(15_f64)]);
        assert_eq!(
            call(env, to_lambda(&new_lambda).unwrap(), vec![Lval::Num(5_f64)]),
            Lval::Num(75_f64)
        );
    }
}
