#![allow(unused_imports,dead_code,unused_variables)]
extern crate cam;

use std::rc::Rc;

use cam::lang::*;
use cam::cam::{VM,Val};
use cam::compile::compile;

fn main() {
    let exp = Exp::Lit(Lit::String(Rc::new(String::from("foo"))));
    let code = compile(&exp);
    let val = VM::run(code);
    println!("what's up, doc: {:?}", val);
}
