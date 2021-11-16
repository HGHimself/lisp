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
            LerrType::Interrupt => "The program is exiting",
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

#[derive(Clone)]
pub struct Lenv {
    head: LinkedEnv,
}

type LinkedEnv = Option<Box<Env>>;
type Lookup = HashMap<String, Lval>;

#[derive(Clone, Debug)]
pub struct Env {
    lookup: Lookup,
    parent: LinkedEnv,
}

impl Lenv {
    pub fn new() -> Self {
        Lenv { head: None }
    }

    pub fn push(&mut self, lookup: Lookup) {
        let new_env = Box::new(Env {
            lookup,
            parent: self.head.take(),
        });

        self.head = Some(new_env);
    }

    pub fn pop(&mut self) -> Option<Lookup> {
        self.head.take().map(|env| {
            self.head = env.parent;
            env.lookup
        })
    }

    pub fn peek(&self) -> Option<&Lookup> {
        self.head.as_ref().map(|env| &env.lookup)
    }

    pub fn peek_mut(&mut self) -> Option<&mut Lookup> {
        self.head.as_mut().map(|env| &mut env.lookup)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn insert(&mut self, key: &str, lval: Lval) {
        self.peek_mut()
            .map(|node| node.insert(key.to_owned(), lval));
    }

    pub fn insert_last(&mut self, key: &str, lval: Lval) {
        let mut i = self.head.as_mut();

        while let Some(env) = i {
            i = env.parent.as_mut();
            if let None = i {
                env.lookup.insert(key.to_owned(), lval.clone());
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Lval> {
        let mut i = self.iter();

        while let Some(env) = i.next() {
            if let Some(v) = env.get(key) {
                return Some(v.clone());
            }
        }

        None
    }
}

impl Drop for Lenv {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_env) = cur_link {
            cur_link = boxed_env.parent.take();
        }
    }
}

pub struct Iter<'a> {
    next: Option<&'a Env>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Lookup;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|env| {
            self.next = env.parent.as_deref();
            &env.lookup
        })
    }
}

pub type Lfun = fn(&mut Lenv, Vec<Lval>) -> Lval;

pub fn init_env() -> Lenv {
    let mut env = Lenv::new();
    env.push(Lookup::new());
    init_builtins(&mut env);
    env
}

pub fn add_builtin(env: &mut Lenv, sym: &str, fun: Lfun) {
    env.insert(sym, Lval::Fun(fun));
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

#[cfg(test)]
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
