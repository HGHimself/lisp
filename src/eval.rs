use crate::{Expression, Symbol};

#[derive(Debug, PartialEq)]
pub enum LispValue {
    Num(f64),
    Error(LispError),
}

#[derive(Debug, PartialEq)]
pub enum LispError {
    DivZero,
    BadOp,
    BadNum,
}

pub fn eval(exp: &Expression) -> LispValue {
    match exp {
        Expression::Num(x) => LispValue::Num(*x),
        Expression::Exp(vec) => {
            let op = &vec[0];
            let mut x = eval(&vec[1]);

            let mut i = 2;
            while i < vec.len() {
                x = eval_op(x, op, eval(&vec[i]));
                i += 1;
            }

            x
        }
        _ => LispValue::Num(0_f64),
    }
}

fn eval_op(x: LispValue, op: &Expression, y: LispValue) -> LispValue {
    let x_value = match x {
        LispValue::Error(_) => return x,
        LispValue::Num(v) => v,
    };

    let y_value = match y {
        LispValue::Error(_) => return y,
        LispValue::Num(v) => v,
    };

    let operator = if let Expression::Sym(operator) = op {
        operator
    } else {
        &Symbol::Add
    };

    match operator {
        Symbol::Add => LispValue::Num(x_value + y_value),
        Symbol::Sub => LispValue::Num(x_value - y_value),
        Symbol::Mul => LispValue::Num(x_value * y_value),
        Symbol::Div => {
            if y_value == 0_f64 {
                LispValue::Error(LispError::DivZero)
            } else {
                LispValue::Num(x_value / y_value)
            }
        }
    }
}
