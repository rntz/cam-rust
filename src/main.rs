#![allow(unused_imports,dead_code,unused_variables)]
extern crate cam;

use std::rc::Rc;
use std::io;
use std::str::FromStr;
use std::borrow::Borrow;

use cam::cam::{VM,Val};
use cam::compile::compile;
use cam::lang::*;
use cam::parse::ParseFrom;
use cam::sexp::Sexp;
use cam::string::Str;

fn main() {
    let mut stdin = io::stdin();
    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();

    // parse s-expression
    let s = Sexp::from_str(&*line).unwrap();
    println!("SEXP: {:?}", s);

    // parse it into an expression
    let e = Exp::parse_from(&s).unwrap();
    println!("EXP:  {:?}", e);

    // compile it
    let code = compile(&e);
    println!("CODE: {:?}", code);

    // run it
    println!("\nRUNNING:");
    let val = VM::run(code);
    println!("VALUE: {:?}", val);
}
