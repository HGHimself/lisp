pub mod alloc;
pub mod eval;
pub mod parser;
pub mod prompt;
pub mod report;
pub mod sample;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Symbol {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Sym(Symbol),
    Num(f64),
    Exp(Expressions),
}

pub type Expressions = Vec<Expression>;

fn char_to_symbol(c: char) -> Symbol {
    if c == '+' {
        Symbol::Add
    } else if c == '-' {
        Symbol::Sub
    } else if c == '*' {
        Symbol::Mul
    } else if c == '/' {
        Symbol::Div
    } else {
        Symbol::Add
    }
}
