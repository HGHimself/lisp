pub mod alloc;
pub mod eval;
pub mod parser;
pub mod prompt;
pub mod report;
pub mod sample;

use std::{error::Error, fmt, iter::FromIterator};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Symbol {
    Add,
    Sub,
    Mul,
    Div,
    List,
    Head,
    Tail,
    Join,
    Eval,
    Noop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LispError {
    details: String,
    etype: LispErrorType,
}

impl LispError {
    fn new(etype: LispErrorType) -> LispError {
        let msg = match &etype {
            LispErrorType::DivZero => "Cannot Divide By Zero",
            LispErrorType::BadOp => "Invalid Operator",
            LispErrorType::BadNum => "Invalid Operand",
            LispErrorType::IncorrectParamCount => "Incorrect Number of Params passed to function",
            LispErrorType::WrongType => "Incorrect Data Type used",
            LispErrorType::EmptyList => "Empty List passed to function",
        };

        LispError {
            details: msg.to_string(),
            etype,
        }
    }
}

impl fmt::Display for LispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for LispError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LispErrorType {
    DivZero,
    BadOp,
    BadNum,
    IncorrectParamCount,
    EmptyList,
    WrongType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Sym(Symbol),
    Num(f64),
    Sexp(Vec<Expression>),
    Qexp(Vec<Expression>),
    Error(LispError),
}

// FromIterator<Expression>` is not implemented for `Result<Vec<Expression>, _>
impl FromIterator<Expression> for Result<Vec<Expression>, LispError> {
    fn from_iter<I: IntoIterator<Item = Expression>>(iter: I) -> Self {
        let mut c = vec![];

        for i in iter {
            match i {
                Expression::Error(e) => return Err(e),
                _ => c.push(i),
            }
        }

        Ok(c)
    }
}

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

fn string_to_symbol(s: &str) -> Symbol {
    if s == "list" {
        Symbol::List
    } else if s == "head" {
        Symbol::Head
    } else if s == "tail" {
        Symbol::Tail
    } else if s == "join" {
        Symbol::Join
    } else if s == "eval" {
        Symbol::Eval
    } else {
        Symbol::Noop
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

// fn to_qexp(expr: &Expression) -> Option<Vec<Expression>> {
//     if let Expression::Qexp(q) = expr {
//         Some(*q)
//     } else {
//         None
//     }
// }

fn is_qexp(expr: &Expression) -> bool {
    if let Expression::Qexp(q) = expr {
        true
    } else {
        false
    }
}

fn to_sym(expr: &Expression) -> Option<Symbol> {
    if let Expression::Sym(s) = expr {
        Some(*s)
    } else {
        None
    }
}
