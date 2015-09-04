#![allow(unused_imports,dead_code,unused_variables)]
extern crate cam;

use std::mem;

use std::rc::Rc;
use std::io;
use std::io::Write;
use std::str::FromStr;
use std::borrow::Borrow;

use cam::cam::{VM,Val,Instr,Proto};
use cam::compile::compile;
use cam::lang::*;
use cam::parse::ParseFrom;
use cam::sexp::Sexp;
use cam::string::Str;

fn repl() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        print!("> ");
        stdout.flush().unwrap();
        line.clear();
        stdin.read_line(&mut line).unwrap();

        // parse s-expression
        let s = Sexp::from_str(&*line).unwrap();
        println!("SEXP: {}", s);

        // parse it into an expression
        let e = Exp::parse_from(&s).unwrap();
        println!("EXP:  {}", e);

        // compile it
        let code = compile(&e);
        println!("CODE: {:?}", code);

        // run it
        println!("\nRUNNING:");
        let val = VM::run(code);
        println!("VALUE: {:?}", val);
    }
}

fn main() {
    println!("align_of(Instr) = {}", mem::align_of::<Instr>());
    println!("size_of(Instr)  = {}", mem::size_of::<Instr>());
    println!("size_of(Prim)   = {}", mem::size_of::<Prim>());
    println!("size_of(Lit)    = {}", mem::size_of::<Lit>());
    println!("size_of(Rc<Proto>) = {}", mem::size_of::<Rc<Proto>>());
    repl();
}
