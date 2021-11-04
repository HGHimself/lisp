use crate::{Expression, Operator};

pub fn eval(exp: &Expression) -> f64 {
    match exp {
        Expression::Num(x) => *x,
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
        Expression::Op(_) => 0_f64,
    }
}

fn eval_op(x: f64, op: &Expression, y: f64) -> f64 {
    let operator = if let Expression::Op(operator) = op {
        operator
    } else {
        &Operator::Add
    };

    match operator {
        Operator::Add => x + y,
        Operator::Sub => x - y,
        Operator::Mul => x * y,
        Operator::Div => x / y,
    }
}
