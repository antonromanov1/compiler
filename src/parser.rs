use std::cell::RefCell;
use std::rc::Rc;

use crate::ir::*;
use crate::lexer::*;
use crate::symbols::*;

pub struct Parser {
    // lex - lexical analyzer for this parser
    // look - lookahead token
    // top - current or top symbol table
    // enclosing - pointer to enclosing loop
    // temp_count - number of temporary variables
    // labels - number of labels
    lex: Lexer,
    look: Token,
    top: Option<Box<Env>>,
    enclosing: bool,
    temp_count: Rc<RefCell<u8>>,
    labels: Rc<RefCell<u32>>,
    used: u32,
}

impl Parser {
    fn move_(&mut self) {
        self.look = self.lex.scan();
    }

    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            lex: l,
            look: Token::Token(TokenBase { tag: 0 }),
            top: None,
            enclosing: false,
            temp_count: Rc::new(RefCell::new(0)),
            labels: Rc::new(RefCell::new(0)),
            used: 0,
        };
        p.move_();
        p
    }

    fn error(&self, s: &str) -> ! {
        println!("Syntax error on line {}: {}", self.lex.line_num, s);
        std::process::exit(0);
    }

    fn match_(&mut self, t: u32) {
        match self.look.get_tag() {
            Some(tag) => {
                if tag == t {
                    self.move_();
                } else {
                    self.error(&self.look.to_string());
                }
            }
            None => panic!("End of file reached"),
        };
    }

    pub fn program(&mut self) {
        let s = self.block();
        let begin = new_label(self.labels.clone());
        let after = new_label(self.labels.clone());
        emit_label(begin);
        (*s).gen(begin, after, 0);
        emit_label(after);
    }

    fn block(&mut self) -> Box<dyn StmtAble> {
        self.match_('{' as u32);
        self.top = Some(Box::new(Env::new(self.top.take())));
        self.decls();
        let s = self.stmts();
        self.match_('}' as u32);
        self.top = self.top.take().unwrap().prev;
        s
    }

    fn decls(&mut self) {
        while self.look.get_tag().unwrap() == Tag::Basic as u32 {
            let p = self.type_();
            let tok = self.look.clone();
            self.match_(Tag::Id as u32);
            self.match_(';' as u32);
            let w = match tok.clone() {
                Token::Word(word) => match word {
                    Word::Word(base) => base,
                    _ => panic!("decls"),
                },
                _ => panic!("decls"),
            };
            let id = Id::new(w.clone(), p.clone(), self.used);
            (*self.top.as_mut().unwrap()).put(w, id);
            self.used += p.get_width();
        }
    }

    fn type_(&mut self) -> TypeBase {
        let p = match self.look.clone() {
            Token::Word(word) => match word {
                Word::Type(t) => t,
                Word::Word(word_base) => {
                    self.error(&word_base.lexeme);
                }
            },
            _ => panic!("Expected type"),
        };
        self.match_(Tag::Basic as u32);
        p
    }

    fn stmts(&mut self) -> Box<dyn StmtAble> {
        if self.look.get_tag().unwrap() == '}' as u32 {
            Box::new(Null {})
        } else {
            Box::new(Seq::new(self.stmt(), self.stmts(), self.labels.clone()))
        }
    }

    fn stmt(&mut self) -> Box<dyn StmtAble> {
        let tag = self.look.get_tag().unwrap();

        if tag == ';' as u32 {
            self.move_();
            Box::new(Null {})
        } else if tag == Tag::If as u32 {
            self.match_(Tag::If as u32);
            self.match_('(' as u32);
            let x = self.bool_();
            self.match_(')' as u32);

            let s1 = self.stmt();
            if self.look.get_tag().unwrap() != Tag::Else as u32 {
                return Box::new(If::new(x, s1, self.lex.line_num, self.labels.clone()));
            }
            self.match_(Tag::Else as u32);
            let s2 = self.stmt();
            Box::new(Else::new(x, s1, s2, self.lex.line_num, self.labels.clone()))
        } else if tag == Tag::While as u32 {
            self.enclosing = true;
            let mut while_node = Box::new(While::new(self.lex.line_num, self.labels.clone()));
            self.match_(Tag::While as u32);
            self.match_('(' as u32);
            let x = self.bool_();
            self.match_(')' as u32);
            let s = self.stmt();
            (*while_node).init(x, s);
            while_node
        } else if tag == Tag::Break as u32 {
            self.match_(Tag::Break as u32);
            self.match_(';' as u32);

            if !self.enclosing {
                panic!("unenclosed break");
            }
            Box::new(Break {})
        } else if tag == '{' as u32 {
            self.block()
        } else {
            self.assign()
        }
    }

    fn assign(&mut self) -> Box<dyn StmtAble> {
        let stmt: Box<dyn StmtAble>;
        let t = self.look.clone();

        self.match_(Tag::Id as u32);

        let w = match t.clone() {
            Token::Word(word) => match word {
                Word::Word(base) => base,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let id = (*self.top.as_ref().unwrap()).get(&w);
        match id {
            None => panic!("Undeclared"), // TODO: add temporary to_string
            _ => {}
        }

        self.match_('=' as u32);
        stmt = Box::new(Set::new(Box::new(id.unwrap()), self.bool_()));
        stmt
    }

    fn bool_(&mut self) -> Box<dyn ExprAble> {
        let mut x = self.join();
        while self.look.get_tag().unwrap() == Tag::Or as u32 {
            let tok = self.look.clone();
            self.move_();
            x = Box::new(Or::new(
                tok,
                x,
                self.join(),
                self.temp_count.clone(),
                self.labels.clone(),
            ));
        }
        x
    }

    fn join(&mut self) -> Box<dyn ExprAble> {
        let mut x = self.equality();
        while self.look.get_tag().unwrap() == Tag::And as u32 {
            let tok = self.look.clone();
            self.move_();
            x = Box::new(And::new(
                tok,
                x,
                self.equality(),
                self.temp_count.clone(),
                self.labels.clone(),
            ));
        }
        x
    }

    fn equality(&mut self) -> Box<dyn ExprAble> {
        let mut x = self.rel();
        while self.look.get_tag().unwrap() == Tag::Eq_ as u32
            || self.look.get_tag().unwrap() == Tag::Ne as u32
        {
            let tok = self.look.clone();
            self.move_();
            x = Box::new(Rel::new(
                tok,
                x,
                self.rel(),
                self.temp_count.clone(),
                self.labels.clone(),
            ));
        }
        x
    }

    fn rel(&mut self) -> Box<dyn ExprAble> {
        let x = self.expr();
        let tag = self.look.get_tag().unwrap();

        if tag == '<' as u32 || tag == Tag::Le as u32 || tag == Tag::Ge as u32 || tag == '>' as u32
        {
            let tok = self.look.clone();
            self.move_();
            Box::new(Rel::new(
                tok,
                x,
                self.expr(),
                self.temp_count.clone(),
                self.labels.clone(),
            ))
        } else {
            x
        }
    }

    fn expr(&mut self) -> Box<dyn ExprAble> {
        let mut x = self.term();
        while self.look.get_tag().unwrap() == '+' as u32
            || self.look.get_tag().unwrap() == '-' as u32
        {
            let tok = self.look.clone();
            self.move_();
            x = Box::new(Arith::new(
                tok,
                x,
                self.term(),
                self.lex.line_num,
                self.temp_count.clone(),
            ));
        }
        x
    }

    fn term(&mut self) -> Box<dyn ExprAble> {
        let mut x = self.unary();
        while self.look.get_tag().unwrap() == '*' as u32
            || self.look.get_tag().unwrap() == '/' as u32
        {
            let tok = self.look.clone();
            self.move_();
            x = Box::new(Arith::new(
                tok,
                x,
                self.unary(),
                self.lex.line_num,
                self.temp_count.clone(),
            ));
        }
        x
    }

    fn unary(&mut self) -> Box<dyn ExprAble> {
        if self.look.get_tag().unwrap() == '-' as u32 {
            self.move_();
            Box::new(Unary::new(
                Token::Word(Word::Word(word_minus())),
                self.unary(),
                self.temp_count.clone(),
            ))
        } else if self.look.get_tag().unwrap() == '!' as u32 {
            let tok = self.look.clone();
            self.move_();
            Box::new(Not::new(
                tok,
                self.unary(),
                self.temp_count.clone(),
                self.labels.clone(),
            ))
        } else {
            self.factor()
        }
    }

    fn factor(&mut self) -> Box<dyn ExprAble> {
        match self.look.get_tag() {
            Some(tag) => {
                if tag == '(' as u32 {
                    self.move_();
                    let x = self.bool_();
                    self.match_(')' as u32);
                    return x;
                } else if tag == Tag::Num as u32 {
                    let x = Box::new(Constant::new(self.look.clone(), type_int()));
                    self.move_();
                    return x;
                } else if tag == Tag::Real as u32 {
                    let x = Box::new(Constant::new(self.look.clone(), type_float()));
                    self.move_();
                    return x;
                } else if tag == Tag::True as u32 {
                    let x = Box::new(constant_true());
                    self.move_();
                    return x;
                } else if tag == Tag::False as u32 {
                    let x = Box::new(constant_false());
                    self.move_();
                    return x;
                } else if tag == Tag::Id as u32 {
                    let s = self.look.to_string();
                    #[allow(unused_assignments)]
                    let mut id: Option<Id> = None;

                    match &self.look {
                        Token::Word(word) => match &word {
                            Word::Word(w) => {
                                id = (*self.top.as_ref().unwrap()).get(&w);
                            }
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }

                    match id {
                        None => self.error(&format!("{} undeclared", s)),
                        _ => {}
                    }
                    self.move_();
                    return Box::new(id.unwrap());
                } else {
                    self.error(&format!("{}", self.look.to_string()));
                }
            }
            None => panic!("End of file reached"),
        }
    }
}
