use std::rc::{Rc};

pub type VarIndex = u32;
// TODO?: use usize for arity everywhere except in representation of bytecode.
// otherwise it just gets annoying.
pub type Arity = u32;
pub type Ident = Rc<String>;

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Prim {
    Equal, Add, Sub, Mul, Div, Print,
}
pub use self::Prim::*;
impl Prim {
    pub fn arity(&self) -> Arity {
        match *self {
            Equal | Add | Sub | Mul | Div => 2,
            Print => 1,
        }
    }
}

#[derive(Clone,PartialEq,Eq,Debug)]
pub enum Lit { Nil, Bool(bool), Int(i64), String(Rc<String>), Prim(Prim) }
impl Lit {
    pub fn truthy(&self) -> bool {
        match *self { Lit::Nil | Lit::Bool(false) => false,
                      _ => true }
    }
}

pub type Expr = Box<Exp>;
pub enum Exp {
    Lit(Lit),
    Var(Ident, VarIndex),
    Lam(Vec<Ident>, Expr),
    App(Expr, Vec<Exp>),
    If(Expr, Expr, Expr),
    // simultaneous binding; no let-bound expression sees any of the others.
    Let(Vec<(Ident,Exp)>, Expr),
}
