use crate::{builtin::init_builtins, Lval};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Lenv {
    head: LinkedEnv,
}

type LinkedEnv = Option<Box<Env>>;
pub type Lookup = HashMap<String, Lval>;

#[derive(Clone, Debug)]
pub struct Env {
    lookup: Lookup,
    parent: LinkedEnv,
}

#[wasm_bindgen]
impl Lenv {
    pub fn new() -> Self {
        Lenv { head: None }
    }
}

impl Lenv {
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

#[wasm_bindgen]
pub fn init_env() -> Lenv {
    let mut env = Lenv::new();
    env.push(Lookup::new());
    init_builtins(&mut env);
    env
}
