pub mod builtin;
pub mod env;
pub mod eval;
pub mod parser;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::env::{Lenv, Lookup};
use std::{error::Error, fmt};

#[derive(Clone)]
pub enum Lval {
    Sym(String),
    Num(f64),
    Sexpr(Vec<Lval>),
    Qexpr(Vec<Lval>),
    Fun(Lfun),
    Lambda(Llambda),
    Str(String),
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
            Lval::Fun(_) => match other {
                Lval::Fun(_) => true,
                _ => false,
            },
            Lval::Str(_) => match other {
                Lval::Str(_) => true,
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
            Lval::Fun(_) => write!(f, "Fun"),
            Lval::Str(s) => write!(f, "Str::{}", s),
            Lval::Lambda(l) => write!(f, "Lambda::{{args:{:?}, body:{:?}}}", l.args, l.body),
        }
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
        let mut lenv = Lenv::new();
        lenv.push(Lookup::new());
        Llambda {
            args,
            body,
            env: lenv,
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
            LerrType::Interrupt => "User defined Error",
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
    Interrupt,
}

pub type Lfun = fn(&mut Lenv, Vec<Lval>) -> Result<Lval, Lerr>;

pub fn add_builtin(env: &mut Lenv, sym: &str, fun: Lfun) {
    env.insert(sym, Lval::Fun(fun));
}

fn to_num(expr: Lval) -> Option<f64> {
    if let Lval::Num(n) = expr {
        Some(n)
    } else {
        None
    }
}

fn to_sym(expr: Lval) -> Option<String> {
    if let Lval::Sym(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_str(expr: Lval) -> Option<String> {
    if let Lval::Str(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_qexpr(expr: Lval) -> Option<Vec<Lval>> {
    if let Lval::Qexpr(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

#[cfg(test)]
fn to_lambda(expr: &Lval) -> Option<Llambda> {
    if let Lval::Lambda(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn lisp(env: &mut Lenv, input: &str) -> String {
    if "env" == input {
        return format!("{:#?}", env.peek().unwrap());
    }

    let ast = parser::parse(input);
    match ast {
        Ok(tree) => format!("{:?}", eval::eval(env, tree.1)),
        Err(_) => String::from("<Parsing Error>"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lenv_nests_properly() {
        let mut env1 = Lenv::new();
        env1.push(Lookup::new());
        env1.insert("abc", Lval::Num(1_f64));
        env1.insert("def", Lval::Num(2_f64));

        {
            let mut env2 = env1.clone();
            env2.push(Lookup::new());
            env2.insert("abc", Lval::Num(3_f64));
            env2.insert("ghi", Lval::Num(4_f64));

            assert_eq!(env2.get("def").unwrap().to_owned(), Lval::Num(2_f64));
            assert_eq!(env2.get("abc").unwrap().to_owned(), Lval::Num(3_f64));
        }

        assert_eq!(env1.get("abc").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env1.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env1.get("ghi"), None);
    }

    #[test]
    fn lenv_inserts_last() {
        let mut env = Lenv::new();
        env.push(Lookup::new());
        env.insert("abc", Lval::Num(1_f64));
        env.insert_last("def", Lval::Num(2_f64));

        env.push(Lookup::new());
        env.insert("abc", Lval::Num(3_f64));
        env.insert_last("jkl", Lval::Num(5_f64));

        assert_eq!(env.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(3_f64));
        assert_eq!(env.get("jkl").unwrap().to_owned(), Lval::Num(5_f64));

        env.pop();

        assert_eq!(env.get("jkl").unwrap().to_owned(), Lval::Num(5_f64));
        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(1_f64));
    }
}
