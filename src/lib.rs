pub mod alloc;
// pub mod builtin;
// pub mod eval;
pub mod parser;
// pub mod prompt;
// pub mod report;
// pub mod sample;

// use crate::builtin::init_builtins;
use std::{collections::HashMap, error::Error, fmt, iter::FromIterator};

#[derive(Clone, Debug, PartialEq)]
pub struct Lerr {
    details: String,
    etype: LerrType,
}

impl Lerr {
    fn new(etype: LerrType) -> Lerr {
        let msg = match &etype {
            LerrType::DivZero => "Cannot Divide By Zero",
            LerrType::BadOp => "Invalid Operator",
            LerrType::BadNum => "Invalid Operand",
            LerrType::IncorrectParamCount => "Incorrect Number of Params passed to function",
            LerrType::WrongType => "Incorrect Data Type used",
            LerrType::EmptyList => "Empty List passed to function",
            LerrType::UnboundSymbol => "This Symbol has not been Defined",
        };

        Lerr {
            details: msg.to_string(),
            etype,
        }
    }
}

impl fmt::Display for Lerr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for Lerr {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LerrType {
    DivZero,
    BadOp,
    BadNum,
    IncorrectParamCount,
    EmptyList,
    WrongType,
    UnboundSymbol,
}

#[derive(Clone)]
pub enum Lval {
    Sym(String),
    Num(f64),
    Sexpr(Vec<Lval>),
    Qexpr(Vec<Lval>),
    Error(Lerr),
    Fun(Lfun),
}

impl PartialEq for Lval {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Lval::Sym(a) => match other {
                Lval::Sym(b) => true,
                _ => false,
            },
            Lval::Num(a) => match other {
                Lval::Num(b) => a == b,
                _ => false,
            },
            Lval::Sexpr(a) => match other {
                Lval::Sexpr(b) => a == b,
                _ => false,
            },
            Lval::Qexpr(a) => match other {
                Lval::Qexpr(b) => a == b,
                _ => false,
            },
            Lval::Error(a) => match other {
                Lval::Error(b) => a == b,
                _ => false,
            },
            Lval::Fun(_) => match other {
                Lval::Fun(_) => true,
                _ => false,
            },
        }
    }
}

impl fmt::Debug for Lval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lval").field("x", &self).finish()
    }
}

pub type Lenv = HashMap<String, Lval>;
pub type Lfun = fn(&mut Lenv, Vec<Lval>) -> Lval;

// pub fn init_env() -> Lenv {
//     let mut env = Lenv::new();
//     init_builtins(&mut env);
//     env
// }

pub fn add_builtin(env: &mut Lenv, sym: &str, fun: Lfun) {
    env.insert(sym.to_string(), Lval::Fun(fun));
    // env.insert(sym.to_string(), Lval::Num(1_f64));
}

// FromIterator<Lval>` is not implemented for `Result<Vec<Lval>, _>
impl FromIterator<Lval> for Result<Vec<Lval>, Lerr> {
    fn from_iter<I: IntoIterator<Item = Lval>>(iter: I) -> Self {
        let mut c = vec![];

        for i in iter {
            match i {
                Lval::Error(e) => return Err(e),
                _ => c.push(i),
            }
        }

        Ok(c)
    }
}

fn is_err(expr: &Lval) -> bool {
    match expr {
        Lval::Error(_) => true,
        _ => false,
    }
}

fn is_num(expr: &Lval) -> bool {
    match expr {
        Lval::Num(_) => true,
        _ => false,
    }
}

fn to_num(expr: &Lval) -> Option<f64> {
    if let Lval::Num(n) = expr {
        Some(*n)
    } else {
        None
    }
}

fn is_qexpr(expr: &Lval) -> bool {
    if let Lval::Qexpr(_) = expr {
        true
    } else {
        false
    }
}

fn to_fun(expr: &Lval) -> Option<Lfun> {
    if let Lval::Fun(s) = expr {
        Some(*s)
    } else {
        None
    }
}
