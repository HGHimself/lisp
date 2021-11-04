pub mod alloc;
pub mod eval;
pub mod parser;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Op(Operator),
    Num(f64),
    Exp(Expressions),
}

pub type Expressions = Vec<Expression>;

fn char_to_operator(c: char) -> Operator {
    if c == '+' {
        Operator::Add
    } else if c == '-' {
        Operator::Sub
    } else if c == '*' {
        Operator::Mul
    } else if c == '/' {
        Operator::Div
    } else {
        Operator::Add
    }
}
