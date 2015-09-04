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
    println!("sexp: {:?}", s);

    // parse it into an expression
    let e = Exp::parse_from(&s).unwrap();
    println!("exp: {:?}", e);

    // compile it
    let code = compile(&e);
    println!("code: {:?}", code);

    // run it
    let val = VM::run(code);
    println!("value: {:?}", val);

    // let exp = Exp::Lit(Lit::String(Str::new("foo")));
    // let code = compile(&exp);
    // let val = VM::run(code);
    // println!("what's up, doc: {:?}", val);
}
