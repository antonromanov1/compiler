use std::collections::HashMap;

use crate::lexer::*;
use crate::ir::*;

pub struct Env {
    table: HashMap<WordBase, Id>,
    prev: Option<Box<Env>>,
}

impl Env {
    #[allow(dead_code)]
    pub fn new(n: Option<Box<Env>>) -> Env {
        Env {
            table: HashMap::new(),
            prev: n,
        }
    }

    #[allow(dead_code)]
    pub fn put(&mut self, w: WordBase, i: Id) {
        self.table.insert(w, i);
    }

    #[allow(dead_code)]
    pub fn get(&self, w: &WordBase) -> Option<Id> {
        match self.table.get(w) {
            Some(id) => {
                return Some(id.clone());
            },
            None => {},
        };

        let mut e = &(self.prev);
        match e {
            Some(ptr) => {

                loop {
                    match (*ptr).table.get(w) {
                        Some(id) => {
                            return Some(id.clone());
                        },
                        None => {
                            e = &(e.as_ref().unwrap().prev);
                            match e {
                                Some(_a) => continue,
                                None => break,
                            }
                        },
                    };
                }

            },
            None => {
                return None;
            },
        };
        None
    }
}

/*
// a work piece
pub fn block() {
    let top = None;
    let mut top = Some(Box::new(Env::new(top)));
    top = top.unwrap().prev;
}
*/