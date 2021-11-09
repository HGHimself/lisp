use crate::{builtin, to_sym, Lerr, LerrType, Lval};

pub fn eval(expr: Lval) -> Lval {
    match expr {
        Lval::Sym(_) => expr,
        Lval::Num(_) => expr,
        Lval::Error(_) => expr,
        Lval::Qexpr(_) => expr,
        Lval::Sexpr(vec) => eval_sexpression(vec),
        Lval::Fun(_) => expr,
    }
}

fn eval_sexpression(sexpr: Vec<Lval>) -> Lval {
    // evaluate each element
    let results: Result<Vec<Lval>, Lerr> = sexpr.into_iter().map(|expr| eval(expr)).collect();
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
        if let Some(sym) = to_sym(&sexpr[0]) {
            // first element needs to be an operator
            let operands = (&sexpr[1..]).to_vec();
            // split off rest of operands and calculate
            return builtin::builtin(&sym, operands);
        } else {
            // we needed an operator for the first element to calculate
            return Lval::Error(Lerr::new(LerrType::BadOp));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_handles_singular_numbers() {
        assert_eq!(eval(Lval::Num(1_f64)), Lval::Num(1_f64));
        assert_eq!(eval(Lval::Sexpr(vec![Lval::Num(1_f64)])), Lval::Num(1_f64));
    }

    #[test]
    fn it_handles_singular_symbols() {
        assert_eq!(
            eval(Lval::Sym(String::from("*"))),
            Lval::Sym(String::from("*"))
        );
        assert_eq!(
            eval(Lval::Sexpr(vec![Lval::Sym(String::from("*"))])),
            Lval::Sym(String::from("*"))
        );
    }

    #[test]
    fn it_handles_singular_errors() {
        let error = Lval::Error(Lerr::new(LerrType::DivZero));
        assert_eq!(eval(error.clone()), error);
    }

    #[test]
    fn it_handles_empty_expressions() {
        assert_eq!(eval(Lval::Sexpr(vec![])), Lval::Sexpr(vec![]));
        assert_eq!(
            eval(Lval::Sexpr(vec![Lval::Sexpr(vec![Lval::Sexpr(vec![])])])),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_uses_operators_properly() {
        assert_eq!(
            eval(Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ])),
            Lval::Num(2_f64)
        );
        assert_eq!(
            eval(Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
            ])),
            Lval::Error(Lerr::new(LerrType::BadNum))
        );
        assert_eq!(
            eval(Lval::Sexpr(vec![
                Lval::Num(1_f64),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ])),
            Lval::Error(Lerr::new(LerrType::BadOp))
        );
    }

    #[test]
    fn it_handles_nested_sexpressions() {
        assert_eq!(
            eval(Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
            ])),
            Lval::Num(3_f64)
        );
    }
}
