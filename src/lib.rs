pub mod alloc;
pub mod builtin;
pub mod eval;
pub mod parser;
pub mod prompt;
pub mod report;
pub mod sample;

use crate::builtin::init_builtins;
use std::{collections::HashMap, error::Error, fmt, iter::FromIterator};

#[derive(Clone)]
pub enum Lval {
    Sym(String),
    Num(f64),
    Sexpr(Vec<Lval>),
    Qexpr(Vec<Lval>),
    Error(Lerr),
    Fun(Lfun),
    Lambda(Llambda),
}

impl PartialEq for Lval {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Lval::Sym(a) => match other {
                Lval::Sym(b) => a == b,
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
            Lval::Lambda(a) => match other {
                Lval::Lambda(b) => a.body == b.body && a.args == b.args,
                _ => false,
            },
        }
    }
}

impl fmt::Debug for Lval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Lval::Sym(s) => write!(f, "Sym::{}", s),
            Lval::Num(n) => write!(f, "Num::{}", n),
            Lval::Sexpr(s) => write!(f, "Sexpr::{:?}", s),
            Lval::Qexpr(q) => write!(f, "Qexpr::{:?}", q),
            Lval::Error(e) => write!(f, "Error::{:?}", e),
            Lval::Fun(_) => write!(f, "Fun"),
            Lval::Lambda(l) => write!(f, "Lambda::{{args:{:?}, body:{:?}}}", l.args, l.body),
        }
    }
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

#[derive(Clone)]
pub struct Llambda {
    args: Vec<String>,
    body: Vec<Lval>,
    env: Lenv,
}

impl Llambda {
    fn new(args: Vec<String>, body: Vec<Lval>) -> Self {
        Llambda {
            args,
            body,
            env: Lenv::new(None),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lerr {
    etype: LerrType,
    details: String,
    message: String,
}

impl Lerr {
    fn new(etype: LerrType, message: String) -> Lerr {
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
            message,
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

#[derive(Clone, Debug)]
pub struct Lenv {
    lookup: HashMap<String, Lval>,
    parent: Option<Box<Lenv>>,
}

impl Lenv {
    fn new(parent: Option<Box<Lenv>>) -> Self {
        Lenv {
            lookup: HashMap::<String, Lval>::new(),
            parent,
        }
    }

    fn set_parent(&mut self, parent: Box<Lenv>) {
        self.parent = Some(parent);
    }

    fn insert(&mut self, k: String, v: Lval) -> Option<Lval> {
        self.lookup.insert(k, v)
    }

    // fn insert_topmost(&mut self, k: String, v: Lval) -> Option<Lval> {
    //     let mut current = self;
    //
    //     while let Some(p) = current.parent {
    //         current = p;
    //     }
    //
    //     current.lookup.insert(k, v)
    // }

    fn get(&self, k: &str) -> Option<&Lval> {
        let mut env = self;
        while let None = env.lookup.get(k) {
            if let Some(p) = &env.parent {
                env = p;
            } else {
                return None;
            }
        }
        env.lookup.get(k)
    }
}

pub type Lfun = fn(&mut Lenv, Vec<Lval>) -> Lval;

pub fn init_env() -> Lenv {
    let mut env = Lenv::new(None);
    init_builtins(&mut env);
    env
}

pub fn add_builtin(env: &mut Lenv, sym: &str, fun: Lfun) {
    env.insert(sym.to_string(), Lval::Fun(fun));
    // env.insert(sym.to_string(), Lval::Num(1_f64));
}

fn is_qexpr(expr: &Lval) -> bool {
    if let Lval::Qexpr(_) = expr {
        true
    } else {
        false
    }
}

fn to_num(expr: &Lval) -> Option<f64> {
    if let Lval::Num(n) = expr {
        Some(*n)
    } else {
        None
    }
}

fn to_sym(expr: &Lval) -> Option<String> {
    if let Lval::Sym(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_qexpr(expr: &Lval) -> Option<Vec<Lval>> {
    if let Lval::Qexpr(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_fun(expr: &Lval) -> Option<Lfun> {
    if let Lval::Fun(s) = expr {
        Some(*s)
    } else {
        None
    }
}

fn to_lambda(expr: &Lval) -> Option<Llambda> {
    if let Lval::Lambda(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_err(expr: &Lval) -> Option<Lerr> {
    if let Lval::Error(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lenv_nests_properly() {
        let mut env1 = Lenv::new(None);
        env1.insert(String::from("abc"), Lval::Num(1_f64));
        env1.insert(String::from("def"), Lval::Num(2_f64));

        let mut env2 = Lenv::new(Some(Box::new(env1.clone())));
        env2.insert(String::from("abc"), Lval::Num(3_f64));
        env2.insert(String::from("ghi"), Lval::Num(4_f64));

        assert_eq!(env2.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env2.get("abc").unwrap().to_owned(), Lval::Num(3_f64));

        assert_eq!(env1.get("abc").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env1.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env1.get("ghi"), None);
    }
}
