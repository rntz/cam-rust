use std::collections::HashMap;
use std::fmt;

use string::Str;
use sexp::Sexp;
use parse::{ParseFrom};

pub type VarIndex = u32;
// TODO?: use usize for arity everywhere except in representation of bytecode.
// otherwise it just gets annoying.
pub type Arity = u32;
pub type Ident = Str;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Debug)]
pub enum Prim {
    Equal, Leq, Add, Sub, Mul, Div, Print,
}
pub use self::Prim::*;
impl Prim {
    pub fn arity(&self) -> Arity {
        match *self {
            Equal | Leq | Add | Sub | Mul | Div => 2,
            Print => 1,
        }
    }
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Debug)]
pub enum Lit { Nil, Bool(bool), Int(i64), String(Str), Prim(Prim) }
impl Lit {
    pub fn truthy(&self) -> bool {
        match *self { Lit::Nil | Lit::Bool(false) => false,
                      _ => true }
    }
}

pub type Expr = Box<Exp>;
#[derive(Debug)]
pub enum Exp {
    Lit(Lit),
    Var(Ident, VarIndex),
    Lam(Vec<Ident>, Expr),
    App(Expr, Vec<Exp>),
    If(Expr, Expr, Expr),
    // simultaneous binding; no let-bound expression sees any of the others.
    Let(Vec<(Ident,Exp)>, Expr),
}

// ---------- Displaying exps. ----------
impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(),fmt::Error> {
        match *self {
            Exp::Lit(ref l) => l.fmt(f),
            Exp::Var(ref name, _) => name.fmt(f),
            Exp::Lam(ref ids, ref body) => {
                if ids.is_empty() { return write!(f, "\\ -> {}", body) }
                try!(write!(f, "\\{}", ids[0]));
                for i in &ids[1..] { try!(write!(f, ",{}", i)) }
                write!(f, " -> {}", body)
            }
            Exp::App(ref e, ref args) => {
                if args.is_empty() { return write!(f, "{}()", e) }
                try!(write!(f, "{}({}", e, args[0]));
                for a in &args[1..] { try!(write!(f, ", {}", a)) }
                write!(f, ")")
            }
            Exp::If(ref cnd, ref thn, ref els) =>
                write!(f, "if {} then {} else {}", cnd, thn, els),
            Exp::Let(ref binds, ref body) => {
                if binds.is_empty() { return write!(f, "let in {}", body) }
                try!(write!(f, "let {} = {}", binds[0].0, binds[0].1));
                for &(ref id, ref e) in &binds[1..] {
                    try!(write!(f, ", {} = {}", id, e))
                }
                write!(f, " in {}", body)
            }
        }
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(),fmt::Error> {
        match *self {
            Lit::Nil => f.write_str("nil"),
            Lit::Bool(b) => b.fmt(f),
            Lit::Int(i) => i.fmt(f),
            Lit::String(ref s) => write!(f, "{:?}", s as &str),
            Lit::Prim(ref p) => p.fmt(f),
        }
    }
}

impl fmt::Display for Prim {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(),fmt::Error> {
        f.write_str(match *self {
            Equal => "eq", Leq => "le",
            Add => "add", Sub => "sub", Mul => "mul", Div => "div",
            Print => "print",
        })
    }
}

// ---------- Parsing sexps into exps. ----------
type ParseEnv = HashMap<Str,usize>;
type ParseResult<A> = Result<A,String>;
fn err<A,S>(s: S) -> ParseResult<A> where String: From<S> {
    Err(String::from(s))
}

fn parse(s: &Sexp, env: &mut ParseEnv) -> ParseResult<Exp> {
    Lit::parse_from(s).map(Exp::Lit).or_else(|_| {
        // but otherwise...
        match *s {
            Sexp::List(ref exps) if exps.is_empty()
                => err("empty list is not an expression"),
            Sexp::List(ref exps) => match exps[0] {
                Sexp::Symbol(ref s) => parse_form(env, &**s, exps),
                // List beginning with non-symbol is always application
                _ => parse_app(env, exps)
            },
            Sexp::Symbol(ref s) => match env.get(s) {
                Some(idx) => Ok(Exp::Var(s.clone(), *idx as VarIndex)),
                None => err("unbound variable")
            },
            _ => err("cannot parse expression")
        }
    })
}

