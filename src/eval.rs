use crate::{Expression, LispError, Symbol};

pub fn eval(expr: Expression) -> Expression {
    match expr {
        Expression::Sym(_) => expr,
        Expression::Num(_) => expr,
        Expression::Sexp(vec) => eval_sexpression(vec),
        _ => Expression::Num(0_f64),
    }
}

fn is_err(expr: &Expression) -> bool {
    match expr {
        Expression::Error(_) => true,
        _ => false,
    }
}

fn is_num(expr: &Expression) -> bool {
    match expr {
        Expression::Num(_) => true,
        _ => false,
    }
}

fn to_num(expr: &Expression) -> Option<f64> {
    if let Expression::Num(n) = expr {
        Some(*n)
    } else {
        None
    }
}

fn to_sym(expr: &Expression) -> Option<Symbol> {
    if let Expression::Sym(s) = expr {
        Some(*s)
    } else {
        None
    }
}

fn eval_sexpression(sexpr: Vec<Expression>) -> Expression {
    let results = sexpr
        .into_iter()
        .map(|expr| eval(expr))
        .collect::<Vec<Expression>>();

    for expr in &results {
        if is_err(&expr) {
            return expr.clone();
        }
    }

    if results.len() == 0 {
        return Expression::Sexp(results);
    } else if results.len() == 1 {
        return results[0].clone();
    } else {
        if let Some(sym) = to_sym(&results[0]) {
            let operands = (&results[1..]).to_vec();
            return builtin_op(sym, operands);
        } else {
            return Expression::Error(LispError::BadOp);
        }
    }
}

fn builtin_op(sym: Symbol, operands: Vec<Expression>) -> Expression {
    if let Some(_) = operands.iter().position(|expr| !is_num(expr)) {
        return Expression::Error(LispError::BadNum);
    } else {
        let numbers = operands
            .iter()
            .map(to_num)
            .map(|n| n.unwrap())
            .collect::<Vec<f64>>();
        if operands.len() == 1 {
            if let Symbol::Sub = sym {
                return Expression::Num(-numbers[0]);
            } else {
                return Expression::Num(numbers[0]);
            }
        } else {
            let mut x = numbers[0];
            let mut i = 1;
            while i < numbers.len() {
                let y = numbers[i];
                match sym {
                    Symbol::Add => x += y,
                    Symbol::Sub => x -= y,
                    Symbol::Mul => x *= y,
                    Symbol::Div => {
                        if y == 0_f64 {
                            return Expression::Error(LispError::DivZero);
                        } else {
                            x /= y;
                        }
                    }
                }
                i += 1;
            }

            Expression::Num(x)
        }
    }
}
