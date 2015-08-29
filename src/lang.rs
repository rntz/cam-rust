use std::rc::{Rc};

pub type Arity = u32;
pub type Ident = Rc<String>;

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Prim { Equal, Add, Sub, Mul, Div, Print }
pub use self::Prim::*;
impl Prim {
    pub fn arity(&self) -> Arity {
        match *self {
            Equal | Add | Sub | Mul | Div => 2,
            Print => 1,
        }
    }
}

#[derive(Clone,PartialEq,Eq)]
pub enum Lit { Nil, Bool(bool), Int(i64), String(Rc<String>), Prim(Prim) }

pub type Expr = Box<Exp>;
pub enum Exp {
    Lit(Lit),
    Var(Ident, u32),
    Lam(Vec<Ident>, Expr),
    App(Expr, Vec<Exp>),
    // simultaneous binding; no let-bound expression sees any of the others.
    Let(Vec<(Ident,Exp)>, Expr),
}
