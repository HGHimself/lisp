use crate::{builtin, init_env, to_fun, Lenv, Lerr, LerrType, Lval};

pub fn eval(env: &mut Lenv, expr: Lval) -> Lval {
    match expr {
        Lval::Sym(s) => eval_symbol(env, s),
        Lval::Num(_) => expr,
        Lval::Error(_) => expr,
        Lval::Qexpr(_) => expr,
        Lval::Fun(_) => expr,
        Lval::Sexpr(vec) => eval_sexpression(env, vec),
    }
}

fn eval_symbol(env: &Lenv, s: String) -> Lval {
    let key = s.to_string();
    match env.get(&key) {
        Some(lval) => lval.clone(),
        None => Lval::Error(Lerr::new(LerrType::UnboundSymbol)),
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
        // else let's try to calculate
        if let Some(fun) = to_fun(&sexpr[0]) {
            // first element needs to be an operator
            let operands = (&sexpr[1..]).to_vec();
            // split off rest of operands and calculate
            return fun(env, operands);
        } else {
            // we needed an operator for the first element to calculate
            return Lval::Error(Lerr::new(LerrType::BadOp));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_singular_numbers() {
        let mut env = init_env();
        assert_eq!(eval(&mut env, Lval::Num(1_f64)), Lval::Num(1_f64));
        assert_eq!(
            eval(&mut env, Lval::Sexpr(vec![Lval::Num(1_f64)])),
            Lval::Num(1_f64)
        );
    }

    #[test]
    fn it_handles_singular_symbols() {
        let mut env = init_env();
        assert_eq!(
            eval(&mut env, Lval::Sym(String::from("+"))),
            Lval::Sym(String::from("+"))
        );
        // assert_eq!(
        //     eval(&mut env, Lval::Sexpr(vec![Lval::Sym(String::from("*"))])),
        //     Lval::Sym(String::from("*"))
        // );
    }

    #[test]
    fn it_handles_singular_errors() {
        let mut env = init_env();
        let error = Lval::Error(Lerr::new(LerrType::DivZero));
        assert_eq!(eval(&mut env, error.clone()), error);
    }

    #[test]
    fn it_handles_empty_expressions() {
        let mut env = init_env();
        assert_eq!(eval(&mut env, Lval::Sexpr(vec![])), Lval::Sexpr(vec![]));
        assert_eq!(
            eval(
                &mut env,
                Lval::Sexpr(vec![Lval::Sexpr(vec![Lval::Sexpr(vec![])])])
            ),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_uses_operators_properly() {
        let mut env = init_env();
        assert_eq!(
            eval(
                &mut env,
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            ),
            Lval::Num(2_f64)
        );
        assert_eq!(
            eval(
                &mut env,
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                ])
            ),
            Lval::Error(Lerr::new(LerrType::BadNum))
        );
        assert_eq!(
            eval(
                &mut env,
                Lval::Sexpr(vec![Lval::Num(1_f64), Lval::Num(1_f64), Lval::Num(1_f64),])
            ),
            Lval::Error(Lerr::new(LerrType::BadOp))
        );
    }

    #[test]
    fn it_handles_nested_sexpressions() {
        let mut env = init_env();
        assert_eq!(
            eval(
                &mut env,
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
}
