#![allow(unused_imports,dead_code,unused_variables)]
extern crate cam;

use std::rc::Rc;
use std::io;
use std::str::FromStr;
use std::borrow::Borrow;

use cam::lang::*;
use cam::cam::{VM,Val};
use cam::compile::compile;
use cam::string::Str;
use cam::sexp::Sexp;

fn main() {
    let mut stdin = io::stdin();
    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();

    let s = Sexp::from_str(&*line).unwrap();
    println!("got: {:?}", s);

    // let exp = Exp::Lit(Lit::String(Str::new("foo")));
    // let code = compile(&exp);
    // let val = VM::run(code);
    // println!("what's up, doc: {:?}", val);
}