fn parse_form(env: &mut ParseEnv, form: &str, exps: &[Sexp]) -> ParseResult<Exp>
{
    let args = &exps[1..];
    match form {
        "var" if args.len() != 1 => err("var takes 1 argument"),
        "var" => unimplemented!(), // TODO: var expressions
        "fn" if args.len() != 2 => err("fn takes 2 arguments"),
        "fn"  => {
            let params = match args[0] {
                Sexp::List(ref v) => v,
                _ => return err("fn takes a list of parameters"),
            };
            // The .rev() is necessary to maintain variable-binding order for
            // DeBruijn indices. Otherwise multi-argument functions take their
            // arguments in reverse of the expected order.
            params.iter().rev().map(|e| match *e {
                Sexp::Symbol(ref n) => Ok(n.clone()),
                // TODO?: allow strings?
                _ => err("fn parameters must be symbols")
            }).collect::<Result<Vec<_>,_>>().and_then(|ids| {
                let len = env.len();
                env.extend(ids.iter().enumerate()
                           .map(|e| (e.1.clone(), len + e.0)));
                let ids_new = ids.clone();
                let r = parse(&args[1], env)
                    .map(|body| Exp::Lam(ids_new, Box::new(body)));
                for x in ids { env.remove(&*x); }
                r
            })
        }
        "app" => parse_app(env, args),
        "if" if args.len() != 3 => err("if takes three arguments"),
        // TODO: make a macro for this shit. maybe just use "try!"?
        "if"  => parse(&args[0], env).and_then(|cnd| {
            parse(&args[1], env).and_then(|thn| {
                parse(&args[2], env).map(|els| {
                    Exp::If(Box::new(cnd), Box::new(thn), Box::new(els))})})}),
        "let" if args.len() != 2 => err("let takes 2 arguments"),
        "let" => {
            let ref _binds = args[0];
            let ref _body = args[1];
            unimplemented!()    // TODO: let expressions
        }
        // otherwise, function application
        _ => parse_app(env, exps)
    }
}

fn parse_app(env: &mut ParseEnv, exps: &[Sexp]) -> ParseResult<Exp> {
    parse(&exps[0], env).and_then(|func| {
        exps[1..].iter().map(|s| parse(s, env))
            .collect::<Result<Vec<_>,_>>()
            .map(|args| Exp::App(Box::new(func), args))
    })
}

impl<'a> ParseFrom<&'a Sexp> for Exp {
    type Error = String;
    fn parse_from(s: &Sexp) -> Result<Exp,String> {
        let mut env = HashMap::new();
        parse(s, &mut env)
    }
}

impl<'a> ParseFrom<&'a Sexp> for Lit {
    type Error = String;
    fn parse_from(s: &Sexp) -> ParseResult<Lit> {
        match *s {
            Sexp::Symbol(ref s) if s == "nil" => Ok(Lit::Nil),
            Sexp::Symbol(ref s) if s == "true" => Ok(Lit::Bool(true)),
            Sexp::Symbol(ref s) if s == "false" => Ok(Lit::Bool(false)),
            Sexp::Int(n) => Ok(Lit::Int(n)),
            Sexp::String(ref s) => Ok(Lit::String(s.clone())),
            // TODO: better error message on failure
            Sexp::Symbol(..) => Prim::parse_from(s).map(Lit::Prim),
            _ => err("invalid literal")
        }
    }
}

impl<'a> ParseFrom<&'a Sexp> for Prim {
    type Error = String;
    fn parse_from(s: &Sexp) -> ParseResult<Prim> {
        match *s {
            Sexp::Symbol(ref s) => match &**s {
                "eq" => Ok(Equal),
                "le" => Ok(Leq),
                "add" => Ok(Add), "sub" => Ok(Sub),
                "mul" => Ok(Mul), "div" => Ok(Div),
                "print" => Ok(Print),
                _ => Err(String::from("unrecognized prim"))
            },
            _ => Err(String::from("prims must be symbols"))
        }
    }
}
